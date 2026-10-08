use serde::Serialize;

/// Request body for `POST /image/edit` (single-image edit / inpaint).
///
/// `image` is a base64 string (raw or data URL) or an http(s) URL.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceEditImageRequest {
  pub model: String,
  pub prompt: String,

  /// Base64 (raw or data URL) or an http(s) URL.
  pub image: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub aspect_ratio: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolution: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub quality: Option<String>,

  /// jpeg | png | webp
  pub output_format: String,

  pub safe_mode: bool,
}

impl VeniceEditImageRequest {
  /// Defaults matching the generation endpoint: webp output, no safe-mode blur.
  pub fn new(model: impl Into<String>, prompt: impl Into<String>, image: impl Into<String>) -> Self {
    Self {
      model: model.into(),
      prompt: prompt.into(),
      image: image.into(),
      aspect_ratio: None,
      resolution: None,
      quality: None,
      output_format: "webp".to_string(),
      safe_mode: false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn omitted_options_are_not_serialized() {
    let request = VeniceEditImageRequest::new("nano-banana-2-edit", "make it green", "aGVsbG8=");
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["model"], "nano-banana-2-edit");
    assert_eq!(value["image"], "aGVsbG8=");
    assert_eq!(value["output_format"], "webp");
    assert!(value.get("aspect_ratio").is_none());
  }
}
