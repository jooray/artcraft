//! `venice_catalog_command`: the RUNTIME Venice.ai model catalog.
//!
//! The desktop app ships a compiled set of Venice models (see the Venice
//! provider specs). Venice adds models independently of app releases, so this
//! command fetches the live catalog and returns only the models this build
//! does NOT know about. The frontend merges them into the model pickers as
//! Venice-provider entries; generation carries the raw id in `model_id`.

use log::{info, warn};
use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;

use crate::core::commands::generate::generate_image::providers::venice::venice_image_model::VENICE_STATIC_IMAGE_MODEL_IDS;
use crate::core::commands::generate::generate_image::providers::venice::venice_pricing::venice_estimate_client;
use crate::core::commands::generate::generate_video::venice::venice_video_model::VENICE_STATIC_VIDEO_MODEL_IDS;
use crate::core::commands::response::shorthand::ResponseOrErrorMessage;
use crate::core::commands::response::success_response_wrapper::SerializeMarker;

impl SerializeMarker for VeniceCatalogResponse {}

#[derive(Serialize)]
pub struct VeniceCatalogResponse {
  pub success: bool,
  pub image_models: Vec<VeniceCatalogImageModel>,
  pub video_models: Vec<VeniceCatalogVideoModel>,
}

#[derive(Serialize)]
pub struct VeniceCatalogImageModel {
  pub id: String,
  pub name: String,
  pub aspect_ratios: Vec<String>,
  pub resolutions: Vec<String>,
  pub maybe_prompt_limit: Option<u64>,
  pub maybe_width_height_divisor: Option<u64>,
}

#[derive(Serialize)]
pub struct VeniceCatalogVideoModel {
  pub id: String,
  pub name: String,
  pub model_type: String,
  pub aspect_ratios: Vec<String>,
  pub resolutions: Vec<String>,
  pub durations: Vec<String>,
  pub audio: bool,
}

#[tauri::command]
pub async fn venice_catalog_command(app: AppHandle) -> ResponseOrErrorMessage<VeniceCatalogResponse> {
  let Some(client) = venice_estimate_client(&app) else {
    // No key configured: an empty catalog is fine.
    return Ok(empty_catalog().into());
  };

  let (image_result, video_result) = tokio::join!(client.list_models("image"), client.list_models("video"));

  let image_models = match image_result {
    Ok(response) => response.data.into_iter().filter_map(|model| {
      if VENICE_STATIC_IMAGE_MODEL_IDS.contains(&model.id.as_str()) || model.id.contains("-edit") {
        return None;
      }
      let spec = model.model_spec?;
      if spec.offline == Some(true) {
        return None;
      }

      let constraints = spec.constraints.clone().unwrap_or(Value::Null);
      Some(VeniceCatalogImageModel {
        name: spec.name.clone().unwrap_or_else(|| model.id.clone()),
        aspect_ratios: string_array(&constraints, "aspectRatios"),
        resolutions: string_array(&constraints, "resolutions"),
        maybe_prompt_limit: constraints.get("promptCharacterLimit").and_then(Value::as_u64),
        maybe_width_height_divisor: constraints.get("widthHeightDivisor").and_then(Value::as_u64),
        id: model.id,
      })
    }).collect::<Vec<_>>(),
    Err(err) => {
      warn!("[VeniceCatalog] Could not fetch Venice image models: {:?}", err);
      Vec::new()
    },
  };

  let video_models = match video_result {
    Ok(response) => response.data.into_iter().filter_map(|model| {
      if VENICE_STATIC_VIDEO_MODEL_IDS.contains(&model.id.as_str()) {
        return None;
      }
      let spec = model.model_spec?;
      if spec.offline == Some(true) {
        return None;
      }

      let constraints = spec.constraints.clone().unwrap_or(Value::Null);

      // Video-to-video workflows (upscale, motion control, recast) need a
      // source video the app cannot attach yet.
      if constraints.get("video_input").and_then(Value::as_bool) == Some(true) {
        return None;
      }

      Some(VeniceCatalogVideoModel {
        name: spec.name.clone().unwrap_or_else(|| model.id.clone()),
        model_type: constraints.get("model_type").and_then(Value::as_str).unwrap_or("text-to-video").to_string(),
        aspect_ratios: string_array(&constraints, "aspect_ratios"),
        resolutions: string_array(&constraints, "resolutions"),
        durations: string_array(&constraints, "durations"),
        audio: constraints.get("audio").and_then(Value::as_bool).unwrap_or(false),
        id: model.id,
      })
    }).collect::<Vec<_>>(),
    Err(err) => {
      warn!("[VeniceCatalog] Could not fetch Venice video models: {:?}", err);
      Vec::new()
    },
  };

  info!("[VeniceCatalog] {} new image model(s), {} new video model(s) beyond this build", image_models.len(), video_models.len());

  Ok(VeniceCatalogResponse { success: true, image_models, video_models }.into())
}

// ── Helpers ──

fn empty_catalog() -> VeniceCatalogResponse {
  VeniceCatalogResponse { success: true, image_models: Vec::new(), video_models: Vec::new() }
}

fn string_array(constraints: &Value, key: &str) -> Vec<String> {
  constraints.get(key)
    .and_then(Value::as_array)
    .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect())
    .unwrap_or_default()
}
