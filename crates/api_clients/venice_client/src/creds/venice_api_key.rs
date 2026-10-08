/// A Venice.ai API key (a bearer token from https://venice.ai/settings/api).
#[derive(Clone)]
pub struct VeniceApiKey(String);

impl VeniceApiKey {
  pub fn new(api_key: String) -> Self {
    Self(api_key.trim().to_string())
  }

  pub fn from_str(api_key: &str) -> Self {
    Self(api_key.trim().to_string())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}
