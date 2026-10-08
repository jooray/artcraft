use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::creds::venice_api_key::VeniceApiKey;
use crate::error::venice_error::VeniceError;
use crate::requests::generate_image::generate_image_request::VeniceGenerateImageRequest;
use crate::requests::generate_image::generate_image_response::VeniceGenerateImageResponse;
use crate::requests::video::queue_video_request::VeniceQueueVideoRequest;
use crate::requests::video::queue_video_response::VeniceQueueVideoResponse;
use crate::requests::video::retrieve_video_request::VeniceRetrieveVideoRequest;
use crate::requests::video::retrieve_video_response::{VeniceVideoRetrieveResponse, VeniceVideoRetrieveStatus};

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

  /// `POST /video/queue`. Asynchronous: returns a `queue_id` to poll with
  /// [`retrieve_video`](Self::retrieve_video).
  pub async fn queue_video(&self, request: &VeniceQueueVideoRequest) -> Result<VeniceQueueVideoResponse, VeniceError> {
    self.post_json("/video/queue", request).await
  }

  /// `POST /video/retrieve`. Returns a status while processing, the finished
  /// file directly, or a `COMPLETED` status when the file must be fetched from
  /// the queue's `download_url` instead.
  pub async fn retrieve_video(&self, request: &VeniceRetrieveVideoRequest) -> Result<VeniceVideoRetrieveResponse, VeniceError> {
    let url = format!("{}{}", VENICE_API_BASE_URL, "/video/retrieve");

    let response = self.http.post(&url).bearer_auth(self.api_key.as_str()).json(request).send().await?;

    let status = response.status();
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("").to_string();
    let bytes = response.bytes().await?;

    if !status.is_success() {
      return Err(VeniceError::Api { status, message: extract_error_message(&String::from_utf8_lossy(&bytes)) });
    }

    if content_type.starts_with("video/") || content_type.starts_with("application/octet-stream") {
      return Ok(VeniceVideoRetrieveResponse::Video { bytes: bytes.to_vec() });
    }

    let parsed: VeniceVideoRetrieveStatus = serde_json::from_slice(&bytes)?;

    Ok(if parsed.status == "COMPLETED" {
      VeniceVideoRetrieveResponse::CompletedWithDownloadUrl
    } else {
      VeniceVideoRetrieveResponse::Processing { status: parsed.status, maybe_average_execution_time: parsed.average_execution_time, maybe_execution_duration: parsed.execution_duration }
    })
  }

  /// Fetch a finished file from a pre-signed `download_url` (no auth needed).
  pub async fn download_url_bytes(&self, download_url: &str) -> Result<Vec<u8>, VeniceError> {
    let response = self.http.get(download_url).send().await?;

    let status = response.status();
    let bytes = response.bytes().await?;

    if !status.is_success() {
      return Err(VeniceError::Api { status, message: extract_error_message(&String::from_utf8_lossy(&bytes)) });
    }

    Ok(bytes.to_vec())
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
