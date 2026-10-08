use anyhow::anyhow;
use artcraft_api_defs::prompts::create_prompt::CreatePromptRequest;
use artcraft_client::endpoints::prompts::create_prompt::create_prompt;
use enums::common::generation::common_generation_mode::CommonGenerationMode;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_status::TaskStatus;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info, warn};
use sqlite_tasks::queries::task::Task;
use tauri::{AppHandle, Manager};
use tokens::tokens::prompts::PromptToken;
use uuid_utils::uuid::generate_random_uuid;
use venice_client::client::venice_client::VeniceClient;
use venice_client::creds::venice_api_key::VeniceApiKey;
use venice_client::requests::generate_image::generate_image_request::VeniceGenerateImageRequest;
use venice_client::utils::decode_base64_image;

use crate::core::api_adapters::models::image::tauri_image_model_to_common_model_type::tauri_image_model_to_common_model_type;
use crate::core::api_adapters::models::image::tauri_image_model_to_generation_model::tauri_image_model_to_generation_model;
use crate::core::commands::enqueue::generate_error::{BadInputReason, GenerateError};
use crate::core::commands::enqueue::task_enqueue_success::{generation_model_to_task_model_type, TaskEnqueueSuccess};
use crate::core::commands::generate::generate_image::providers::venice::complete_venice_task::complete_venice_task;
use crate::core::commands::generate::generate_image::providers::venice::venice_image_model::{map_aspect_ratio, map_quality, map_resolution, venice_model_spec, VeniceSizingMode, VENICE_MAX_VARIANTS};
use crate::core::commands::generate::generate_image::tauri_generate_image_request::TauriGenerateImageRequest;
use crate::core::commands::generate::generate_image::tauri_image_model::TauriImageModel;
use crate::core::events::basic_sendable_event_trait::BasicSendableEvent;
use crate::core::events::functional_events::credits_balance_changed_event::CreditsBalanceChangedEvent;
use crate::core::events::generation_events::generation_enqueue_success_event::GenerationEnqueueSuccessEvent;
use crate::core::providers::credentials::payload::provider_credential_payload::ProviderCredentialPayload;
use crate::core::providers::credentials::provider_credential_key::ProviderCredentialKey;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::data_dir::trait_data_subdir::DataSubdir;
use crate::core::state::task_database::TaskDatabase;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;

/// Enqueue a Venice.ai BYOK image generation.
///
/// Venice generates synchronously, so the work happens in a spawned background
/// task: we persist the local task and emit the enqueue event immediately, then
/// generate, upload the results and mark the task complete.
pub async fn handle_venice_enqueue(request: &TauriGenerateImageRequest, app: &AppHandle, app_env_configs: &AppEnvConfigs, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, credential_cache: &ProviderCredentialLoadingCache) -> Result<(), GenerateError> {
  let api_key = load_venice_api_key(credential_cache)?;

  // Logged in: upload results to the user's ArtCraft library. Logged out: save
  // them locally — Venice generation never requires an ArtCraft account.
  let use_library_upload = storyteller_creds_manager.get_credentials().ok().flatten().is_some();

  let prepared = prepare_venice_request(request)?;

  let maybe_prompt_token = if use_library_upload {
    create_prompt_record(request, prepared.tauri_model, app_env_configs, storyteller_creds_manager).await
  } else {
    info!("[Venice] No Storyteller credentials; results will be saved locally");
    None
  };

  let success = TaskEnqueueSuccess { task_type: TaskType::ImageGeneration, model: Some(tauri_image_model_to_generation_model(prepared.tauri_model)), provider: GenerationProvider::Venice, provider_job_id: None, maybe_queue_status_url: None, maybe_queue_response_url: None, maybe_prompt_token: maybe_prompt_token.clone() };

  let task_id = success.insert_into_task_database_with_frontend_payload(task_database, request.frontend_caller, request.frontend_subscriber_id.as_deref(), request.frontend_subscriber_payload.as_deref()).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not persist the Venice task: {:?}", err)))?;

  let task = build_task_record(task_id, &success, maybe_prompt_token.as_ref(), request);

  // Mirror the enqueue path so the task queue UI shows the job immediately.
  GenerationEnqueueSuccessEvent { action: success.to_frontend_event_action(), service: success.to_frontend_event_service(), model: success.model.clone() }.send_infallible(app);
  CreditsBalanceChangedEvent {}.send_infallible(app);

  let job = VeniceGenerationJob { api_key, venice_request: prepared.venice_request, maybe_prompt_token, task, use_library_upload };

  let app_handle = app.clone();
  let app_env_configs = app_env_configs.clone();
  let storyteller_creds_manager = storyteller_creds_manager.clone();
  let task_database = task_database.clone();
  tauri::async_runtime::spawn(async move {
    run_venice_generation(&app_handle, &app_env_configs, &storyteller_creds_manager, &task_database, &job).await;
  });

  Ok(())
}

// ── Background generation ──

struct VeniceGenerationJob {
  api_key: VeniceApiKey,
  venice_request: VeniceGenerateImageRequest,
  maybe_prompt_token: Option<PromptToken>,
  task: Task,
  use_library_upload: bool,
}

async fn run_venice_generation(app: &AppHandle, app_env_configs: &AppEnvConfigs, storyteller_creds_manager: &StorytellerCredentialManager, task_database: &TaskDatabase, job: &VeniceGenerationJob) {
  info!("[Venice] Starting generation for task {}", job.task.id.as_str());

  match generate_and_complete(app, app_env_configs, storyteller_creds_manager, task_database, job).await {
    Ok(()) => info!("[Venice] Task {} fully handled", job.task.id.as_str()),
    Err(err) => {
      error!("[Venice] Task {} failed: {:?}", job.task.id.as_str(), err);
      crate::core::commands::generate::generate_image::providers::venice::complete_venice_task::mark_venice_task_failed_and_notify(app, task_database, &job.task, &generate_error_reason(&err)).await;
    },
  }
}

async fn generate_and_complete(app: &AppHandle, app_env_configs: &AppEnvConfigs, storyteller_creds_manager: &StorytellerCredentialManager, task_database: &TaskDatabase, job: &VeniceGenerationJob) -> Result<(), GenerateError> {
  let client = VeniceClient::new(job.api_key.clone()).map_err(|err| GenerateError::AnyhowError(anyhow!("Could not create the Venice client: {:?}", err)))?;

  let response = client.generate_image(&job.venice_request).await?;

  if response.images.is_empty() {
    return Err(GenerateError::AnyhowError(anyhow!("Venice accepted request {} but returned no images", response.id,)));
  }

  let app_data_root = app.state::<AppDataRoot>();
  let media_file_paths = save_images_to_temp_files(&response.images, &app_data_root)?;

  complete_venice_task(app, app_env_configs, task_database, storyteller_creds_manager, &job.task, job.maybe_prompt_token.as_ref(), &media_file_paths, job.use_library_upload).await
}

// ── Helpers ──

/// A user-facing reason for a failed Venice generation.
fn generate_error_reason(err: &GenerateError) -> String {
  format!("Venice generation failed: {:?}", err)
}

pub(crate) fn load_venice_api_key(credential_cache: &ProviderCredentialLoadingCache) -> Result<VeniceApiKey, GenerateError> {
  // NB: Also used by the Venice video handler.
  match credential_cache.get_credentials(ProviderCredentialKey::VeniceApiKey) {
    Ok(Some(ProviderCredentialPayload::ApiKey(data))) => Ok(VeniceApiKey::from_str(data.as_str())),
    Ok(_) => Err(GenerateError::needs_venice_api_key()),
    Err(err) => Err(GenerateError::AnyhowError(anyhow!("Failed to load Venice credentials: {:?}", err,))),
  }
}

struct PreparedVeniceRequest {
  tauri_model: TauriImageModel,
  venice_request: VeniceGenerateImageRequest,
}

fn prepare_venice_request(request: &TauriGenerateImageRequest) -> Result<PreparedVeniceRequest, GenerateError> {
  let tauri_model = request.model.ok_or_else(GenerateError::no_model_specified)?;

  let spec = venice_model_spec(tauri_model).ok_or_else(|| GenerateError::BadInput(BadInputReason::WrongImageArguments(format!("Model {:?} cannot use a Venice account.", tauri_model,))))?;

  reject_unsupported_inputs(request)?;

  let prompt = request.prompt.clone().unwrap_or_default();
  if prompt.trim().is_empty() {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments("A prompt is required for Venice image generation.".to_string())));
  }

  let mut venice_request = VeniceGenerateImageRequest::new(spec.venice_model, prompt);
  venice_request.aspect_ratio = request.aspect_ratio.map(map_aspect_ratio).map(str::to_owned);

  if spec.sizing == VeniceSizingMode::AspectRatioAndResolution {
    venice_request.resolution = request.resolution.and_then(map_resolution).map(str::to_owned);
  }

  if spec.supports_quality {
    venice_request.quality = request.quality.and_then(map_quality).map(str::to_owned);
  }

  venice_request.variants = map_variants(request.batch_size)?;

  Ok(PreparedVeniceRequest { tauri_model, venice_request })
}

/// Venice's text-to-image endpoint has no reference-image inputs (style
/// references are model-specific), so reject edit-style requests up front.
fn reject_unsupported_inputs(request: &TauriGenerateImageRequest) -> Result<(), GenerateError> {
  let has_unsupported_inputs = request.canvas_image_media_token.is_some() || request.canvas_image_raw_bytes.is_some() || request.scene_image_media_token.is_some() || request.scene_image_raw_bytes.is_some() || request.inpainting_mask_image_media_token.is_some() || request.inpainting_mask_image_raw_bytes.is_some() || request.image_media_tokens.as_ref().is_some_and(|tokens| !tokens.is_empty()) || request.adjust_horizontal_angle.is_some() || request.adjust_vertical_angle.is_some() || request.adjust_zoom.is_some();

  if has_unsupported_inputs {
    return Err(GenerateError::BadInput(BadInputReason::WrongImageArguments("The Venice provider supports text-to-image only; remove the reference images and try again.".to_string())));
  }

  Ok(())
}

fn map_variants(maybe_batch_size: Option<u32>) -> Result<Option<u32>, GenerateError> {
  match maybe_batch_size {
    None | Some(0) => Ok(None),
    Some(n) if n <= VENICE_MAX_VARIANTS => Ok(Some(n)),
    Some(n) => Err(GenerateError::BadInput(BadInputReason::InvalidNumberOfRequestedImages { requested: n, min: 1, max: VENICE_MAX_VARIANTS })),
  }
}

fn build_task_record(task_id: tokens::tokens::sqlite::tasks::TaskId, success: &TaskEnqueueSuccess, maybe_prompt_token: Option<&PromptToken>, request: &TauriGenerateImageRequest) -> Task {
  Task { id: task_id, status: TaskStatus::Pending, task_type: success.task_type, model_type: generation_model_to_task_model_type(&success.model), provider: success.provider, provider_job_id: success.provider_job_id.clone(), queue_status_url: success.maybe_queue_status_url.clone(), queue_response_url: success.maybe_queue_response_url.clone(), prompt_token: maybe_prompt_token.map(|token| token.as_str().to_string()), frontend_caller: request.frontend_caller, frontend_subscriber_id: request.frontend_subscriber_id.clone(), frontend_subscriber_payload: request.frontend_subscriber_payload.clone() }
}

/// Create a prompt record in the Artcraft backend before generating, so the
/// result is associated with its prompt in the user's library.
/// Fails open: prompt tracking must never block generation.
async fn create_prompt_record(request: &TauriGenerateImageRequest, tauri_model: TauriImageModel, app_env_configs: &AppEnvConfigs, storyteller_creds_manager: &StorytellerCredentialManager) -> Option<PromptToken> {
  let creds = match storyteller_creds_manager.get_credentials() {
    Ok(Some(creds)) => creds,
    _ => {
      warn!("[Venice] No Storyteller credentials available, skipping prompt creation");
      return None;
    },
  };

  let prompt_request = CreatePromptRequest { uuid_idempotency_token: generate_random_uuid(), positive_prompt: request.prompt.clone(), negative_prompt: None, model_type: Some(tauri_image_model_to_common_model_type(tauri_model)), generation_provider: Some(GenerationProvider::Venice), maybe_generation_mode: Some(CommonGenerationMode::Text), maybe_aspect_ratio: request.aspect_ratio, maybe_resolution: request.resolution, maybe_batch_count: request.batch_size.map(|n| n.min(255) as u8), maybe_generate_audio: None, maybe_duration_seconds: None };

  match create_prompt(&app_env_configs.storyteller_host, Some(&creds), prompt_request).await {
    Ok(response) => {
      info!("[Venice] Created prompt: {:?}", response.prompt_token);
      Some(response.prompt_token)
    },
    Err(err) => {
      error!("[Venice] Failed to create prompt (continuing anyway): {:?}", err);
      None
    },
  }
}

/// Decode the base64 images into temporary webp files (the request format).
fn save_images_to_temp_files(images: &[String], app_data_root: &AppDataRoot) -> Result<Vec<std::path::PathBuf>, GenerateError> {
  let temp_dir = app_data_root.temp_dir().path();
  let mut paths = Vec::with_capacity(images.len());

  for (index, image_base64) in images.iter().enumerate() {
    let bytes = decode_base64_image(image_base64)?;
    let path = temp_dir.join(format!("venice_{}_{}.webp", generate_random_uuid(), index));
    std::fs::write(&path, &bytes).map_err(GenerateError::IoError)?;
    info!("[Venice] Saved generated image to {:?} ({} bytes)", path, bytes.len());
    paths.push(path);
  }

  Ok(paths)
}
