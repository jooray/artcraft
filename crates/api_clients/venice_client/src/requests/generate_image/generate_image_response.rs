use serde::Deserialize;

/// Response body for `POST /image/generate`.
///
/// `images` entries are base64-encoded image data (one per requested variant).
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceGenerateImageResponse {
  pub id: String,
  pub images: Vec<String>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_the_documented_response_shape() {
    let response: VeniceGenerateImageResponse = serde_json::from_str(
      r#"{
        "id": "generate-image-123",
        "images": ["UklGRg==", "UklGRh=="],
        "request": {"data": {"model": "nano-banana-2"}},
        "timing": {"inferenceDuration": 1, "inferencePreprocessingTime": 1, "inferenceQueueTime": 1, "total": 3}
      }"#,
    )
    .unwrap();
    assert_eq!(response.id, "generate-image-123");
    assert_eq!(response.images, vec!["UklGRg==", "UklGRh=="]);
  }
}
