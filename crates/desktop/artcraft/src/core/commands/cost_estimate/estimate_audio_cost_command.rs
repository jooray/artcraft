use crate::core::commands::generate::generate_audio::venice_audio_models::venice_supports_audio_model_id;
use crate::core::commands::generate::omni::{self, Modality, OmniRequest, OmniResult, OmniResponse};
use crate::core::commands::generate::omni::dispatch::local_error;
use crate::core::commands::response::failure_response_wrapper::{CommandErrorResponseWrapper, CommandErrorStatus};
use enums::common::generation_provider::GenerationProvider;
use tauri::AppHandle;

#[tauri::command]
pub async fn estimate_audio_cost_command(request: OmniRequest, app: AppHandle) -> OmniResult {
  if request.uses_artcraft() {
    return omni::estimate(request, Modality::Audio, &app).await;
  }

  if matches!(request.provider, Some(GenerationProvider::Venice)) {
    if !request.model().is_some_and(venice_supports_audio_model_id) {
      return Err(CommandErrorResponseWrapper {
        status: CommandErrorStatus::BadRequest,
        error_message: Some("This model cannot use a Venice account".to_string()),
        error_type: Some("invalid_provider_for_model".to_string()),
        error_details: None,
      });
    }
    // Paid for with the user's own Venice.ai account, with no ArtCraft credit
    // charge. Do not send this estimate to ArtCraft's generation backend.
    return Ok(OmniResponse(serde_json::json!({
      "success": true, "cost_in_credits": 0, "cost_in_usd_cents": null,
      "is_free": false, "is_unlimited": false, "is_rate_limited": true,
      "has_watermark": false,
    })).into());
  }

  Err(local_error("No direct provider adapter is configured for this modality."))
}
