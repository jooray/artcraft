use serde::Serialize;

/// Request body for `POST /audio/queue` (music and sound-effect generation).
///
/// `model` and `prompt` are required. Only send the optional toggles the
/// selected model supports.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceQueueAudioRequest {
  pub model: String,
  pub prompt: String,

  /// Optional duration hint in seconds (supported by some models).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub duration_seconds: Option<u32>,

  /// Render instrumental-only (supported by some music models).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub force_instrumental: Option<bool>,

  /// Seamless-loop rendering (supported by some sound models).
  #[serde(rename = "loop", skip_serializing_if = "Option::is_none")]
  pub r#loop: Option<bool>,

  /// Speed multiplier (supported by some models).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub speed: Option<f64>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn omitted_options_are_not_serialized() {
    let request = VeniceQueueAudioRequest {
      model: "ace-step-15".to_string(),
      prompt: "a calm lofi loop".to_string(),
      duration_seconds: None,
      force_instrumental: None,
      r#loop: None,
      speed: None,
    };
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["model"], "ace-step-15");
    assert!(value.get("force_instrumental").is_none());
    assert!(value.get("loop").is_none());
  }
}
