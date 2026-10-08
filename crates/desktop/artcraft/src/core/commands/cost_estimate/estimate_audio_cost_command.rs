use crate::core::commands::generate::generate_audio::venice_audio_models::venice_supports_audio_model_id;
use crate::core::commands::generate::generate_image::providers::venice::venice_pricing::{estimate_audio_cost_usd, usd_to_cents, venice_estimate_client, zero_credits_json};
use crate::core::commands::generate::omni::{self, Modality, OmniRequest, OmniResult, OmniResponse};
use crate::core::commands::generate::omni::dispatch::local_error;
use crate::core::commands::response::failure_response_wrapper::{CommandErrorResponseWrapper, CommandErrorStatus};
use enums::common::generation_provider::GenerationProvider;
use serde_json::Value;
use tauri::AppHandle;

#[tauri::command]
pub async fn estimate_audio_cost_command(request: OmniRequest, app: AppHandle) -> OmniResult {
  if request.uses_artcraft() {
    return omni::estimate(request, Modality::Audio, &app).await;
  }

  if matches!(request.provider, Some(GenerationProvider::Venice)) {
    let model_id = request.model().unwrap_or_default().to_string();
    if !venice_supports_audio_model_id(&model_id) {
      return Err(CommandErrorResponseWrapper {
        status: CommandErrorStatus::BadRequest,
        error_message: Some("This model cannot use a Venice account".to_string()),
        error_type: Some("invalid_provider_for_model".to_string()),
        error_details: None,
      });
    }
    // No ArtCraft credits are used; report the estimated USD cost on the
    // user's Venice account (POST /audio/quote).
    let estimate = venice_audio_estimate(&request, &app, &model_id).await;
    return Ok(OmniResponse(estimate).into());
  }

  Err(local_error("No direct provider adapter is configured for this modality."))
}

async fn venice_audio_estimate(request: &OmniRequest, app: &AppHandle, model_id: &str) -> Value {
  let Some(client) = venice_estimate_client(app) else {
    return zero_credits_json(None);
  };

  let duration_seconds = request.fields.get("duration_seconds").and_then(Value::as_u64).map(|value| value as u32);

  let maybe_usd = estimate_audio_cost_usd(&client, model_id, duration_seconds).await;

  zero_credits_json(maybe_usd.map(usd_to_cents))
}
