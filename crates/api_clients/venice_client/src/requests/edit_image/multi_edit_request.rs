use serde::Serialize;

/// Request body for `POST /image/multi-edit` (layered multi-image edit).
///
/// Up to the model's `maxInputImages`; images are base64 strings (raw or data
/// URL) or http(s) URLs. NB: the field is `modelId` here, unlike `/image/edit`.
#[derive(Clone, Debug, Serialize)]
pub struct VeniceMultiEditImageRequest {
  #[serde(rename = "modelId")]
  pub model_id: String,

  pub prompt: String,

  pub images: Vec<String>,

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

impl VeniceMultiEditImageRequest {
  pub fn new(model_id: impl Into<String>, prompt: impl Into<String>, images: Vec<String>) -> Self {
    Self {
      model_id: model_id.into(),
      prompt: prompt.into(),
      images,
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
  fn uses_the_model_id_field_name() {
    let request = VeniceMultiEditImageRequest::new("gpt-image-2-edit", "combine", vec!["aGVsbG8=".into(), "d29ybGQ=".into()]);
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["modelId"], "gpt-image-2-edit");
    assert_eq!(value["images"].as_array().unwrap().len(), 2);
    assert!(value.get("model").is_none());
  }
}
