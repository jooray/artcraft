use serde::Serialize;

/// Request body for `POST /video/retrieve`.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceRetrieveVideoRequest {
  pub model: String,
  pub queue_id: String,
}
