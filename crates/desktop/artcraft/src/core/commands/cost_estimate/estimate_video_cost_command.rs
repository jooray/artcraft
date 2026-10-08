use crate::core::commands::generate::omni::{self, Modality, OmniRequest, OmniResponse, OmniResult};
use crate::core::commands::generate::omni::dispatch::{adapt_legacy_response, decode_native};
use crate::core::commands::generate::generate_video::venice::venice_video_model::venice_supports_video_model_id;
use tauri::{AppHandle, Manager};
use crate::core::commands::response::failure_response_wrapper::{CommandErrorResponseWrapper, CommandErrorStatus};
use enums::common::generation_provider::GenerationProvider;
use crate::core::commands::response::shorthand::ResponseOrError;
use crate::core::commands::response::success_response_wrapper::SerializeMarker;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::services::storyteller::state::storyteller_credential_manager::StorytellerCredentialManager;
use artcraft_api_defs::generate::cost_estimate::estimate_video_cost::{
  EstimateVideoCostError, EstimateVideoCostErrorType, EstimateVideoCostRequest,
  EstimateVideoCostResponse,
};
use artcraft_client::endpoints::generate::cost_estimate::video::estimate_video_cost::estimate_video_cost;
use log::{debug, info};
use tauri::State;

impl SerializeMarker for EstimateVideoCostResponse {}

#[tauri::command]
pub async fn estimate_video_cost_command(request: OmniRequest, app: AppHandle) -> OmniResult {
  if request.uses_artcraft() {
    return omni::estimate(request, Modality::Video, &app).await;
  }
  if matches!(request.provider, Some(GenerationProvider::Venice)) {
    if !request.model().is_some_and(venice_supports_video_model_id) {
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
