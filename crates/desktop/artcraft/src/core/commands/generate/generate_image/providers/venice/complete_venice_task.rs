use std::path::PathBuf;

use anyhow::anyhow;
use artcraft_api_defs::utils::media_links_to_thumbnail_template::media_links_to_thumbnail_template;
use artcraft_client::endpoints::media_files::get_media_file::get_media_file;
use artcraft_client::endpoints::media_files::upload_image_media_file_from_file::{upload_image_media_file_from_file, UploadImageFromFileArgs};
use chrono::Local;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_media_file_class::TaskMediaFileClass;
use enums::tauri::tasks::task_status::TaskStatus;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info, warn};
use sqlite_tasks::queries::task::Task;
use sqlite_tasks::queries::update_successful_task_status_with_metadata::{update_successful_task_status_with_metadata, UpdateSuccessfulTaskArgs};
use sqlite_tasks::queries::update_task_status::{update_task_status, UpdateTaskArgs};
use tauri::{AppHandle, Manager};
use tokens::tokens::batch_generations::BatchGenerationToken;
use tokens::tokens::media_files::MediaFileToken;
use tokens::tokens::prompts::PromptToken;
use url::Url;

use crate::core::commands::enqueue::generate_error::GenerateError;
use crate::core::events::basic_sendable_event_trait::BasicSendableEvent;
use crate::core::events::functional_events::text_to_image_generation_complete_event::{GeneratedImage, TextToImageGenerationCompleteEvent};
use crate::core::events::generation_events::common::{GenerationAction, GenerationServiceProvider};
use crate::core::events::generation_events::generation_complete_event::GenerationCompleteEvent;
use crate::core::events::generation_events::generation_failed_event::GenerationFailedEvent;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::app_preferences::app_preferences_manager::AppPreferencesManager;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::task_database::TaskDatabase;
use crate::core::threads::third_party_task_polling_thread::events::notify_frontend_of_completion::{notify_frontend_of_completion, CompletionData};
use crate::core::utils::auto_download::auto_download_task_local_files;
use crate::core::utils::download_url_to_user_download_dir::{save_local_file_with_filename, suggested_download_filename};
use crate::core::utils::local_asset_url::local_asset_url_for_path;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;

/// Finish an already-generated Venice result.
///
/// Logged in: upload to the user's ArtCraft library (mirrors the Fal completion
/// pipeline). Logged out: save into the local download folder and show the
/// result via the asset protocol — no account required.
pub async fn complete_venice_task(app_handle: &AppHandle, app_env_configs: &AppEnvConfigs, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, task: &Task, maybe_prompt_token: Option<&PromptToken>, media_file_paths: &[PathBuf], use_library_upload: bool) -> Result<(), GenerateError> {
  info!("[VeniceComplete] Handling completed task {} (library upload: {})", task.id.as_str(), use_library_upload);

  if use_library_upload {
    complete_via_library(app_handle, app_env_configs, task_database, storyteller_creds_manager, task, maybe_prompt_token, media_file_paths).await
  } else {
    complete_locally(app_handle, task_database, task, media_file_paths).await
  }
}

/// Mark a failed Venice task in the local database and notify the frontend.
pub async fn mark_venice_task_failed_and_notify(app_handle: &AppHandle, task_database: &TaskDatabase, task: &Task, reason: &str) {
  info!("[VeniceComplete] Marking task {} as failed: {}", task.id.as_str(), reason);

  if let Err(err) = update_task_status(UpdateTaskArgs { db: task_database.get_connection(), task_id: &task.id, status: TaskStatus::CompleteFailure }).await {
    error!("[VeniceComplete] Failed to update task status for {}: {:?}", task.id.as_str(), err);
  }

  let event = GenerationFailedEvent { action: task_type_to_generation_action(task.task_type), service: GenerationServiceProvider::Venice, model: None, reason: Some(reason.to_string()) };

  event.send_infallible(app_handle);
}

// ── Library upload (logged in) ──

async fn complete_via_library(app_handle: &AppHandle, app_env_configs: &AppEnvConfigs, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, task: &Task, maybe_prompt_token: Option<&PromptToken>, media_file_paths: &[PathBuf]) -> Result<(), GenerateError> {
  // Save local copies before uploading (best effort; never blocks completion).
  if let Err(err) = auto_download_task_local_files(app_handle, task, media_file_paths) {
    warn!("[VeniceComplete] Auto-download failed (continuing): {:?}", err);
  }

  let creds = storyteller_creds_manager.get_credentials().map_err(|err| GenerateError::AnyhowError(anyhow!("Failed to load Storyteller credentials: {:?}", err)))?.ok_or_else(GenerateError::needs_storyteller_credentials)?;

  let media_class = TaskMediaFileClass::Image;
  let maybe_batch_token = (media_file_paths.len() > 1).then(BatchGenerationToken::generate);

  let mut maybe_primary_media_file_token: Option<MediaFileToken> = None;

  for path in media_file_paths {
    info!("[VeniceComplete] Uploading result {:?} to the backend...", path);

    let result = upload_image_media_file_from_file(UploadImageFromFileArgs { api_host: &app_env_configs.storyteller_host, maybe_creds: Some(&creds), path, is_intermediate_system_file: false, maybe_prompt_token, maybe_batch_token: maybe_batch_token.as_ref(), maybe_generation_provider: Some(GenerationProvider::Venice) }).await?;

    info!("[VeniceComplete] Uploaded as media file: {:?}", result.media_file_token);

    if maybe_primary_media_file_token.is_none() {
      maybe_primary_media_file_token = Some(result.media_file_token);
    }
  }

  let primary_media_file_token = maybe_primary_media_file_token.ok_or_else(|| GenerateError::AnyhowError(anyhow!("Venice completed without any media files to upload")))?;

  // Look up CDN/thumbnail URLs for the primary media file (failing open).
  let mut maybe_cdn_url = None;
  let mut maybe_cdn_url_str = None;
  let mut maybe_thumbnail_url_template = None;

  match get_media_file(&app_env_configs.storyteller_host, &primary_media_file_token).await {
    Ok(response) => {
      maybe_cdn_url = Some(response.media_file.media_links.cdn_url.clone());
      maybe_cdn_url_str = Some(response.media_file.media_links.cdn_url.to_string());
      maybe_thumbnail_url_template = media_links_to_thumbnail_template(&response.media_file.media_links).map(|s| s.to_string());
    },
    Err(err) => {
      error!("[VeniceComplete] Failed to look up the media file after upload (failing open): {:?}", err);
    },
  }

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs { db: task_database.get_connection(), task_id: &task.id, maybe_batch_token: maybe_batch_token.as_ref(), maybe_primary_media_file_token: Some(&primary_media_file_token), maybe_primary_media_file_class: Some(media_class), maybe_primary_media_file_cdn_url: maybe_cdn_url_str.as_deref(), maybe_primary_media_file_thumbnail_url_template: maybe_thumbnail_url_template.as_deref() }).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not mark the Venice task complete: {:?}", err)))?;

  if updated {
    let completion = CompletionData { primary_media_file_token, maybe_cdn_url, maybe_thumbnail_url_template, maybe_batch_token, media_class };

    notify_frontend_of_completion(app_handle, &app_env_configs.storyteller_host, Some(&creds), task, &completion).await;
  } else {
    warn!("[VeniceComplete] Task {} was not updated", task.id.as_str());
  }

  Ok(())
}

// ── Local completion (no account) ──

/// Copy the generated files into the download folder and show them in the app
/// via the asset protocol. The task record keeps the result, so it survives
/// restarts (the desktop feed/gallery reads the local task database).
async fn complete_locally(app_handle: &AppHandle, task_database: &TaskDatabase, task: &Task, media_file_paths: &[PathBuf]) -> Result<(), GenerateError> {
  let app_data_root = app_handle.state::<AppDataRoot>();
  let prefs = app_handle.state::<AppPreferencesManager>().get_clone().map_err(|err| GenerateError::AnyhowError(anyhow!("Could not load app preferences: {:?}", err)))?;
  let directory = prefs.preferred_download_directory.download_directory(&app_data_root);

  let mut generated_images = Vec::with_capacity(media_file_paths.len());

  for (index, source_path) in media_file_paths.iter().enumerate() {
    let source_url = Url::from_file_path(source_path).map_err(|_| GenerateError::AnyhowError(anyhow!("Cannot build a file URL for {:?}", source_path)))?;
    let filename = suggested_download_filename(&source_url, &prefs, task.model_type.as_ref().map(|model| model.to_str()), (media_file_paths.len() > 1).then_some(index + 1), Local::now());
    let saved_path = save_local_file_with_filename(source_path, &directory, &filename)?;

    let asset_url = local_asset_url_for_path(&saved_path).ok_or_else(|| GenerateError::AnyhowError(anyhow!("Cannot build an asset URL for {:?}", saved_path)))?;

    info!("[VeniceComplete] Saved generation to {:?}", saved_path);

    generated_images.push(GeneratedImage { media_token: MediaFileToken::generate(), cdn_url: Url::parse(&asset_url).map_err(|err| GenerateError::AnyhowError(anyhow!("Bad asset URL {:?}: {:?}", asset_url, err)))?, maybe_thumbnail_template: None });
  }

  let primary = generated_images.first().ok_or_else(|| GenerateError::AnyhowError(anyhow!("Venice completed without any media files")))?;

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs { db: task_database.get_connection(), task_id: &task.id, maybe_batch_token: None, maybe_primary_media_file_token: Some(&primary.media_token), maybe_primary_media_file_class: Some(TaskMediaFileClass::Image), maybe_primary_media_file_cdn_url: Some(primary.cdn_url.as_str()), maybe_primary_media_file_thumbnail_url_template: None }).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not mark the Venice task complete: {:?}", err)))?;

  if updated {
    notify_local_image_completion(app_handle, task, generated_images);
  } else {
    warn!("[VeniceComplete] Task {} was not updated", task.id.as_str());
  }

  Ok(())
}

/// Notify the frontend about locally saved images (no backend involvement).
fn notify_local_image_completion(app_handle: &AppHandle, task: &Task, generated_images: Vec<GeneratedImage>) {
  let complete_event = GenerationCompleteEvent { action: Some(task_type_to_generation_action(task.task_type)), service: GenerationServiceProvider::Venice, model: None };
  complete_event.send_infallible(app_handle);

  let event = TextToImageGenerationCompleteEvent { generated_images, maybe_frontend_subscriber_id: task.frontend_subscriber_id.clone(), maybe_frontend_subscriber_payload: task.frontend_subscriber_payload.clone() };
  event.send_infallible(app_handle);
}

// ── Helpers ──

fn task_type_to_generation_action(task_type: TaskType) -> GenerationAction {
  match task_type {
    TaskType::ImageGeneration => GenerationAction::GenerateImage,
    TaskType::VideoGeneration => GenerationAction::GenerateVideo,
    TaskType::AudioGeneration => GenerationAction::GenerateAudio,
    TaskType::BackgroundRemoval => GenerationAction::RemoveBackground,
    TaskType::ObjectGeneration => GenerationAction::ImageTo3d,
    TaskType::GaussianGeneration => GenerationAction::GenerateGaussian,
    TaskType::ImageInpaintEdit => GenerationAction::ImageInpaintEdit,
  }
}
