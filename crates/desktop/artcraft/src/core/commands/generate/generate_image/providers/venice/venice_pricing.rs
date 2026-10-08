use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use artcraft_router::api::router_aspect_ratio::RouterAspectRatio;
use artcraft_router::api::router_resolution::RouterResolution;
use log::warn;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use venice_client::client::venice_client::VeniceClient;
use venice_client::creds::venice_api_key::VeniceApiKey;
use venice_client::requests::audio::quote_audio::VeniceQuoteAudioRequest;
use venice_client::requests::video::quote_video::VeniceQuoteVideoRequest;

use crate::core::commands::generate::generate_video::venice::venice_video_model::{resolve_supported_aspect_ratio, resolve_supported_resolution, VeniceVideoModelSpec};
use crate::core::providers::credentials::payload::provider_credential_payload::ProviderCredentialPayload;
use crate::core::providers::credentials::provider_credential_key::ProviderCredentialKey;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;

/// How long cached model pricing stays fresh.
const PRICING_CACHE_TTL: Duration = Duration::from_secs(10 * 60);

// ── Shared estimate response plumbing ──

/// The estimate shape for BYOK providers: no ArtCraft credits, with an
/// optional USD estimate for the provider account.
pub fn zero_credits_json(maybe_usd_cents: Option<u32>) -> Value {
  serde_json::json!({
    "success": true,
    "cost_in_credits": 0,
    "cost_in_usd_cents": maybe_usd_cents,
    "is_free": false,
    "is_unlimited": false,
    "is_rate_limited": true,
    "has_watermark": false,
  })
}

/// Load the configured Venice API key, if any (estimates fail open).
pub fn read_venice_api_key(app: &AppHandle) -> Option<VeniceApiKey> {
  let credential_cache = app.state::<ProviderCredentialLoadingCache>();
  match credential_cache.get_credentials(ProviderCredentialKey::VeniceApiKey) {
    Ok(Some(ProviderCredentialPayload::ApiKey(data))) => Some(VeniceApiKey::from_str(data.as_str())),
    _ => None,
  }
}

/// Create a Venice client for an estimate, or `None` when unavailable.
pub fn venice_estimate_client(app: &AppHandle) -> Option<VeniceClient> {
  let api_key = read_venice_api_key(app)?;
  match VeniceClient::new(api_key) {
    Ok(client) => Some(client),
    Err(err) => {
      warn!("[Venice] Could not create a client for the estimate: {:?}", err);
      None
    },
  }
}

// ── Image pricing (from GET /models?type=image) ──

/// Estimate the per-request USD cost for a Venice image generation.
///
/// Venice charges per request (variants of one request do not multiply the
/// price — verified against the account balance).
pub async fn estimate_image_cost_usd(client: &VeniceClient, venice_model: &str, maybe_resolution_tier: Option<&str>, maybe_quality: Option<&str>) -> Option<f64> {
  let pricing = image_pricing(client).await?;
  let model_pricing = pricing.get(venice_model)?;

  per_request_price_usd(model_pricing, maybe_resolution_tier.unwrap_or("1K"), maybe_quality.unwrap_or("high"))
}

/// Cached image pricing per Venice model id.
fn pricing_cache() -> &'static Mutex<Option<(Instant, HashMap<String, Value>)>> {
  static CACHE: OnceLock<Mutex<Option<(Instant, HashMap<String, Value>)>>> = OnceLock::new();
  CACHE.get_or_init(|| Mutex::new(None))
}

async fn image_pricing(client: &VeniceClient) -> Option<HashMap<String, Value>> {
  {
    let guard = pricing_cache().lock().await;
    if let Some((loaded_at, pricing)) = guard.as_ref() {
      if loaded_at.elapsed() < PRICING_CACHE_TTL {
        return Some(pricing.clone());
      }
    }
  }

  let response = match client.list_image_models().await {
    Ok(response) => response,
    Err(err) => {
      warn!("[Venice] Could not load image pricing: {:?}", err);
      return None;
    },
  };

  let mut pricing = HashMap::new();
  for model in response.data {
    if let Some(spec) = model.model_spec {
      if let Some(pricing_value) = spec.pricing {
        pricing.insert(model.id, pricing_value);
      }
    }
  }

  *pricing_cache().lock().await = Some((Instant::now(), pricing.clone()));

  Some(pricing)
}

/// Resolve the price of one generation from a model's pricing object.
fn per_request_price_usd(pricing: &Value, resolution_tier: &str, quality: &str) -> Option<f64> {
  // Quality-aware models: pricing.quality.<tier>.<quality>.usd
  if let Some(quality_tiers) = pricing.get("quality") {
    if let Some(value) = quality_price_usd(quality_tiers, resolution_tier, quality) {
      return Some(value);
    }
  }

  // Resolution-tier models: pricing.resolutions.<tier>.usd
  if let Some(resolutions) = pricing.get("resolutions") {
    if let Some(value) = resolutions.get(resolution_tier).and_then(|tier| tier.get("usd")).and_then(Value::as_f64) {
      return Some(value);
    }
    // Fall back to any available tier (e.g. the model's default).
    if let Some(value) = resolutions.as_object().and_then(|object| object.values().next()).and_then(|tier| tier.get("usd")).and_then(Value::as_f64) {
      return Some(value);
    }
  }

  // Flat per-generation price.
  pricing.get("generation").and_then(|generation| generation.get("usd")).and_then(Value::as_f64)
}

fn quality_price_usd(quality_tiers: &Value, tier: &str, quality: &str) -> Option<f64> {
  let tier_object = quality_tiers.get(tier).or_else(|| quality_tiers.get("1K"))?;

  tier_object
    .get(quality)
    .or_else(|| tier_object.get("high"))
    .or_else(|| tier_object.get("medium"))
    .and_then(|entry| entry.get("usd"))
    .and_then(Value::as_f64)
}

// ── Video / audio quotes ──

/// Which Venice video variant an estimate request maps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoEstimateVariant {
  TextToVideo,
  ImageToVideo,
  ReferenceToVideo,
  FirstLastFrame,
}

/// Quote a video generation via `POST /video/quote`.
pub async fn estimate_video_cost_usd(client: &VeniceClient, spec: &VeniceVideoModelSpec, variant: VideoEstimateVariant, duration_seconds: u32, maybe_resolution: Option<RouterResolution>, maybe_aspect_ratio: Option<RouterAspectRatio>) -> Option<f64> {
  let (venice_model, aspect_ratio_options, resolution_options) = match variant {
    VideoEstimateVariant::TextToVideo => (spec.text_to_video, spec.text_to_video_aspect_ratios, spec.text_to_video_resolutions),
    VideoEstimateVariant::ImageToVideo => (spec.image_to_video?, spec.image_to_video_aspect_ratios, spec.image_to_video_resolutions),
    VideoEstimateVariant::FirstLastFrame => {
      let model = spec.first_last_frame.or(spec.image_to_video)?;
      (model, spec.image_to_video_aspect_ratios, spec.image_to_video_resolutions)
    },
    VideoEstimateVariant::ReferenceToVideo => (spec.reference_to_video?, spec.reference_to_video_aspect_ratios, spec.reference_to_video_resolutions),
  };

  let request = VeniceQuoteVideoRequest {
    model: venice_model.to_string(),
    duration: format!("{}s", duration_seconds),
    resolution: maybe_resolution.and_then(|value| resolve_supported_resolution(value, resolution_options)),
    aspect_ratio: maybe_aspect_ratio.and_then(|value| resolve_supported_aspect_ratio(value, aspect_ratio_options)),
  };

  match client.quote_video(&request).await {
    Ok(response) => Some(response.quote),
    Err(err) => {
      warn!("[Venice] Video quote failed: {:?}", err);
      None
    },
  }
}

/// Quote an audio generation via `POST /audio/quote`.
pub async fn estimate_audio_cost_usd(client: &VeniceClient, venice_model: &str, maybe_duration_seconds: Option<u32>) -> Option<f64> {
  let request = VeniceQuoteAudioRequest { model: venice_model.to_string(), duration_seconds: maybe_duration_seconds };

  match client.quote_audio(&request).await {
    Ok(response) => Some(response.quote),
    Err(err) => {
      warn!("[Venice] Audio quote failed: {:?}", err);
      None
    },
  }
}

/// USD → integer cents for the estimate API (round up so tiny costs don't show
/// as $0.00; zero stays zero).
pub fn usd_to_cents(usd: f64) -> u32 {
  if usd <= 0.0 {
    return 0;
  }
  (usd * 100.0).ceil() as u32
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn prices_flat_generation_models() {
    let pricing = json!({"generation": {"usd": 0.09, "diem": 0.09}});
    assert_eq!(per_request_price_usd(&pricing, "1K", "high"), Some(0.09));
  }

  #[test]
  fn prices_resolution_tier_models() {
    let pricing = json!({"resolutions": {"1K": {"usd": 0.18}, "2K": {"usd": 0.23}, "4K": {"usd": 0.35}}});
    assert_eq!(per_request_price_usd(&pricing, "2K", "high"), Some(0.23));
    // Unknown tier falls back to any available tier.
    assert_eq!(per_request_price_usd(&pricing, "3K", "high"), Some(0.18));
  }

  #[test]
  fn prices_quality_matrix_models() {
    let pricing = json!({
      "resolutions": {"1K": {"usd": 0.27}, "2K": {"usd": 0.51}},
      "quality": {
        "1K": {"low": {"usd": 0.02}, "high": {"usd": 0.27}},
        "2K": {"low": {"usd": 0.05}, "medium": {"usd": 0.3}, "high": {"usd": 0.51}}
      }
    });
    assert_eq!(per_request_price_usd(&pricing, "2K", "medium"), Some(0.3));
    assert_eq!(per_request_price_usd(&pricing, "1K", "low"), Some(0.02));
    // Unknown quality falls back to high.
    assert_eq!(per_request_price_usd(&pricing, "1K", "ultra"), Some(0.27));
  }

  #[test]
  fn cents_round_up_and_zero_stays_zero() {
    assert_eq!(usd_to_cents(0.0), 0);
    assert_eq!(usd_to_cents(0.19), 19);
    assert_eq!(usd_to_cents(0.00207), 1);
    assert_eq!(usd_to_cents(0.65), 65);
  }
}
