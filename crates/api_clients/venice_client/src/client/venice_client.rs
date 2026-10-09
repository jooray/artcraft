use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::creds::venice_api_key::VeniceApiKey;
use crate::error::venice_error::VeniceError;
use crate::requests::audio::queue_audio_request::VeniceQueueAudioRequest;
use crate::requests::audio::queue_audio_response::VeniceQueueAudioResponse;
use crate::requests::audio::quote_audio::VeniceQuoteAudioRequest;
use crate::requests::audio::quote_audio::VeniceQuoteAudioResponse;
use crate::requests::audio::retrieve_audio_request::VeniceRetrieveAudioRequest;
use crate::requests::audio::retrieve_audio_response::{VeniceAudioRetrieveResponse, VeniceAudioRetrieveStatus};
use crate::requests::edit_image::edit_image_request::VeniceEditImageRequest;
use crate::requests::edit_image::multi_edit_request::VeniceMultiEditImageRequest;
use crate::requests::generate_image::generate_image_request::VeniceGenerateImageRequest;
use crate::requests::generate_image::generate_image_response::VeniceGenerateImageResponse;
use crate::requests::models::list_models::VeniceModelsResponse;
use crate::requests::video::queue_video_request::VeniceQueueVideoRequest;
use crate::requests::video::queue_video_response::VeniceQueueVideoResponse;
use crate::requests::video::quote_video::VeniceQuoteVideoRequest;
use crate::requests::video::quote_video::VeniceQuoteVideoResponse;
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

  /// `POST /image/edit` (single-image edit / inpaint). The endpoint returns the
  /// edited image as binary data.
  pub async fn edit_image(&self, request: &VeniceEditImageRequest) -> Result<Vec<u8>, VeniceError> {
    self.post_for_image_bytes("/image/edit", request).await
  }

  /// `POST /image/multi-edit` (layered multi-image edit). Returns the edited
  /// image as binary data.
  pub async fn multi_edit_image(&self, request: &VeniceMultiEditImageRequest) -> Result<Vec<u8>, VeniceError> {
    self.post_for_image_bytes("/image/multi-edit", request).await
  }

  /// `POST /audio/queue` (music / sound effects). Asynchronous: returns a
  /// `queue_id` to poll with [`retrieve_audio`](Self::retrieve_audio).
  pub async fn queue_audio(&self, request: &VeniceQueueAudioRequest) -> Result<VeniceQueueAudioResponse, VeniceError> {
    self.post_json("/audio/queue", request).await
  }

  /// `POST /audio/retrieve`. Returns a status while processing or the finished
  /// audio file once complete.
  pub async fn retrieve_audio(&self, request: &VeniceRetrieveAudioRequest) -> Result<VeniceAudioRetrieveResponse, VeniceError> {
    let url = format!("{}{}", VENICE_API_BASE_URL, "/audio/retrieve");

    let response = self.http.post(&url).bearer_auth(self.api_key.as_str()).json(request).send().await?;

    let status = response.status();
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("").to_string();
    let bytes = response.bytes().await?;

    if !status.is_success() {
      return Err(VeniceError::Api { status, message: extract_error_message(&String::from_utf8_lossy(&bytes)) });
    }

    if content_type.starts_with("audio/") || content_type.starts_with("application/octet-stream") {
      return Ok(VeniceAudioRetrieveResponse::Audio { bytes: bytes.to_vec(), content_type });
    }

    let parsed: VeniceAudioRetrieveStatus = serde_json::from_slice(&bytes)?;

    Ok(VeniceAudioRetrieveResponse::Processing { status: parsed.status, maybe_average_execution_time: parsed.average_execution_time, maybe_execution_duration: parsed.execution_duration })
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

  /// `GET /models?type=...`: models with their pricing and constraint metadata.
  pub async fn list_models(&self, model_type: &str) -> Result<VeniceModelsResponse, VeniceError> {
    self.get_json(&format!("/models?type={}", model_type)).await
  }

  /// `GET /models?type=image`: image models with their pricing metadata.
  pub async fn list_image_models(&self) -> Result<VeniceModelsResponse, VeniceError> {
    self.list_models("image").await
  }

  /// `POST /video/quote`: estimated USD price for a video generation.
  pub async fn quote_video(&self, request: &VeniceQuoteVideoRequest) -> Result<VeniceQuoteVideoResponse, VeniceError> {
    self.post_json("/video/quote", request).await
  }

  /// `POST /audio/quote`: estimated USD price for an audio generation.
  pub async fn quote_audio(&self, request: &VeniceQuoteAudioRequest) -> Result<VeniceQuoteAudioResponse, VeniceError> {
    self.post_json("/audio/quote", request).await
  }

  // ── Helpers ──

  async fn get_json<TResponse>(&self, path: &str) -> Result<TResponse, VeniceError>
  where
    TResponse: DeserializeOwned,
  {
    let url = format!("{}{}", VENICE_API_BASE_URL, path);

    let response = self.http.get(&url).bearer_auth(self.api_key.as_str()).send().await?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
      return Err(VeniceError::Api { status, message: extract_error_message(&body) });
    }

    Ok(serde_json::from_str(&body)?)
  }

  /// POST an edit request and return the edited image bytes. Handles both the
  /// documented binary response and (defensively) base64 JSON responses.
  async fn post_for_image_bytes<TRequest>(&self, path: &str, request: &TRequest) -> Result<Vec<u8>, VeniceError>
  where
    TRequest: Serialize + Sync,
  {
    let url = format!("{}{}", VENICE_API_BASE_URL, path);

    let response = self.http.post(&url).bearer_auth(self.api_key.as_str()).json(request).send().await?;

    let status = response.status();
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("").to_string();
    let bytes = response.bytes().await?;

    if !status.is_success() {
      return Err(VeniceError::Api { status, message: extract_error_message(&String::from_utf8_lossy(&bytes)) });
    }

    if content_type.starts_with("image/") || content_type.starts_with("application/octet-stream") {
      return Ok(bytes.to_vec());
    }

    // Defensive: tolerate a JSON body that carries base64 images.
    if let Ok(parsed) = serde_json::from_slice::<VeniceGenerateImageResponse>(&bytes) {
      if let Some(first_image) = parsed.images.first() {
        return crate::utils::decode_base64_image(first_image);
      }
    }

    Err(VeniceError::Other(format!("Unexpected image edit response (content-type: {})", content_type)))
  }

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
