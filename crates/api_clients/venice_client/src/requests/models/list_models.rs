use serde::Deserialize;

/// Response body for `GET /models?type=...`.
#[derive(Clone, Debug, Deserialize)]
pub struct VeniceModelsResponse {
  pub data: Vec<VeniceModel>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VeniceModel {
  pub id: String,
  #[serde(default)]
  pub model_spec: Option<VeniceModelSpec>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VeniceModelSpec {
  #[serde(default)]
  pub name: Option<String>,

  /// The pricing object; its shape varies per model (flat `generation`,
  /// `resolutions` tiers, and/or a `quality` matrix).
  #[serde(default)]
  pub pricing: Option<serde_json::Value>,

  /// Model constraints (sizing, durations, aspect ratios, …); shape varies by
  /// model type.
  #[serde(default)]
  pub constraints: Option<serde_json::Value>,

  #[serde(default, rename = "supportsStyleReferences")]
  pub supports_style_references: Option<bool>,

  #[serde(default)]
  pub offline: Option<bool>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_a_models_response() {
    let response: VeniceModelsResponse = serde_json::from_str(
      r#"{"data":[{"id":"nano-banana-pro","model_spec":{"name":"Nano Banana Pro","pricing":{"resolutions":{"1K":{"usd":0.18},"2K":{"usd":0.23}}}}}]}"#,
    )
    .unwrap();
    assert_eq!(response.data.len(), 1);
    let spec = response.data[0].model_spec.as_ref().unwrap();
    assert_eq!(spec.pricing.as_ref().unwrap()["resolutions"]["1K"]["usd"], 0.18);
  }
}
