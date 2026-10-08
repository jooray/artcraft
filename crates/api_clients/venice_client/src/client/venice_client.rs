use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::creds::venice_api_key::VeniceApiKey;
use crate::error::venice_error::VeniceError;
use crate::requests::generate_image::generate_image_request::VeniceGenerateImageRequest;
use crate::requests::generate_image::generate_image_response::VeniceGenerateImageResponse;

pub const VENICE_API_BASE_URL: &str = "https://api.venice.ai/api/v1";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Generation can legitimately take minutes for slow models (e.g. GPT Image).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

/// A thin HTTP client for the Venice.ai API.
pub struct VeniceClient {
  api_key: VeniceApiKey,
  http: reqwest::Client,
}

impl VeniceClient {
  pub fn new(api_key: VeniceApiKey) -> Result<Self, VeniceError> {
    let http = reqwest::Client::builder().connect_timeout(CONNECT_TIMEOUT).timeout(REQUEST_TIMEOUT).build()?;
    Ok(Self { api_key, http })
  }

  pub fn get_api_key(&self) -> &VeniceApiKey {
    &self.api_key
  }

  // ── Endpoints ──

  /// `POST /image/generate`. Synchronous: the response carries base64 images.
  pub async fn generate_image(&self, request: &VeniceGenerateImageRequest) -> Result<VeniceGenerateImageResponse, VeniceError> {
    self.post_json("/image/generate", request).await
  }

  // ── Helpers ──

  async fn post_json<TRequest, TResponse>(&self, path: &str, request: &TRequest) -> Result<TResponse, VeniceError>
  where
    TRequest: Serialize + Sync,
    TResponse: DeserializeOwned,
  {
    let url = format!("{}{}", VENICE_API_BASE_URL, path);

    let response = self.http.post(&url).bearer_auth(self.api_key.as_str()).json(request).send().await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
      return Err(VeniceError::Api { status, message: extract_error_message(&body) });
    }

    Ok(serde_json::from_str(&body)?)
  }
}

/// Venice returns `{"error": "..."}` for most failures; fall back to the raw body.
fn extract_error_message(body: &str) -> String {
  serde_json::from_str::<serde_json::Value>(body).ok().and_then(|value| value.get("error").or_else(|| value.get("message")).and_then(serde_json::Value::as_str).map(str::to_owned)).unwrap_or_else(|| body.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn error_message_prefers_the_json_error_field() {
    assert_eq!(extract_error_message(r#"{"error":"Invalid API key"}"#), "Invalid API key");
    assert_eq!(extract_error_message(r#"{"message":"Nope"}"#), "Nope");
    assert_eq!(extract_error_message("plain text"), "plain text");
  }
}
