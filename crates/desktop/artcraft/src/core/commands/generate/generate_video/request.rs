use artcraft_router::api::router_aspect_ratio::RouterAspectRatio;
use artcraft_router::api::router_resolution::RouterResolution;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::ux::tauri_command_caller::TauriCommandCaller;
use serde_derive::{Deserialize, Serialize};
use tokens::tokens::characters::CharacterToken;
use tokens::tokens::media_files::MediaFileToken;

use crate::core::commands::response::success_response_wrapper::SerializeMarker;

/// This is used in the Tauri command bridge.
/// Don't change the serializations without coordinating with the frontend.
///
/// MIGRATION (2026-07): we're moving the frontend to send storyteller-web omni
/// identifiers (`CommonVideoModel` serde strings). Variants whose legacy Tauri
/// id differs accept BOTH spellings (see `From<String>`).
///
/// Runtime provider catalogs (Venice BYOK) can send model ids this build does
/// not know; those deserialize to [`TauriVideoModel::Unknown`], and the raw id
/// travels in `TauriGenerateVideoRequest::model_id`.
#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(from = "String")]
pub enum TauriVideoModel {
  GrokVideo,
  GrokImagineVideo1p5,
  Kling16Pro,
  Kling21Pro,
  Kling21Master,
  Kling2p5TurboPro,
  Kling2p6Pro,
  Kling3p0Standard,
  Kling3p0Pro,
  HappyHorse1p0,
  Seedance10Lite,
  Seedance1p5Pro,
  Seedance2p0,
  Seedance2p0Fast,
  Seedance2p5,
  Wan3p0,
  Wan3p0Prime,
  ViduQ3,
  MinimaxH3,
  Flux3,
  Sora2,
  Sora2Pro,
  Veo2,
  Veo3,
  Veo3Fast,
  Veo3p1,
  Veo3p1Fast,

  /// An id from a runtime provider catalog (no compiled variant).
  Unknown,
}

impl From<String> for TauriVideoModel {
  fn from(value: String) -> Self {
    match value.as_str() {
      "grok_video" | "grok_imagine_video" => Self::GrokVideo,
      "grok_imagine_video_1p5" => Self::GrokImagineVideo1p5,
      "kling_1.6_pro" | "kling_1p6_pro" => Self::Kling16Pro,
      "kling_2.1_pro" | "kling_2p1_pro" => Self::Kling21Pro,
      "kling_2.1_master" | "kling_2p1_master" => Self::Kling21Master,
      "kling_2p5_turbo_pro" => Self::Kling2p5TurboPro,
      "kling_2p6_pro" => Self::Kling2p6Pro,
      "kling_3p0_standard" => Self::Kling3p0Standard,
      "kling_3p0_pro" => Self::Kling3p0Pro,
      "happy_horse_1p0" => Self::HappyHorse1p0,
      "seedance_1.0_lite" | "seedance_1p0_lite" => Self::Seedance10Lite,
      "seedance_1p5_pro" => Self::Seedance1p5Pro,
      "seedance_2p0" => Self::Seedance2p0,
      "seedance_2p0_fast" => Self::Seedance2p0Fast,
      "seedance_2p5" => Self::Seedance2p5,
      "wan_3p0" => Self::Wan3p0,
      "wan_3p0_prime" => Self::Wan3p0Prime,
      "vidu_q3" => Self::ViduQ3,
      "minimax_h3" => Self::MinimaxH3,
      "flux_3" => Self::Flux3,
      "sora_2" => Self::Sora2,
      "sora_2_pro" => Self::Sora2Pro,
      "veo_2" => Self::Veo2,
      "veo_3" => Self::Veo3,
      "veo_3_fast" => Self::Veo3Fast,
      "veo_3p1" => Self::Veo3p1,
      "veo_3p1_fast" => Self::Veo3p1Fast,
      _ => Self::Unknown,
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct TauriGenerateVideoRequest {
  pub provider: Option<GenerationProvider>,
  pub model: Option<TauriVideoModel>,

  /// The raw model id sent by the frontend. Carries runtime provider catalog
  /// ids (Venice BYOK) that have no compiled `TauriVideoModel` variant.
  #[serde(default)]
  pub model_id: Option<String>,

  pub prompt: Option<String>,
  pub negative_prompt: Option<String>,

  #[deprecated(note = "Use start_frame_image_media_token instead")]
  pub image_media_token: Option<MediaFileToken>,

  pub start_frame_image_media_token: Option<MediaFileToken>,
  pub end_frame_image_media_token: Option<MediaFileToken>,

  pub reference_image_media_tokens: Option<Vec<MediaFileToken>>,
  pub reference_video_media_tokens: Option<Vec<MediaFileToken>>,
  pub reference_audio_media_tokens: Option<Vec<MediaFileToken>>,
  pub reference_character_tokens: Option<Vec<CharacterToken>>,

  pub aspect_ratio: Option<RouterAspectRatio>,
  pub resolution: Option<RouterResolution>,

  pub duration_seconds: Option<u16>,
  pub generate_audio: Option<bool>,
  pub video_batch_count: Option<u16>,

  #[deprecated(note = "Use start_frame_image_media_token instead")]
  pub sora_orientation: Option<SoraOrientation>,

  #[deprecated(note = "Use start_frame_image_media_token instead")]
  pub grok_aspect_ratio: Option<GrokAspectRatio>,

  pub frontend_caller: Option<TauriCommandCaller>,
  pub frontend_subscriber_id: Option<String>,
  pub frontend_subscriber_payload: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SoraOrientation {
  Portrait,
  Landscape,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum GrokAspectRatio {
  Portrait,
  Landscape,
  Square,
}

#[derive(Serialize)]
pub struct TauriGenerateVideoResponse {
}

impl SerializeMarker for TauriGenerateVideoResponse {}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TauriGenerateVideoErrorType {
  ModelNotSpecified,
  NoProviderAvailable,
  ServerError,
  NeedsFalApiKey,
  FalError,
  /// Needs a Venice.ai API key
  NeedsVeniceApiKey,
  NeedsStorytellerCredentials,
}
