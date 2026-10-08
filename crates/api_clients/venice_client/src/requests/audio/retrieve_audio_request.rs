use serde::Serialize;

/// Request body for `POST /audio/retrieve`.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceRetrieveAudioRequest {
  pub model: String,
  pub queue_id: String,
}
