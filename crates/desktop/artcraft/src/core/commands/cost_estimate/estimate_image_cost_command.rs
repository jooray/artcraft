use enums::common::generation::common_quality::CommonQuality;
use enums::common::generation::common_resolution::CommonResolution;
use enums::common::generation_provider::GenerationProvider;
use log::warn;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use venice_client::client::venice_client::VeniceClient;
use venice_client::creds::venice_api_key::VeniceApiKey;

use crate::core::commands::generate::generate_image::providers::venice::venice_image_model::{map_quality, map_resolution, venice_model_spec, venice_supports_model_id};
use crate::core::commands::generate::generate_image::providers::venice::venice_pricing::{estimate_image_cost_usd, usd_to_cents, venice_estimate_client, zero_credits_json};
use crate::core::commands::generate::generate_image::tauri_image_model::TauriImageModel;
use crate::core::commands::generate::omni::OmniResponse;
use crate::core::commands::generate::omni::{self, Modality, OmniRequest, OmniResult};
use crate::core::commands::generate::omni::dispatch::{adapt_legacy_response, decode_native};
use crate::core::commands::response::failure_response_wrapper::{CommandErrorResponseWrapper, CommandErrorStatus};
use crate::core::commands::response::shorthand::ResponseOrError;
use crate::core::commands::response::success_response_wrapper::SerializeMarker;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use artcraft_api_defs::generate::cost_estimate::estimate_image_cost::{
  EstimateImageCostError, EstimateImageCostErrorType, EstimateImageCostRequest,
  EstimateImageCostResponse,
};
use artcraft_client::endpoints::generate::cost_estimate::image::estimate_image_cost::estimate_image_cost;
use log::{debug, info};
use tauri::State;

impl SerializeMarker for EstimateImageCostResponse {}

#[tauri::command]
pub async fn estimate_image_cost_command(request: OmniRequest, app: AppHandle) -> OmniResult {
  if matches!(request.provider, Some(GenerationProvider::Midjourney)) {
    if !matches!(request.model(), Some("midjourney" | "midjourney_7" | "midjourney_7_niji" | "midjourney_8")) {
      return Err(CommandErrorResponseWrapper {
        status: CommandErrorStatus::BadRequest,
        error_message: Some("This model cannot use a Midjourney account".to_string()),
        error_type: Some("invalid_provider_for_model".to_string()),
        error_details: None,
      });
    }
    // Paid for by the user's Midjourney subscription, with no ArtCraft credit
    // charge. Do not send this estimate to ArtCraft's generation backend.
    return Ok(OmniResponse(zero_credits_json(None)).into());
  }
  if request.uses_artcraft() && !request.uses_legacy_image_endpoint() {
    return omni::estimate(request, Modality::Image, &app).await;
  }
  if matches!(request.provider, Some(GenerationProvider::Venice)) {
    let model_id = request.model().unwrap_or_default().to_string();
    if !venice_supports_model_id(&model_id) {
      return Err(CommandErrorResponseWrapper {
        status: CommandErrorStatus::BadRequest,
        error_message: Some("This model cannot use a Venice account".to_string()),
        error_type: Some("invalid_provider_for_model".to_string()),
        error_details: None,
      });
    }
    // No ArtCraft credits are used; report the estimated USD cost on the
    // user's Venice account (from the model's Venice pricing metadata).
    let estimate = venice_image_estimate(&request, &app, &model_id).await;
    return Ok(OmniResponse(estimate).into());
  }
  adapt_legacy_response(estimate_image_cost_native(decode_native(request)?, app.state()).await)
}

async fn estimate_image_cost_native(
  request: EstimateImageCostRequest,
  app_env_configs: State<'_, AppEnvConfigs>,
) -> ResponseOrError<EstimateImageCostResponse, EstimateImageCostError> {
  debug!("estimate_image_cost_command called: {:?}", request);

  let result = estimate_image_cost(
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
      error_details: Some(EstimateImageCostError {
        success: false,
        error_type: EstimateImageCostErrorType::InvalidInput,
        error_message: err.to_string(),
      }),
    }),
  }
}

// ── Venice estimates ──

async fn venice_image_estimate(request: &OmniRequest, app: &AppHandle, model_id: &str) -> Value {
  let Some(client) = venice_estimate_client(app) else {
    return zero_credits_json(None);
  };

  let Ok(tauri_model) = serde_json::from_value::<TauriImageModel>(Value::String(model_id.to_string())) else {
    return zero_credits_json(None);
  };
  let Some(spec) = venice_model_spec(tauri_model) else {
    return zero_credits_json(None);
  };

  // Requests with image inputs are edits and use the edit model's pricing.
  let is_edit = ["image_media_tokens", "canvas_image_media_token", "scene_image_media_token", "canvas_image_raw_bytes", "scene_image_raw_bytes"]
    .iter()
    .any(|field| field_has_any_content(request.fields.get(*field)));

  let venice_model = if is_edit { spec.edit_model.unwrap_or(spec.venice_model) } else { spec.venice_model };

  let resolution_tier = request.fields.get("resolution").and_then(Value::as_str).and_then(|value| CommonResolution::from_str(value).ok()).and_then(map_resolution);
  let quality = request.fields.get("quality").and_then(Value::as_str).and_then(|value| CommonQuality::from_str(value).ok()).and_then(map_quality);

  // Venice charges per request; variants do not multiply the price.
  let maybe_usd = estimate_image_cost_usd(&client, venice_model, resolution_tier, quality).await;

  zero_credits_json(maybe_usd.map(usd_to_cents))
}

fn field_has_any_content(value: Option<&Value>) -> bool {
  match value {
    Some(Value::Array(items)) => !items.is_empty(),
    Some(Value::String(text)) => !text.is_empty(),
    Some(Value::Null) | None => false,
    Some(_) => true,
  }
}
