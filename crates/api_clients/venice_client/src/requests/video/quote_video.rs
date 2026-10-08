use serde::{Deserialize, Serialize};

/// Request body for `POST /video/quote`.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceQuoteVideoRequest {
  pub model: String,

  /// A duration string like `"5s"`.
  pub duration: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolution: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub aspect_ratio: Option<String>,
}

/// Response body for `POST /video/quote`.
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceQuoteVideoResponse {
  /// Estimated price in USD.
  pub quote: f64,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_the_quote() {
    let response: VeniceQuoteVideoResponse = serde_json::from_str(r#"{"quote":0.29}"#).unwrap();
    assert_eq!(response.quote, 0.29);
  }
}
