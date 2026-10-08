use serde::Deserialize;

/// Response body for `POST /video/queue`.
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceQueueVideoResponse {
  pub model: String,
  pub queue_id: String,

  /// Pre-signed download URL, present only for VPS-backed (private) models.
  /// When present, `/video/retrieve` returns JSON status only and the finished
  /// file must be fetched from this URL.
  #[serde(default)]
  pub download_url: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_public_and_private_queue_responses() {
    let public: VeniceQueueVideoResponse = serde_json::from_str(
      r#"{"model":"seedance-2-0-text-to-video-basic","queue_id":"abc"}"#,
    )
    .unwrap();
    assert_eq!(public.queue_id, "abc");
    assert!(public.download_url.is_none());

    let private: VeniceQueueVideoResponse = serde_json::from_str(
      r#"{"model":"grok-imagine-text-to-video-private","queue_id":"abc","download_url":"https://x/y.mp4"}"#,
    )
    .unwrap();
    assert_eq!(private.download_url.as_deref(), Some("https://x/y.mp4"));
  }
}
