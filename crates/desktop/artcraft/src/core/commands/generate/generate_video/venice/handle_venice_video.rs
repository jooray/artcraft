use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_status::TaskStatus;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info, warn};
use sqlite_tasks::queries::task::Task;
use tauri::{AppHandle, Manager};
use tokens::tokens::media_files::MediaFileToken;
use venice_client::client::venice_client::VeniceClient;
use venice_client::creds::venice_api_key::VeniceApiKey;
use venice_client::requests::video::queue_video_request::VeniceQueueVideoRequest;
use venice_client::requests::video::retrieve_video_request::VeniceRetrieveVideoRequest;
use venice_client::requests::video::retrieve_video_response::VeniceVideoRetrieveResponse;

use crate::core::api_adapters::models::video::tauri_video_model_to_generation_model::tauri_video_model_to_generation_model;
use crate::core::commands::enqueue::generate_error::{BadInputReason, GenerateError};
use crate::core::commands::enqueue::task_enqueue_success::{generation_model_to_task_model_type, TaskEnqueueSuccess};
use crate::core::commands::generate::generate_image::providers::venice::handle_venice::load_venice_api_key;
use crate::core::commands::generate::generate_image::providers::venice::complete_venice_task::mark_venice_task_failed_and_notify;
use crate::core::commands::generate::generate_image::providers::artcraft_router::utils::map_media_files_to_urls::map_media_file_tokens_to_cdn_urls;
use crate::core::commands::generate::generate_video::request::TauriGenerateVideoRequest;
use crate::core::commands::generate::generate_video::venice::complete_venice_video_task::complete_venice_video_task;
use crate::core::commands::generate::generate_video::venice::venice_video_model::{map_aspect_ratio, map_resolution, venice_video_model_spec, VENICE_MAX_VIDEO_BATCH};
use crate::core::events::basic_sendable_event_trait::BasicSendableEvent;
use crate::core::events::functional_events::credits_balance_changed_event::CreditsBalanceChangedEvent;
use crate::core::events::generation_events::generation_enqueue_success_event::GenerationEnqueueSuccessEvent;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::data_dir::trait_data_subdir::DataSubdir;
use crate::core::state::task_database::TaskDatabase;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;

const VIDEO_POLL_INTERVAL: Duration = Duration::from_secs(10);
const MAX_VIDEO_POLLS: u32 = 240; // ~40 minutes
const MAX_CONSECUTIVE_RETRIEVE_ERRORS: u32 = 10;

/// Enqueue a Venice.ai BYOK video generation.
///
/// Venice video is asynchronous: we queue the job, persist the local task and
/// poll for the result in a spawned background task (the desktop's third-party
/// polling thread only handles Fal).
pub async fn handle_venice_video_enqueue(request: &TauriGenerateVideoRequest, app: &AppHandle, app_env_configs: &AppEnvConfigs, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, credential_cache: &ProviderCredentialLoadingCache) -> Result<(), GenerateError> {
  let api_key = load_venice_api_key(credential_cache)?;

  // Logged in: upload results to the ArtCraft library. Logged out: save locally.
  let use_library_upload = storyteller_creds_manager.get_credentials().ok().flatten().is_some();

  let tauri_model = request.model.ok_or_else(GenerateError::no_model_specified)?;

  let spec = venice_video_model_spec(tauri_model).ok_or_else(|| GenerateError::BadInput(BadInputReason::WrongImageArguments(format!("Model {:?} cannot use a Venice account.", tauri_model))))?;

  reject_unsupported_video_inputs(request)?;

  let prompt = request.prompt.clone().unwrap_or_default();
  if prompt.trim().is_empty() {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments("A prompt is required for Venice video generation.".to_string())));
  }

  let duration_seconds = request.duration_seconds.ok_or_else(|| GenerateError::BadInput(BadInputReason::WrongImageArguments("A duration is required for Venice video generation.".to_string())))?;
  let duration = format!("{}s", duration_seconds);

  let image_url = resolve_frame_url(&request.start_frame_image_media_token, app_env_configs).await?;
  let end_image_url = resolve_frame_url(&request.end_frame_image_media_token, app_env_configs).await?;

  if end_image_url.is_some() && image_url.is_none() {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments("Venice needs a start frame before an end frame.".to_string())));
  }

  // Venice exposes separate text-to-video and image-to-video model ids.
  let (venice_model, aspect_ratio_options, resolution_options) = if image_url.is_some() {
    let image_to_video = spec.image_to_video.ok_or_else(|| GenerateError::BadInput(BadInputReason::WrongImageArguments(format!("Model {:?} does not support image-to-video on Venice yet.", tauri_model))))?;
    (image_to_video, &[][..] as &[&str], spec.image_to_video_resolutions)
  } else {
    (spec.text_to_video, spec.text_to_video_aspect_ratios, spec.text_to_video_resolutions)
  };

  let aspect_ratio = resolve_aspect_ratio(request.aspect_ratio, aspect_ratio_options)?;
  let resolution = resolve_resolution(request.resolution, resolution_options)?;

  let queue_request = VeniceQueueVideoRequest {
    model: venice_model.to_string(),
    duration,
    prompt: Some(prompt),
    negative_prompt: request.negative_prompt.clone().filter(|value| !value.trim().is_empty()),
    aspect_ratio,
    resolution,
    image_url,
    end_image_url,
    audio: request.generate_audio,
  };

  info!("[VeniceVideo] Queueing generation: {:?}", queue_request);

  let client = VeniceClient::new(api_key.clone()).map_err(|err| GenerateError::AnyhowError(anyhow!("Could not create the Venice client: {:?}", err)))?;

  let queue_response = client.queue_video(&queue_request).await?;

  info!("[VeniceVideo] Queued {} as {}", queue_request.model, queue_response.queue_id);

  let success = TaskEnqueueSuccess {
    task_type: TaskType::VideoGeneration,
    model: Some(tauri_video_model_to_generation_model(tauri_model)),
    provider: GenerationProvider::Venice,
    provider_job_id: Some(queue_response.queue_id.clone()),
    maybe_queue_status_url: None,
    maybe_queue_response_url: queue_response.download_url.clone(),
    maybe_prompt_token: None,
  };

  let task_id = success.insert_into_task_database_with_frontend_payload(task_database, request.frontend_caller, request.frontend_subscriber_id.as_deref(), request.frontend_subscriber_payload.as_deref()).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not persist the Venice task: {:?}", err)))?;

  let task = build_task_record(task_id, &success, request);

  // Mirror the enqueue path so the task queue UI shows the job immediately.
  GenerationEnqueueSuccessEvent { action: success.to_frontend_event_action(), service: success.to_frontend_event_service(), model: success.model.clone() }.send_infallible(app);
  CreditsBalanceChangedEvent {}.send_infallible(app);

  let job = VeniceVideoJob { api_key, venice_model: queue_request.model.clone(), queue_id: queue_response.queue_id, maybe_download_url: queue_response.download_url, task, use_library_upload };

  let app_handle = app.clone();
  let app_env_configs = app_env_configs.clone();
  let storyteller_creds_manager = storyteller_creds_manager.clone();
  let task_database = task_database.clone();
  tauri::async_runtime::spawn(async move {
    run_venice_video_generation(&app_handle, &app_env_configs, &storyteller_creds_manager, &task_database, &job).await;
  });

  Ok(())
}

// ── Background polling ──

struct VeniceVideoJob {
  api_key: VeniceApiKey,
  venice_model: String,
  queue_id: String,
  maybe_download_url: Option<String>,
  task: Task,
  use_library_upload: bool,
}

async fn run_venice_video_generation(app: &AppHandle, app_env_configs: &AppEnvConfigs, storyteller_creds_manager: &StorytellerCredentialManager, task_database: &TaskDatabase, job: &VeniceVideoJob) {
  info!("[VeniceVideo] Polling task {} (queue {})", job.task.id.as_str(), job.queue_id);

  match poll_and_complete(app, app_env_configs, storyteller_creds_manager, task_database, job).await {
    Ok(()) => info!("[VeniceVideo] Task {} fully handled", job.task.id.as_str()),
    Err(err) => {
      error!("[VeniceVideo] Task {} failed: {:?}", job.task.id.as_str(), err);
      mark_venice_task_failed_and_notify(app, task_database, &job.task, &format!("Venice video generation failed: {:?}", err)).await;
    },
  }
}

async fn poll_and_complete(app: &AppHandle, app_env_configs: &AppEnvConfigs, storyteller_creds_manager: &StorytellerCredentialManager, task_database: &TaskDatabase, job: &VeniceVideoJob) -> Result<(), GenerateError> {
  let client = VeniceClient::new(job.api_key.clone()).map_err(|err| GenerateError::AnyhowError(anyhow!("Could not create the Venice client: {:?}", err)))?;

  let retrieve_request = VeniceRetrieveVideoRequest { model: job.venice_model.clone(), queue_id: job.queue_id.clone() };

  let mut consecutive_errors: u32 = 0;

  for _ in 0..MAX_VIDEO_POLLS {
    let maybe_video_bytes = match client.retrieve_video(&retrieve_request).await {
      Ok(VeniceVideoRetrieveResponse::Video { bytes }) => Some(bytes),
      Ok(VeniceVideoRetrieveResponse::CompletedWithDownloadUrl) => {
        let download_url = job.maybe_download_url.as_deref().ok_or_else(|| GenerateError::AnyhowError(anyhow!("Venice reported completion for queue {} but provided no download URL", job.queue_id)))?;
        Some(client.download_url_bytes(download_url).await?)
      },
      Ok(VeniceVideoRetrieveResponse::Processing { status, maybe_average_execution_time, maybe_execution_duration }) => {
        info!(
          "[VeniceVideo] Task {} still processing (status: {}, average: {:?}, elapsed: {:?})",
          job.task.id.as_str(),
          status,
          maybe_average_execution_time,
          maybe_execution_duration,
        );
        consecutive_errors = 0;
        None
      },
      Err(err) => {
        consecutive_errors += 1;
        warn!("[VeniceVideo] Retrieve failed ({}/{}): {:?}", consecutive_errors, MAX_CONSECUTIVE_RETRIEVE_ERRORS, err);
        if consecutive_errors >= MAX_CONSECUTIVE_RETRIEVE_ERRORS {
          return Err(err.into());
        }
        None
      },
    };

    if let Some(bytes) = maybe_video_bytes {
      let video_file_path = save_video_bytes_to_temp(&bytes, app)?;
      return complete_venice_video_task(app, app_env_configs, task_database, storyteller_creds_manager, &job.task, &video_file_path, job.use_library_upload).await;
    }

    tokio::time::sleep(VIDEO_POLL_INTERVAL).await;
  }

  Err(GenerateError::AnyhowError(anyhow!("Venice video generation for queue {} timed out after ~{} minutes", job.queue_id, (MAX_VIDEO_POLLS * VIDEO_POLL_INTERVAL.as_secs() as u32) / 60)))
}

// ── Helpers ──

fn build_task_record(task_id: tokens::tokens::sqlite::tasks::TaskId, success: &TaskEnqueueSuccess, request: &TauriGenerateVideoRequest) -> Task {
  Task {
    id: task_id,
    status: TaskStatus::Pending,
    task_type: success.task_type,
    model_type: generation_model_to_task_model_type(&success.model),
    provider: success.provider,
    provider_job_id: success.provider_job_id.clone(),
    queue_status_url: None,
    queue_response_url: success.maybe_queue_response_url.clone(),
    prompt_token: None,
    frontend_caller: request.frontend_caller,
    frontend_subscriber_id: request.frontend_subscriber_id.clone(),
    frontend_subscriber_payload: request.frontend_subscriber_payload.clone(),
  }
}

/// Venice video only generates from text or a start/end frame for now.
fn reject_unsupported_video_inputs(request: &TauriGenerateVideoRequest) -> Result<(), GenerateError> {
  let has_references = request.reference_image_media_tokens.as_ref().is_some_and(|tokens| !tokens.is_empty())
    || request.reference_video_media_tokens.as_ref().is_some_and(|tokens| !tokens.is_empty())
    || request.reference_audio_media_tokens.as_ref().is_some_and(|tokens| !tokens.is_empty())
    || request.reference_character_tokens.as_ref().is_some_and(|tokens| !tokens.is_empty());

  if has_references {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments(
      "The Venice provider supports text-to-video and start/end-frame video only; remove the reference media and try again.".to_string(),
    )));
  }

  if request.video_batch_count.unwrap_or(1) > VENICE_MAX_VIDEO_BATCH {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments(format!(
      "Venice generates {} video per request; set the generation count to {}.",
      VENICE_MAX_VIDEO_BATCH, VENICE_MAX_VIDEO_BATCH,
    ))));
  }

  Ok(())
}

async fn resolve_frame_url(maybe_token: &Option<MediaFileToken>, app_env_configs: &AppEnvConfigs) -> Result<Option<String>, GenerateError> {
  let Some(token) = maybe_token else {
    return Ok(None);
  };

  let urls = map_media_file_tokens_to_cdn_urls(std::slice::from_ref(token), &app_env_configs.storyteller_host).await?;
  Ok(urls.into_iter().next())
}

fn resolve_aspect_ratio(value: Option<artcraft_router::api::router_aspect_ratio::RouterAspectRatio>, supported: &[&str]) -> Result<Option<String>, GenerateError> {
  let Some(value) = value else {
    return Ok(None);
  };

  match map_aspect_ratio(value) {
    None => Ok(None), // auto-style values: let the model decide
    Some(mapped) if supported.contains(&mapped) => Ok(Some(mapped.to_string())),
    Some(mapped) => Err(GenerateError::BadInput(BadInputReason::WrongImageArguments(format!(
      "This Venice model does not support the {} aspect ratio. Supported: {}.",
      mapped,
      supported.join(", "),
    )))),
  }
}

fn resolve_resolution(value: Option<artcraft_router::api::router_resolution::RouterResolution>, supported: &[&str]) -> Result<Option<String>, GenerateError> {
  let Some(value) = value else {
    return Ok(None);
  };

  if supported.is_empty() {
    return Ok(None); // the model doesn't take a resolution
  }

  match map_resolution(value) {
    Some(mapped) if supported.contains(&mapped) => Ok(Some(mapped.to_string())),
    _ => Err(GenerateError::BadInput(BadInputReason::WrongImageArguments(format!(
      "This Venice model does not support resolution {:?}. Supported: {}.",
      value,
      supported.join(", "),
    )))),
  }
}

/// Write the finished video to a temporary file.
fn save_video_bytes_to_temp(bytes: &[u8], app: &AppHandle) -> Result<PathBuf, GenerateError> {
  let app_data_root = app.state::<AppDataRoot>();
  let temp_dir = app_data_root.temp_dir().path();
  let path = temp_dir.join(format!("venice_video_{}.mp4", uuid_utils::uuid::generate_random_uuid()));

  std::fs::write(&path, bytes).map_err(GenerateError::IoError)?;
  info!("[VeniceVideo] Saved generated video to {:?} ({} bytes)", path, bytes.len());

  Ok(path)
}
