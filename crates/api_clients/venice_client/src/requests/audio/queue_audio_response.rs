use serde::Deserialize;

/// Response body for `POST /audio/queue`.
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceQueueAudioResponse {
  pub model: String,
  pub queue_id: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_the_queue_response() {
    let response: VeniceQueueAudioResponse = serde_json::from_str(
      r#"{"model":"ace-step-15","queue_id":"abc-123"}"#,
    )
    .unwrap();
    assert_eq!(response.queue_id, "abc-123");
  }
}
