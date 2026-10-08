use serde::Serialize;

/// Request body for `POST /video/queue`.
///
/// `model` and `duration` are required. Available fields and valid values vary
/// by model, so only include what the selected model supports.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceQueueVideoRequest {
  pub model: String,

  /// A duration string like `"5s"`.
  pub duration: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub prompt: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub negative_prompt: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub aspect_ratio: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolution: Option<String>,

  /// Start frame for image-to-video models (URL or data URL).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image_url: Option<String>,

  /// End frame for models that support transitions (URL or data URL).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_image_url: Option<String>,

  /// Whether to generate audio when the model supports it.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub audio: Option<bool>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn omitted_options_are_not_serialized() {
    let request = VeniceQueueVideoRequest {
      model: "seedance-2-0-text-to-video-basic".to_string(),
      duration: "5s".to_string(),
      prompt: Some("a duck".to_string()),
      negative_prompt: None,
      aspect_ratio: None,
      resolution: None,
      image_url: None,
      end_image_url: None,
      audio: None,
    };
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["model"], "seedance-2-0-text-to-video-basic");
    assert_eq!(value["duration"], "5s");
    assert!(value.get("aspect_ratio").is_none());
    assert!(value.get("image_url").is_none());
  }
}
