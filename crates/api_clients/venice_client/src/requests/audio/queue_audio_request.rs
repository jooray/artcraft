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

  /// User-supplied lyrics for lyric-capable music models.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lyrics_prompt: Option<String>,

  /// Ask the model to write lyrics from the prompt (only for models that
  /// support the optimizer and when no lyrics are supplied).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lyrics_optimizer: Option<bool>,
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
      lyrics_prompt: None,
      lyrics_optimizer: None,
    };
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["model"], "ace-step-15");
    assert!(value.get("force_instrumental").is_none());
    assert!(value.get("loop").is_none());
    assert!(value.get("lyrics_prompt").is_none());
    assert!(value.get("lyrics_optimizer").is_none());
  }

  #[test]
  fn lyrics_fields_serialize_when_set() {
    let request = VeniceQueueAudioRequest {
      model: "minimax-music-v26".to_string(),
      prompt: "a big fat frog song".to_string(),
      duration_seconds: None,
      force_instrumental: None,
      r#loop: None,
      speed: None,
      lyrics_prompt: Some("Verse 1: a big fat frog...".to_string()),
      lyrics_optimizer: None,
    };
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["lyrics_prompt"], "Verse 1: a big fat frog...");
    assert!(value.get("lyrics_optimizer").is_none());
  }
}
