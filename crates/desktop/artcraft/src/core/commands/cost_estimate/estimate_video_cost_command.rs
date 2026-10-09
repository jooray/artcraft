use crate::core::commands::generate::omni::{self, Modality, OmniRequest, OmniResponse, OmniResult};
use crate::core::commands::generate::omni::dispatch::{adapt_legacy_response, decode_native};
use crate::core::commands::generate::generate_image::providers::venice::venice_pricing::{estimate_video_cost_usd, estimate_video_cost_usd_for_model, usd_to_cents, venice_estimate_client, zero_credits_json, VideoEstimateVariant};
use crate::core::commands::generate::generate_video::request::TauriVideoModel;
use crate::core::commands::generate::generate_video::venice::handle_venice_video::swap_video_model_variant;
use crate::core::commands::generate::generate_video::venice::venice_video_model::venice_video_model_spec;
use artcraft_router::api::router_aspect_ratio::RouterAspectRatio;
use artcraft_router::api::router_resolution::RouterResolution;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use crate::core::commands::response::failure_response_wrapper::{CommandErrorResponseWrapper, CommandErrorStatus};
use enums::common::generation_provider::GenerationProvider;
use crate::core::commands::response::shorthand::ResponseOrError;
use crate::core::commands::response::success_response_wrapper::SerializeMarker;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use artcraft_api_defs::generate::cost_estimate::estimate_video_cost::{
  EstimateVideoCostError, EstimateVideoCostErrorType, EstimateVideoCostRequest,
  EstimateVideoCostResponse,
};
use artcraft_client::endpoints::generate::cost_estimate::video::estimate_video_cost::estimate_video_cost;
use log::debug;
use tauri::State;

impl SerializeMarker for EstimateVideoCostResponse {}

#[tauri::command]
pub async fn estimate_video_cost_command(request: OmniRequest, app: AppHandle) -> OmniResult {
  if request.uses_artcraft() {
    return omni::estimate(request, Modality::Video, &app).await;
  }
  if matches!(request.provider, Some(GenerationProvider::Venice)) {
    let model_id = request.model().unwrap_or_default().to_string();
    // No ArtCraft credits are used; report the estimated USD cost on the
    // user's Venice account (POST /video/quote). Both compiled models and
    // runtime catalog ids quote through their Venice model id.
    let estimate = venice_video_estimate(&request, &app, &model_id).await;
    return Ok(OmniResponse(estimate).into());
  }
  adapt_legacy_response(estimate_video_cost_native(decode_native(request)?, app.state()).await)
}

async fn estimate_video_cost_native(
  request: EstimateVideoCostRequest,
  app_env_configs: State<'_, AppEnvConfigs>,
) -> ResponseOrError<EstimateVideoCostResponse, EstimateVideoCostError> {
  debug!("estimate_video_cost_command called: {:?}", request);

  let result = estimate_video_cost(
    &app_env_configs.storyteller_host,
    None, // Credentials are not required for this endpoint.
    request,
  )
  .await;

  match result {
    Ok(response) => Ok(response.into()),
    Err(err) => Err(CommandErrorResponseWrapper {
      status: CommandErrorStatus::BadRequest,
      error_message: None,
      error_type: None,
      error_details: Some(EstimateVideoCostError {
        success: false,
        error_type: EstimateVideoCostErrorType::InvalidInput,
        error_message: err.to_string(),
      }),
    }),
  }
}

// ── Venice estimates ──

async fn venice_video_estimate(request: &OmniRequest, app: &AppHandle, model_id: &str) -> Value {
  let Some(client) = venice_estimate_client(app) else {
    return zero_credits_json(None);
  };

  let Some(duration_seconds) = request.fields.get("duration_seconds").and_then(Value::as_u64) else {
    return zero_credits_json(None);
  };

  let resolution = parse_request_field::<RouterResolution>(request, "resolution");
  let aspect_ratio = parse_request_field::<RouterAspectRatio>(request, "aspect_ratio");

  let has_references = field_has_content(request.fields.get("reference_image_media_tokens"));
  let has_start_frame = field_has_content(request.fields.get("start_frame_image_media_token"));
  let has_end_frame = field_has_content(request.fields.get("end_frame_image_media_token"));

  let tauri_model: TauriVideoModel = serde_json::from_value(Value::String(model_id.to_string())).unwrap_or(TauriVideoModel::Unknown);

  // Compiled specs quote their variant's Venice id. Runtime catalog ids are
  // quoted as-is, switching variants by id suffix the same way generation does.
  let maybe_usd = match venice_video_model_spec(tauri_model) {
    Some(spec) => {
      let variant = if has_references {
        VideoEstimateVariant::ReferenceToVideo
      } else if has_start_frame {
        if has_end_frame && spec.first_last_frame.is_some() {
          VideoEstimateVariant::FirstLastFrame
        } else {
          VideoEstimateVariant::ImageToVideo
        }
      } else {
        VideoEstimateVariant::TextToVideo
      };

      estimate_video_cost_usd(&client, &spec, variant, duration_seconds as u32, resolution, aspect_ratio).await
    },
    None => {
      let quoted_model = if has_references {
        swap_video_model_variant(model_id, "-text-to-video", "-reference-to-video")
      } else if has_start_frame {
        swap_video_model_variant(model_id, "-text-to-video", "-image-to-video")
      } else {
        None
      };
      let quoted_model = quoted_model.as_deref().unwrap_or(model_id);

      estimate_video_cost_usd_for_model(&client, quoted_model, duration_seconds as u32, resolution, aspect_ratio).await
    },
  };

  zero_credits_json(maybe_usd.map(usd_to_cents))
}

fn parse_request_field<T: DeserializeOwned>(request: &OmniRequest, field: &str) -> Option<T> {
  request.fields.get(field).and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn field_has_content(value: Option<&Value>) -> bool {
  match value {
    Some(Value::Array(items)) => !items.is_empty(),
    Some(Value::String(text)) => !text.is_empty(),
    Some(Value::Null) | None => false,
    Some(_) => true,
  }
}
