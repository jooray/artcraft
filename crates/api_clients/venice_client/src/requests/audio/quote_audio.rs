use serde::{Deserialize, Serialize};

/// Request body for `POST /audio/quote`.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceQuoteAudioRequest {
  pub model: String,

  /// Optional duration hint in seconds (models that price per second).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub duration_seconds: Option<u32>,
}

/// Response body for `POST /audio/quote`.
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceQuoteAudioResponse {
  /// Estimated price in USD.
  pub quote: f64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_the_quote() {
    let response: VeniceQuoteAudioResponse = serde_json::from_str(r#"{"quote":0.19}"#).unwrap();
    assert_eq!(response.quote, 0.19);
  }
}
