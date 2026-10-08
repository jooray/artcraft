use serde::Serialize;

/// Request body for `POST /image/generate`.
///
/// Venice uses model-specific sizing parameters:
/// - resolution-tier models (e.g. `nano-banana-pro`) take `aspect_ratio` + `resolution` ("1K"/"2K"/"4K").
/// - aspect-ratio models (e.g. `seedream-v4`) take `aspect_ratio` only.
/// - pixel models (e.g. `venice-sd35`) take `width` + `height` (bounded by the model's divisor).
///
/// Unknown fields are rejected by Venice (`additionalProperties: false`), so
/// unsupported sizing parameters must be omitted, not zeroed.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceGenerateImageRequest {
  pub model: String,

  pub prompt: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub negative_prompt: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub aspect_ratio: Option<String>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub resolution: Option<String>,

  /// Output quality for supported models (e.g. GPT Image 2): low/medium/high.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub quality: Option<String>,

  /// Number of images to generate (1–4). Only supported when `return_binary` is false.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub variants: Option<u32>,

  /// Inference steps. Only meaningful for pixel models; ignored elsewhere.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub steps: Option<u32>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub width: Option<u32>,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub height: Option<u32>,

  /// jpeg | png | webp
  pub format: String,

  pub safe_mode: bool,

  pub hide_watermark: bool,
}

impl VeniceGenerateImageRequest {
  /// A request with the desktop app's defaults: webp output, no safe-mode
  /// blurring and no visible watermark (the user pays for their own generation).
  pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
    Self { model: model.into(), prompt: prompt.into(), negative_prompt: None, aspect_ratio: None, resolution: None, quality: None, variants: None, steps: None, width: None, height: None, format: "webp".to_string(), safe_mode: false, hide_watermark: true }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn omitted_options_are_not_serialized() {
    let request = VeniceGenerateImageRequest::new("nano-banana-pro", "a duck");
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["model"], "nano-banana-pro");
    assert_eq!(value["prompt"], "a duck");
    assert_eq!(value["format"], "webp");
    assert_eq!(value["safe_mode"], false);
    assert_eq!(value["hide_watermark"], true);
    assert!(value.get("aspect_ratio").is_none());
    assert!(value.get("resolution").is_none());
    assert!(value.get("variants").is_none());
  }

  #[test]
  fn sizing_and_batch_options_are_serialized_when_present() {
    let mut request = VeniceGenerateImageRequest::new("gpt-image-2", "a duck");
    request.aspect_ratio = Some("16:9".to_string());
    request.resolution = Some("2K".to_string());
    request.quality = Some("high".to_string());
    request.variants = Some(3);
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["aspect_ratio"], "16:9");
    assert_eq!(value["resolution"], "2K");
    assert_eq!(value["quality"], "high");
    assert_eq!(value["variants"], 3);
  }
}
