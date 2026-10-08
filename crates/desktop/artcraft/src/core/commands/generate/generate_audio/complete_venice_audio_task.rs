use std::path::PathBuf;

use anyhow::anyhow;
use artcraft_client::endpoints::media_files::get_media_file::get_media_file;
use artcraft_client::endpoints::media_files::legacy_upload_media_file_from_file::{legacy_upload_media_file_from_file, LegacyUploadMediaFileFromFileArgs};
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
use crate::core::events::generation_events::common::{GenerationAction, GenerationServiceProvider};
use crate::core::events::generation_events::generation_complete_event::GenerationCompleteEvent;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::app_preferences::app_preferences_manager::AppPreferencesManager;
use crate::core::state::data_dir::app_data_root::AppDataRoot;
use crate::core::state::task_database::TaskDatabase;
use crate::core::utils::download_url_to_user_download_dir::{save_local_file_with_filename, suggested_download_filename};
use crate::core::utils::local_asset_url::local_asset_url_for_path;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;

/// Finish a generated Venice audio file: upload to the ArtCraft library
/// (logged in) or save into the local download folder (no account).
pub async fn complete_venice_audio_task(app_handle: &AppHandle, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, task: &Task, audio_file_path: &PathBuf, use_library_upload: bool) -> Result<(), GenerateError> {
  info!("[VeniceAudioComplete] Handling task {} (library upload: {})", task.id.as_str(), use_library_upload);

  if use_library_upload {
    complete_audio_via_library(app_handle, task_database, storyteller_creds_manager, task, audio_file_path).await
  } else {
    complete_audio_locally(app_handle, task_database, task, audio_file_path).await
  }
}

// ── Library upload (logged in) ──

async fn complete_audio_via_library(app_handle: &AppHandle, task_database: &TaskDatabase, storyteller_creds_manager: &StorytellerCredentialManager, task: &Task, audio_file_path: &PathBuf) -> Result<(), GenerateError> {
  let app_env_configs = app_handle.state::<AppEnvConfigs>();

  let creds = storyteller_creds_manager.get_credentials().map_err(|err| GenerateError::AnyhowError(anyhow!("Failed to load Storyteller credentials: {:?}", err)))?.ok_or_else(GenerateError::needs_storyteller_credentials)?;

  let upload = legacy_upload_media_file_from_file(LegacyUploadMediaFileFromFileArgs { api_host: &app_env_configs.storyteller_host, maybe_creds: Some(&creds), path: audio_file_path, maybe_generation_provider: Some(GenerationProvider::Venice) }).await?;

  let primary_media_file_token = upload.media_file_token;
  info!("[VeniceAudioComplete] Uploaded as media file: {:?}", primary_media_file_token);

  // Look up the CDN URL for the uploaded file (failing open).
  let mut maybe_cdn_url_str = None;
  match get_media_file(&app_env_configs.storyteller_host, &primary_media_file_token).await {
    Ok(response) => {
      maybe_cdn_url_str = Some(response.media_file.media_links.cdn_url.to_string());
    },
    Err(err) => {
      error!("[VeniceAudioComplete] Failed to look up the media file after upload (failing open): {:?}", err);
    },
  }

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs { db: task_database.get_connection(), task_id: &task.id, maybe_batch_token: None, maybe_primary_media_file_token: Some(&primary_media_file_token), maybe_primary_media_file_class: Some(TaskMediaFileClass::Audio), maybe_primary_media_file_cdn_url: maybe_cdn_url_str.as_deref(), maybe_primary_media_file_thumbnail_url_template: None }).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not mark the Venice task complete: {:?}", err)))?;

  if updated {
    GenerationCompleteEvent { action: Some(GenerationAction::GenerateAudio), service: GenerationServiceProvider::Venice, model: None }.send_infallible(app_handle);
  } else {
    warn!("[VeniceAudioComplete] Task {} was not updated", task.id.as_str());
  }

  Ok(())
}

// ── Local completion (no account) ──

async fn complete_audio_locally(app_handle: &AppHandle, task_database: &TaskDatabase, task: &Task, audio_file_path: &PathBuf) -> Result<(), GenerateError> {
  let app_data_root = app_handle.state::<AppDataRoot>();
  let prefs = app_handle.state::<AppPreferencesManager>().get_clone().map_err(|err| GenerateError::AnyhowError(anyhow!("Could not load app preferences: {:?}", err)))?;
  let directory = prefs.preferred_download_directory.download_directory(&app_data_root);

  let source_url = Url::from_file_path(audio_file_path).map_err(|_| GenerateError::AnyhowError(anyhow!("Cannot build a file URL for {:?}", audio_file_path)))?;
  let filename = suggested_download_filename(&source_url, &prefs, task.model_type.as_ref().map(|model| model.to_str()), None, Local::now());
  let saved_path = save_local_file_with_filename(audio_file_path, &directory, &filename)?;

  let asset_url = local_asset_url_for_path(&saved_path).ok_or_else(|| GenerateError::AnyhowError(anyhow!("Cannot build an asset URL for {:?}", saved_path)))?;
  let cdn_url = Url::parse(&asset_url).map_err(|err| GenerateError::AnyhowError(anyhow!("Bad asset URL {:?}: {:?}", asset_url, err)))?;
  let media_token = MediaFileToken::generate();

  info!("[VeniceAudioComplete] Saved generation to {:?}", saved_path);

  let updated = update_successful_task_status_with_metadata(UpdateSuccessfulTaskArgs { db: task_database.get_connection(), task_id: &task.id, maybe_batch_token: None, maybe_primary_media_file_token: Some(&media_token), maybe_primary_media_file_class: Some(TaskMediaFileClass::Audio), maybe_primary_media_file_cdn_url: Some(cdn_url.as_str()), maybe_primary_media_file_thumbnail_url_template: None }).await.map_err(|err| GenerateError::AnyhowError(anyhow!("Could not mark the Venice task complete: {:?}", err)))?;

  if updated {
    GenerationCompleteEvent { action: Some(GenerationAction::GenerateAudio), service: GenerationServiceProvider::Venice, model: None }.send_infallible(app_handle);
  } else {
    warn!("[VeniceAudioComplete] Task {} was not updated", task.id.as_str());
  }

  Ok(())
}
