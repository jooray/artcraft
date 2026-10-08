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

  /// Reference images for reference-to-video models (URLs or data URLs).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reference_image_urls: Option<Vec<String>>,

  /// Reference videos for models with reference video support (Seedance 2.x).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reference_video_urls: Option<Vec<String>>,

  /// Reference audios for models with reference audio support (Seedance 2.x).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reference_audio_urls: Option<Vec<String>>,

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
      reference_image_urls: None,
      reference_video_urls: None,
      reference_audio_urls: None,
      audio: None,
    };
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["model"], "seedance-2-0-text-to-video-basic");
    assert_eq!(value["duration"], "5s");
    assert!(value.get("aspect_ratio").is_none());
    assert!(value.get("image_url").is_none());
  }
}
