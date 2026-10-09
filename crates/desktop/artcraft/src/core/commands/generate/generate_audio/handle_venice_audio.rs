use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_status::TaskStatus;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info, warn};
use serde::Deserialize;
use serde_json::Value;
use sqlite_tasks::queries::task::Task;
use tauri::{AppHandle, Manager};
use venice_client::client::venice_client::VeniceClient;
use venice_client::creds::venice_api_key::VeniceApiKey;
use venice_client::requests::audio::queue_audio_request::VeniceQueueAudioRequest;
use venice_client::requests::audio::retrieve_audio_request::VeniceRetrieveAudioRequest;
use venice_client::requests::audio::retrieve_audio_response::VeniceAudioRetrieveResponse;

use crate::core::commands::enqueue::generate_error::{BadInputReason, GenerateError};
use crate::core::commands::enqueue::task_enqueue_success::{generation_model_to_task_model_type, TaskEnqueueSuccess};
use crate::core::commands::generate::generate_audio::complete_venice_audio_task::complete_venice_audio_task;
use crate::core::commands::generate::generate_audio::venice_audio_models::venice_audio_model_spec;
use crate::core::commands::generate::generate_image::providers::venice::complete_venice_task::mark_venice_task_failed_and_notify;
use crate::core::commands::generate::generate_image::providers::venice::handle_venice::load_venice_api_key;
use crate::core::commands::generate::omni::request::OmniRequest;
use crate::core::commands::generate::omni::OmniResponse;
use crate::core::events::basic_sendable_event_trait::BasicSendableEvent;
use crate::core::events::functional_events::credits_balance_changed_event::CreditsBalanceChangedEvent;
use crate::core::events::generation_events::common::GenerationModel;
use crate::core::events::generation_events::generation_enqueue_success_event::GenerationEnqueueSuccessEvent;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::data_dir::trait_data_subdir::DataSubdir;
use crate::core::state::task_database::TaskDatabase;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;

const AUDIO_POLL_INTERVAL: Duration = Duration::from_secs(10);
const MAX_AUDIO_POLLS: u32 = 60; // ~10 minutes
const MAX_CONSECUTIVE_RETRIEVE_ERRORS: u32 = 10;

/// The desktop frontend's audio request fields (see `OmniGenAudioRequest`).
#[derive(Deserialize, Debug)]
struct TauriGenerateAudioFields {
  model: Option<String>,
  prompt: Option<String>,
  lyrics: Option<String>,
  is_instrumental: Option<bool>,
  is_loopable: Option<bool>,
  speed: Option<f64>,
}

/// Enqueue a Venice.ai BYOK music / sound-effect generation.
///
/// Venice audio is asynchronous: we queue the job, persist the local task and
/// poll for the result in a spawned background task.
pub async fn handle_venice_audio_enqueue(request: &OmniRequest, app: &AppHandle) -> Result<OmniResponse, GenerateError> {
  let api_key = load_venice_api_key(&app.state::<ProviderCredentialLoadingCache>())?;

  // Logged in: upload results to the ArtCraft library. Logged out: save locally.
  let use_library_upload = app.state::<StorytellerCredentialManager>().get_credentials().ok().flatten().is_some();

  let fields: TauriGenerateAudioFields = serde_json::from_value(Value::Object(request.fields.clone())).map_err(|err| GenerateError::BadInput(BadInputReason::WrongImageArguments(format!("Invalid audio request: {}", err))))?;

  let model = fields.model.ok_or_else(GenerateError::no_model_specified)?;

  let spec = venice_audio_model_spec(&model).ok_or_else(|| GenerateError::BadInput(BadInputReason::WrongImageArguments(format!("Model {} cannot use a Venice account.", model))))?;

  let prompt = fields.prompt.clone().unwrap_or_default();
  if prompt.trim().is_empty() {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments("A prompt is required for Venice audio generation.".to_string())));
  }

  // Lyrics: user-supplied when present, otherwise auto-written by models that
  // can do it. Instrumental generations never carry lyrics.
  let lyrics = fields.lyrics.clone().unwrap_or_default();
  let lyrics = lyrics.trim();
  let is_instrumental = fields.is_instrumental == Some(true);

  let (lyrics_prompt, lyrics_optimizer) = if is_instrumental {
    (None, None)
  } else if !lyrics.is_empty() {
    if !spec.supports_lyrics {
      return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments("This model does not support custom lyrics; remove them or pick a music model that does.".to_string())));
    }
    (Some(lyrics.to_string()), None)
  } else if spec.lyrics_required {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments("This model needs lyrics; type them in the lyrics field or pick a model that writes its own.".to_string())));
  } else if spec.supports_lyrics_optimizer {
    (None, Some(true))
  } else {
    (None, None)
  };

  let queue_request = VeniceQueueAudioRequest { model: model.clone(), prompt, duration_seconds: None, force_instrumental: fields.is_instrumental, r#loop: fields.is_loopable, speed: fields.speed, lyrics_prompt, lyrics_optimizer };

  info!("[VeniceAudio] Queueing generation: {:?}", queue_request);

  let client = VeniceClient::new(api_key.clone()).map_err(|err| GenerateError::AnyhowError(anyhow!("Could not create the Venice client: {:?}", err)))?;

  let queue_response = client.queue_audio(&queue_request).await?;

  info!("[VeniceAudio] Queued {} as {}", queue_request.model, queue_response.queue_id);

  let success = TaskEnqueueSuccess { task_type: TaskType::AudioGeneration, model: Some(GenerationModel::Unknown(model.clone())), provider: GenerationProvider::Venice, provider_job_id: Some(queue_response.queue_id.clone()), maybe_queue_status_url: None, maybe_queue_response_url: None, maybe_prompt_token: None };

  let task_database = app.state::<TaskDatabase>();
  let task_id = success.insert_into_task_database_with_frontend_payload(&task_database, request.frontend_caller, request.frontend_subscriber_id.as_deref(), request.frontend_subscriber_payload.as_deref()).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not persist the Venice task: {:?}", err)))?;

  let task = build_task_record(task_id, &success, request);

  GenerationEnqueueSuccessEvent { action: success.to_frontend_event_action(), service: success.to_frontend_event_service(), model: success.model.clone() }.send_infallible(app);
  CreditsBalanceChangedEvent {}.send_infallible(app);

  let job = VeniceAudioJob { api_key, venice_model: model, queue_id: queue_response.queue_id.clone(), task, use_library_upload };

  let app_handle = app.clone();
  let task_database = task_database.inner().clone();
  let storyteller_creds_manager = app.state::<StorytellerCredentialManager>().inner().clone();
  tauri::async_runtime::spawn(async move {
    run_venice_audio_generation(&app_handle, &task_database, &storyteller_creds_manager, &job).await;
  });

  Ok(OmniResponse(serde_json::json!({
    "success": true,
    "inference_job_token": queue_response.queue_id,
    "all_job_tokens": [queue_response.queue_id],
  })).into())
}

// ── Background polling ──

struct VeniceAudioJob {
  api_key: VeniceApiKey,
  venice_model: String,
  queue_id: String,
  task: Task,
  use_library_upload: bool,
}

async fn run_venice_audio_generation(app: &AppHandle, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, job: &VeniceAudioJob) {
  info!("[VeniceAudio] Polling task {} (queue {})", job.task.id.as_str(), job.queue_id);

  match poll_and_complete(app, task_database, storyteller_creds_manager, job).await {
    Ok(()) => info!("[VeniceAudio] Task {} fully handled", job.task.id.as_str()),
    Err(err) => {
      error!("[VeniceAudio] Task {} failed: {:?}", job.task.id.as_str(), err);
      mark_venice_task_failed_and_notify(app, task_database, &job.task, &format!("Venice audio generation failed: {:?}", err)).await;
    },
  }
}

async fn poll_and_complete(app: &AppHandle, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, job: &VeniceAudioJob) -> Result<(), GenerateError> {
  let client = VeniceClient::new(job.api_key.clone()).map_err(|err| GenerateError::AnyhowError(anyhow!("Could not create the Venice client: {:?}", err)))?;

  let retrieve_request = VeniceRetrieveAudioRequest { model: job.venice_model.clone(), queue_id: job.queue_id.clone() };

  let mut consecutive_errors: u32 = 0;

  for _ in 0..MAX_AUDIO_POLLS {
    let maybe_audio = match client.retrieve_audio(&retrieve_request).await {
      Ok(VeniceAudioRetrieveResponse::Audio { bytes, content_type }) => Some((bytes, content_type)),
      Ok(VeniceAudioRetrieveResponse::Processing { status, maybe_average_execution_time, maybe_execution_duration }) => {
        info!("[VeniceAudio] Task {} still processing (status: {}, average: {:?}, elapsed: {:?})", job.task.id.as_str(), status, maybe_average_execution_time, maybe_execution_duration);
        consecutive_errors = 0;
        None
      },
      Err(err) => {
        consecutive_errors += 1;
        warn!("[VeniceAudio] Retrieve failed ({}/{}): {:?}", consecutive_errors, MAX_CONSECUTIVE_RETRIEVE_ERRORS, err);
        if consecutive_errors >= MAX_CONSECUTIVE_RETRIEVE_ERRORS {
          return Err(err.into());
        }
        None
      },
    };

    if let Some((bytes, content_type)) = maybe_audio {
      let audio_file_path = save_audio_bytes_to_temp(&bytes, &content_type, app)?;
      return complete_venice_audio_task(app, task_database, storyteller_creds_manager, &job.task, &audio_file_path, job.use_library_upload).await;
    }

    tokio::time::sleep(AUDIO_POLL_INTERVAL).await;
  }

  Err(GenerateError::AnyhowError(anyhow!("Venice audio generation for queue {} timed out after ~{} minutes", job.queue_id, (MAX_AUDIO_POLLS * AUDIO_POLL_INTERVAL.as_secs() as u32) / 60)))
}

// ── Helpers ──

fn build_task_record(task_id: tokens::tokens::sqlite::tasks::TaskId, success: &TaskEnqueueSuccess, request: &OmniRequest) -> Task {
  Task {
    id: task_id,
    status: TaskStatus::Pending,
    task_type: success.task_type,
    model_type: generation_model_to_task_model_type(&success.model),
    provider: success.provider,
    provider_job_id: success.provider_job_id.clone(),
    queue_status_url: None,
    queue_response_url: None,
    prompt_token: None,
    frontend_caller: request.frontend_caller,
    frontend_subscriber_id: request.frontend_subscriber_id.clone(),
    frontend_subscriber_payload: request.frontend_subscriber_payload.clone(),
  }
}

/// Save the finished audio to a temporary file, picking an extension from the
/// response content type.
fn save_audio_bytes_to_temp(bytes: &[u8], content_type: &str, app: &AppHandle) -> Result<PathBuf, GenerateError> {
  let extension = if content_type.contains("wav") {
    "wav"
  } else if content_type.contains("flac") {
    "flac"
  } else {
    "mp3"
  };

  let app_data_root = app.state::<AppDataRoot>();
  let temp_dir = app_data_root.temp_dir().path();
  let path = temp_dir.join(format!("venice_audio_{}.{}", uuid_utils::uuid::generate_random_uuid(), extension));

  std::fs::write(&path, bytes).map_err(GenerateError::IoError)?;
  info!("[VeniceAudio] Saved generated audio to {:?} ({} bytes)", path, bytes.len());

  Ok(path)
}
