use std::path::PathBuf;

use anyhow::anyhow;
use artcraft_api_defs::utils::media_links_to_thumbnail_template::media_links_to_thumbnail_template;
use artcraft_client::endpoints::media_files::get_media_file::get_media_file;
use artcraft_client::endpoints::media_files::upload_video_media_file_from_file::{upload_video_media_file_from_file, UploadVideoFromFileArgs};
use chrono::Local;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_media_file_class::TaskMediaFileClass;
use log::{error, info, warn};
use sqlite_tasks::queries::task::Task;
use sqlite_tasks::queries::update_successful_task_status_with_metadata::{update_successful_task_status_with_metadata, UpdateSuccessfulTaskArgs};
use tauri::{AppHandle, Manager};
use tokens::tokens::media_files::MediaFileToken;
use url::Url;

use crate::core::commands::enqueue::generate_error::GenerateError;
use crate::core::events::basic_sendable_event_trait::BasicSendableEvent;
use crate::core::events::functional_events::video_generation_complete_event::{GeneratedVideo, VideoGenerationCompleteEvent};
use crate::core::events::generation_events::common::{GenerationAction, GenerationServiceProvider};
use crate::core::events::generation_events::generation_complete_event::GenerationCompleteEvent;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::app_preferences::app_preferences_manager::AppPreferencesManager;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::task_database::TaskDatabase;
use crate::core::threads::third_party_task_polling_thread::events::notify_frontend_of_completion::{notify_frontend_of_completion, CompletionData};
use crate::core::utils::download_url_to_user_download_dir::{save_local_file_with_filename, suggested_download_filename};
use crate::core::utils::local_asset_url::local_asset_url_for_path;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;

/// Finish a generated Venice video: upload to the ArtCraft library (logged in)
/// or save into the local download folder and show it via the asset protocol.
pub async fn complete_venice_video_task(app_handle: &AppHandle, app_env_configs: &AppEnvConfigs, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, task: &Task, video_file_path: &PathBuf, use_library_upload: bool) -> Result<(), GenerateError> {
  info!("[VeniceVideoComplete] Handling task {} (library upload: {})", task.id.as_str(), use_library_upload);

  if use_library_upload {
    complete_video_via_library(app_handle, app_env_configs, task_database, storyteller_creds_manager, task, video_file_path).await
  } else {
    complete_video_locally(app_handle, task_database, task, video_file_path).await
  }
}

// ── Library upload (logged in) ──

async fn complete_video_via_library(app_handle: &AppHandle, app_env_configs: &AppEnvConfigs, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, task: &Task, video_file_path: &PathBuf) -> Result<(), GenerateError> {
  let creds = storyteller_creds_manager.get_credentials().map_err(|err| GenerateError::AnyhowError(anyhow!("Failed to load Storyteller credentials: {:?}", err)))?.ok_or_else(GenerateError::needs_storyteller_credentials)?;

  let media_class = TaskMediaFileClass::Video;

  let upload = upload_video_media_file_from_file(UploadVideoFromFileArgs { api_host: &app_env_configs.storyteller_host, maybe_creds: Some(&creds), path: video_file_path, maybe_prompt_token: None, maybe_generation_provider: Some(GenerationProvider::Venice) }).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Venice video upload failed: {:?}", err)))?;

  let primary_media_file_token = upload.media_file_token;
  info!("[VeniceVideoComplete] Uploaded as media file: {:?}", primary_media_file_token);

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
      error!("[VeniceVideoComplete] Failed to look up the media file after upload (failing open): {:?}", err);
    },
  }

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs { db: task_database.get_connection(), task_id: &task.id, maybe_batch_token: None, maybe_primary_media_file_token: Some(&primary_media_file_token), maybe_primary_media_file_class: Some(media_class), maybe_primary_media_file_cdn_url: maybe_cdn_url_str.as_deref(), maybe_primary_media_file_thumbnail_url_template: maybe_thumbnail_url_template.as_deref() }).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not mark the Venice task complete: {:?}", err)))?;

  if updated {
    let completion = CompletionData { primary_media_file_token, maybe_cdn_url, maybe_thumbnail_url_template, maybe_batch_token: None, media_class };

    notify_frontend_of_completion(app_handle, &app_env_configs.storyteller_host, Some(&creds), task, &completion).await;
  } else {
    warn!("[VeniceVideoComplete] Task {} was not updated", task.id.as_str());
  }

  Ok(())
}

// ── Local completion (no account) ──

async fn complete_video_locally(app_handle: &AppHandle, task_database: &TaskDatabase, task: &Task, video_file_path: &PathBuf) -> Result<(), GenerateError> {
  let app_data_root = app_handle.state::<AppDataRoot>();
  let prefs = app_handle.state::<AppPreferencesManager>().get_clone().map_err(|err| GenerateError::AnyhowError(anyhow!("Could not load app preferences: {:?}", err)))?;
  let directory = prefs.preferred_download_directory.download_directory(&app_data_root);

  let source_url = Url::from_file_path(video_file_path).map_err(|_| GenerateError::AnyhowError(anyhow!("Cannot build a file URL for {:?}", video_file_path)))?;
  let filename = suggested_download_filename(&source_url, &prefs, task.model_type.as_ref().map(|model| model.to_str()), None, Local::now());
  let saved_path = save_local_file_with_filename(video_file_path, &directory, &filename)?;

  let asset_url = local_asset_url_for_path(&saved_path).ok_or_else(|| GenerateError::AnyhowError(anyhow!("Cannot build an asset URL for {:?}", saved_path)))?;
  let cdn_url = Url::parse(&asset_url).map_err(|err| GenerateError::AnyhowError(anyhow!("Bad asset URL {:?}: {:?}", asset_url, err)))?;
  let media_token = MediaFileToken::generate();

  info!("[VeniceVideoComplete] Saved generation to {:?}", saved_path);

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs { db: task_database.get_connection(), task_id: &task.id, maybe_batch_token: None, maybe_primary_media_file_token: Some(&media_token), maybe_primary_media_file_class: Some(TaskMediaFileClass::Video), maybe_primary_media_file_cdn_url: Some(cdn_url.as_str()), maybe_primary_media_file_thumbnail_url_template: None }).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not mark the Venice task complete: {:?}", err)))?;

  if updated {
    GenerationCompleteEvent { action: Some(GenerationAction::GenerateVideo), service: GenerationServiceProvider::Venice, model: None }.send_infallible(app_handle);

    let event = VideoGenerationCompleteEvent { generated_video: Some(GeneratedVideo { media_token, cdn_url, maybe_thumbnail_template: None }), maybe_frontend_subscriber_id: task.frontend_subscriber_id.clone(), maybe_frontend_subscriber_payload: task.frontend_subscriber_payload.clone() };
    event.send_infallible(app_handle);
  } else {
    warn!("[VeniceVideoComplete] Task {} was not updated", task.id.as_str());
  }

  Ok(())
}
