/// Error type for Venice.ai API interactions.
#[derive(Debug, thiserror::Error)]
pub enum VeniceError {
  #[error("Venice request failed: {0}")]
  Request(#[from] reqwest::Error),

  /// The API returned a non-2xx status; `message` is the best-effort error text.
  #[error("Venice API error ({status}): {message}")]
  Api { status: reqwest::StatusCode, message: String },

  #[error("Venice response parse error: {0}")]
  Decode(#[from] serde_json::Error),

  #[error("Base64 decode error: {0}")]
  Base64(#[from] base64::DecodeError),

  #[error("Venice IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Venice error: {0}")]
  Other(String),
}
