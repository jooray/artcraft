use crate::core::commands::enqueue::common::notify_frontend_of_errors::notify_frontend_of_errors;
use crate::core::commands::enqueue::generate_error::{GenerateError, MissingCredentialsReason};
use crate::core::commands::generate::generate_audio::handle_venice_audio::handle_venice_audio_enqueue;
use crate::core::commands::generate::omni::{self, Modality, OmniRequest, OmniResult};
use crate::core::commands::generate::omni::dispatch::local_error;
use crate::core::commands::response::failure_response_wrapper::{CommandErrorResponseWrapper, CommandErrorStatus};
use crate::core::events::basic_sendable_event_trait::BasicSendableEvent;
use crate::core::events::functional_events::show_provider_login_modal_event::ShowProviderLoginModalEvent;
use enums::common::generation_provider::GenerationProvider;
use log::error;
use tauri::AppHandle;

#[tauri::command]
pub async fn generate_audio_command(request: OmniRequest, app: AppHandle) -> OmniResult {
  if request.uses_artcraft() {
    return omni::generate(request, Modality::Audio, &app).await;
  }

  // Venice.ai BYOK music / sound effects: queue locally, poll in the background.
  if matches!(request.provider, Some(GenerationProvider::Venice)) {
    return match handle_venice_audio_enqueue(&request, &app).await {
      Ok(response) => Ok(response.into()),
      Err(err) => Err(handle_audio_error(&app, err).await),
    };
  }

  Err(local_error("No direct provider adapter is configured for this modality."))
}

async fn handle_audio_error(app: &AppHandle, err: GenerateError) -> CommandErrorResponseWrapper<String, ()> {
  error!("generate_audio_command error: {:?}", err);

  notify_frontend_of_errors(app, &err).await;

  if matches!(&err, GenerateError::MissingCredentials(MissingCredentialsReason::NeedsVeniceApiKey)) {
    ShowProviderLoginModalEvent::send_for_provider(GenerationProvider::Venice, app);
  }

  let (status, error_type, error_message) = match &err {
    GenerateError::BadInput(_) => (CommandErrorStatus::BadRequest, "bad_input", format!("{:?}", err)),
    GenerateError::MissingCredentials(MissingCredentialsReason::NeedsVeniceApiKey) => (CommandErrorStatus::Unauthorized, "needs_venice_api_key", "You need to set a Venice.ai API key".to_string()),
    GenerateError::MissingCredentials(MissingCredentialsReason::NeedsStorytellerCredentials) => (CommandErrorStatus::Unauthorized, "needs_storyteller_credentials", "You need to be logged into Artcraft.".to_string()),
    _ => (CommandErrorStatus::ServerError, "server_error", format!("{:?}", err)),
  };

  CommandErrorResponseWrapper { status, error_message: Some(error_message), error_type: Some(error_type.to_string()), error_details: None }
}
