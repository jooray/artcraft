use artcraft_router::api::router_aspect_ratio::RouterAspectRatio;
use artcraft_router::api::router_resolution::RouterResolution;
use log::warn;

use crate::core::commands::generate::generate_video::request::TauriVideoModel;

/// The maximum number of videos Venice queues per request (one).
pub const VENICE_MAX_VIDEO_BATCH: u16 = 1;

/// A desktop video model that can be dispatched to Venice, and its Venice
/// counterparts. Venice exposes separate text-to-video, image-to-video and
/// reference-to-video model ids, plus per-model supported aspect ratios and
/// resolutions.
#[derive(Clone, Copy, Debug)]
pub struct VeniceVideoModelSpec {
  pub text_to_video: &'static str,
  pub image_to_video: Option<&'static str>,
  /// Reference-to-video variant (character/style reference images).
  pub reference_to_video: Option<&'static str>,
  /// First-last-frame variant (start + end frame inputs).
  pub first_last_frame: Option<&'static str>,
  pub text_to_video_aspect_ratios: &'static [&'static str],
  pub text_to_video_resolutions: &'static [&'static str],
  pub image_to_video_aspect_ratios: &'static [&'static str],
  pub image_to_video_resolutions: &'static [&'static str],
  pub reference_to_video_aspect_ratios: &'static [&'static str],
  pub reference_to_video_resolutions: &'static [&'static str],
  /// Reference videos are supported (Seedance 2.x reference workflows).
  pub supports_reference_videos: bool,
  /// Reference audios are supported (Seedance 2.x reference workflows).
  pub supports_reference_audios: bool,
}

pub fn venice_video_model_spec(model: TauriVideoModel) -> Option<VeniceVideoModelSpec> {
  const WIDE_TALL_SQUARE: &[&str] = &["16:9", "9:16", "1:1"];
  const SEEDANCE_ASPECTS: &[&str] = &["21:9", "16:9", "4:3", "1:1", "3:4", "9:16"];
  const WAN_ASPECTS: &[&str] = &["16:9", "9:16", "1:1", "4:3", "3:4"];
  const WAN_EDIT_ASPECTS: &[&str] = &["adaptive", "16:9", "9:16", "1:1", "4:3", "3:4"];
  const MINIMAX_ASPECTS: &[&str] = &["16:9", "21:9", "4:3", "1:1", "3:4", "9:16"];
  const VIDU_ASPECTS: &[&str] = &["16:9", "9:16", "4:3", "3:4", "1:1"];
  const VEO_ASPECTS: &[&str] = &["16:9", "9:16"];
  const FLUX_ASPECTS: &[&str] = &["auto", "21:9", "16:9", "4:3", "1:1", "3:4", "9:16"];
  const SEEDANCE_2_RES: &[&str] = &["4k", "1080p", "720p", "480p"];
  const SEEDANCE_2_FAST_RES: &[&str] = &["720p", "480p"];
  const SEEDANCE_1_5_RES: &[&str] = &["1080p", "720p", "480p"];
  const WAN_RES: &[&str] = &["1080p", "720p", "480p"];
  const MINIMAX_RES: &[&str] = &["768P", "2K"];
  const VIDU_RES: &[&str] = &["360p", "540p", "720p", "1080p"];
  const VEO_RES: &[&str] = &["720p", "1080p"];
  const VEO_3P1_RES: &[&str] = &["720p", "1080p", "4k"];
  const FLUX_RES: &[&str] = &["720p", "1080p"];
  const NO_RESOLUTIONS: &[&str] = &[];
  const NO_ASPECTS: &[&str] = &[];
  const HAPPY_HORSE_RES: &[&str] = &["1080p", "720p"];

  let spec = match model {
    TauriVideoModel::Seedance2p0 => VeniceVideoModelSpec {
      text_to_video: "seedance-2-0-text-to-video-basic",
      image_to_video: Some("seedance-2-0-image-to-video-basic"),
      reference_to_video: Some("seedance-2-0-reference-to-video-basic"),
      first_last_frame: None,
      text_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      text_to_video_resolutions: SEEDANCE_2_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: SEEDANCE_2_RES,
      reference_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      reference_to_video_resolutions: SEEDANCE_2_RES,
      supports_reference_videos: true,
      supports_reference_audios: true,
    },
    TauriVideoModel::Seedance2p0Fast => VeniceVideoModelSpec {
      text_to_video: "seedance-2-0-fast-text-to-video-basic",
      image_to_video: Some("seedance-2-0-fast-image-to-video-basic"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      text_to_video_resolutions: SEEDANCE_2_FAST_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: SEEDANCE_2_FAST_RES,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Seedance1p5Pro => VeniceVideoModelSpec {
      text_to_video: "seedance-1-5-pro-text-to-video-basic",
      image_to_video: Some("seedance-1-5-pro-image-to-video-basic"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      text_to_video_resolutions: SEEDANCE_1_5_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: SEEDANCE_1_5_RES,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Seedance2p5 => VeniceVideoModelSpec {
      text_to_video: "seedance-2-5-text-to-video-basic",
      image_to_video: Some("seedance-2-5-image-to-video-basic"),
      reference_to_video: Some("seedance-2-5-reference-to-video-basic"),
      first_last_frame: None,
      text_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      text_to_video_resolutions: SEEDANCE_1_5_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: SEEDANCE_1_5_RES,
      reference_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      reference_to_video_resolutions: SEEDANCE_1_5_RES,
      supports_reference_videos: true,
      supports_reference_audios: true,
    },
    TauriVideoModel::Wan3p0 => VeniceVideoModelSpec {
      text_to_video: "wan-3-0-text-to-video",
      image_to_video: Some("wan-3-0-image-to-video"),
      reference_to_video: Some("wan-3-0-reference-to-video"),
      first_last_frame: None,
      text_to_video_aspect_ratios: WAN_ASPECTS,
      text_to_video_resolutions: WAN_RES,
      image_to_video_aspect_ratios: WAN_EDIT_ASPECTS,
      image_to_video_resolutions: WAN_RES,
      reference_to_video_aspect_ratios: WAN_EDIT_ASPECTS,
      reference_to_video_resolutions: WAN_RES,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Wan3p0Prime => VeniceVideoModelSpec {
      text_to_video: "wan-3-0-prime-text-to-video",
      image_to_video: Some("wan-3-0-prime-image-to-video"),
      reference_to_video: Some("wan-3-0-prime-reference-to-video"),
      first_last_frame: None,
      text_to_video_aspect_ratios: WAN_ASPECTS,
      text_to_video_resolutions: WAN_RES,
      image_to_video_aspect_ratios: WAN_EDIT_ASPECTS,
      image_to_video_resolutions: WAN_RES,
      reference_to_video_aspect_ratios: WAN_EDIT_ASPECTS,
      reference_to_video_resolutions: WAN_RES,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Kling2p5TurboPro => VeniceVideoModelSpec {
      text_to_video: "kling-2.5-turbo-pro-text-to-video",
      image_to_video: Some("kling-2.5-turbo-pro-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: NO_RESOLUTIONS,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Kling2p6Pro => VeniceVideoModelSpec {
      text_to_video: "kling-2.6-pro-text-to-video",
      image_to_video: Some("kling-2.6-pro-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: NO_RESOLUTIONS,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Kling3p0Standard => VeniceVideoModelSpec {
      text_to_video: "kling-v3-standard-text-to-video",
      image_to_video: Some("kling-v3-standard-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: NO_RESOLUTIONS,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Kling3p0Pro => VeniceVideoModelSpec {
      text_to_video: "kling-v3-pro-text-to-video",
      image_to_video: Some("kling-v3-pro-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: NO_RESOLUTIONS,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Veo3 => VeniceVideoModelSpec {
      text_to_video: "veo3-full-text-to-video",
      image_to_video: Some("veo3-full-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: NO_RESOLUTIONS,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Veo3Fast => VeniceVideoModelSpec {
      text_to_video: "veo3-fast-text-to-video",
      image_to_video: Some("veo3-fast-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: NO_RESOLUTIONS,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Veo3p1 => VeniceVideoModelSpec {
      text_to_video: "veo3.1-full-text-to-video",
      image_to_video: Some("veo3.1-full-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_3P1_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: VEO_3P1_RES,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Veo3p1Fast => VeniceVideoModelSpec {
      text_to_video: "veo3.1-fast-text-to-video",
      image_to_video: Some("veo3.1-fast-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_3P1_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: VEO_3P1_RES,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::HappyHorse1p0 => VeniceVideoModelSpec {
      text_to_video: "happyhorse-1-0-text-to-video",
      image_to_video: Some("happyhorse-1-0-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: HAPPY_HORSE_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: HAPPY_HORSE_RES,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::ViduQ3 => VeniceVideoModelSpec {
      text_to_video: "vidu-q3-text-to-video",
      image_to_video: Some("vidu-q3-image-to-video"),
      reference_to_video: None,
      first_last_frame: None,
      text_to_video_aspect_ratios: VIDU_ASPECTS,
      text_to_video_resolutions: VIDU_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: VIDU_RES,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::MinimaxH3 => VeniceVideoModelSpec {
      text_to_video: "minimax-h3-text-to-video",
      image_to_video: Some("minimax-h3-image-to-video"),
      reference_to_video: Some("minimax-h3-reference-to-video"),
      first_last_frame: None,
      text_to_video_aspect_ratios: MINIMAX_ASPECTS,
      text_to_video_resolutions: MINIMAX_RES,
      image_to_video_aspect_ratios: NO_ASPECTS,
      image_to_video_resolutions: MINIMAX_RES,
      reference_to_video_aspect_ratios: MINIMAX_ASPECTS,
      reference_to_video_resolutions: MINIMAX_RES,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    TauriVideoModel::Flux3 => VeniceVideoModelSpec {
      text_to_video: "flux-3-text-to-video",
      image_to_video: Some("flux-3-image-to-video"),
      reference_to_video: None,
      first_last_frame: Some("flux-3-first-last-frame-to-video"),
      text_to_video_aspect_ratios: FLUX_ASPECTS,
      text_to_video_resolutions: FLUX_RES,
      image_to_video_aspect_ratios: FLUX_ASPECTS,
      image_to_video_resolutions: FLUX_RES,
      reference_to_video_aspect_ratios: NO_ASPECTS,
      reference_to_video_resolutions: NO_RESOLUTIONS,
      supports_reference_videos: false,
      supports_reference_audios: false,
    },
    _ => return None,
  };

  Some(spec)
}

/// Whether Venice can serve this API model id (the string ids in requests).
pub fn venice_supports_video_model_id(model_id: &str) -> bool {
  matches!(
    model_id,
    "seedance_2p0"
      | "seedance_2p0_fast"
      | "seedance_1p5_pro"
      | "seedance_2p5"
      | "wan_3p0"
      | "wan_3p0_prime"
      | "kling_2p5_turbo_pro"
      | "kling_2p6_pro"
      | "kling_3p0_standard"
      | "kling_3p0_pro"
      | "veo_3"
      | "veo_3_fast"
      | "veo_3p1"
      | "veo_3p1_fast"
      | "vidu_q3"
      | "minimax_h3"
      | "flux_3"
      | "happy_horse_1p0"
  )
}

// ── Parameter mapping ──

/// Map the desktop aspect ratio to Venice's `aspect_ratio` string.
/// `auto`-style values return `None` (let the model decide).
pub fn map_aspect_ratio(value: RouterAspectRatio) -> Option<&'static str> {
  match value {
    RouterAspectRatio::Auto | RouterAspectRatio::Auto2k | RouterAspectRatio::Auto3k | RouterAspectRatio::Auto4k => None,
    RouterAspectRatio::Square | RouterAspectRatio::SquareHd => Some("1:1"),
    RouterAspectRatio::WideThreeByTwo => Some("3:2"),
    RouterAspectRatio::WideFourByThree => Some("4:3"),
    RouterAspectRatio::WideFiveByFour => Some("5:4"),
    RouterAspectRatio::WideSixteenByNine => Some("16:9"),
    RouterAspectRatio::WideTwentyOneByNine => Some("21:9"),
    RouterAspectRatio::TallTwoByThree => Some("2:3"),
    RouterAspectRatio::TallThreeByFour => Some("3:4"),
    RouterAspectRatio::TallFourByFive => Some("4:5"),
    RouterAspectRatio::TallNineBySixteen => Some("9:16"),
    RouterAspectRatio::TallNineByTwentyOne => Some("9:21"),
    RouterAspectRatio::Wide => Some("16:9"),
    RouterAspectRatio::Tall => Some("9:16"),
  }
}

/// Map the desktop resolution to Venice's `resolution` string.
pub fn map_resolution(value: RouterResolution) -> Option<&'static str> {
  match value {
    RouterResolution::FourEightyP => Some("480p"),
    RouterResolution::SevenTwentyP => Some("720p"),
    RouterResolution::TenEightyP => Some("1080p"),
    RouterResolution::TwoK => Some("2k"),
    RouterResolution::FourK => Some("4k"),
    RouterResolution::OneK | RouterResolution::ThreeK | RouterResolution::HalfK => None,
  }
}

/// Pick the model's supported resolution matching the requested value.
///
/// Lenient by design: when the vocabularies drift (e.g. a model lists `768P`
/// while the app offers `720p`), pick the nearest supported tier; when nothing
/// matches, omit the parameter and let the model default apply instead of
/// failing the request.
pub fn resolve_supported_resolution(value: RouterResolution, supported: &[&str]) -> Option<String> {
  if supported.is_empty() {
    return None;
  }

  let mapped = map_resolution(value)?;

  if let Some(match_value) = supported.iter().find(|s| s.eq_ignore_ascii_case(mapped)) {
    return Some((*match_value).to_string());
  }

  // Nearest tier: 720p <-> 768P families.
  if mapped == "720p" {
    if let Some(match_value) = supported.iter().find(|s| s.eq_ignore_ascii_case("768P")) {
      return Some((*match_value).to_string());
    }
  }

  warn!("Venice video model does not support resolution {:?} (supported: {:?}); omitting", mapped, supported);
  None
}

/// Pick the model's supported aspect ratio matching the requested value.
/// Like resolutions, this is lenient: unmatched values are omitted (Venice
/// infers from the input image or uses its default).
pub fn resolve_supported_aspect_ratio(value: RouterAspectRatio, supported: &[&str]) -> Option<String> {
  if supported.is_empty() {
    return None;
  }

  let mapped = match map_aspect_ratio(value) {
    Some(mapped) => mapped,
    None => return None, // auto: let the model decide
  };

  if let Some(match_value) = supported.iter().find(|s| s.eq_ignore_ascii_case(mapped)) {
    return Some((*match_value).to_string());
  }

  warn!("Venice video model does not support aspect ratio {:?} (supported: {:?}); omitting", mapped, supported);
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  mod model_mapping {
    use super::*;

    #[test]
    fn maps_the_expected_models() {
      let cases = [
        (TauriVideoModel::Seedance2p0, "seedance-2-0-text-to-video-basic"),
        (TauriVideoModel::Seedance2p0Fast, "seedance-2-0-fast-text-to-video-basic"),
        (TauriVideoModel::Seedance1p5Pro, "seedance-1-5-pro-text-to-video-basic"),
        (TauriVideoModel::Seedance2p5, "seedance-2-5-text-to-video-basic"),
        (TauriVideoModel::Wan3p0, "wan-3-0-text-to-video"),
        (TauriVideoModel::Wan3p0Prime, "wan-3-0-prime-text-to-video"),
        (TauriVideoModel::Kling2p5TurboPro, "kling-2.5-turbo-pro-text-to-video"),
        (TauriVideoModel::Kling2p6Pro, "kling-2.6-pro-text-to-video"),
        (TauriVideoModel::Kling3p0Standard, "kling-v3-standard-text-to-video"),
        (TauriVideoModel::Kling3p0Pro, "kling-v3-pro-text-to-video"),
        (TauriVideoModel::Veo3, "veo3-full-text-to-video"),
        (TauriVideoModel::Veo3Fast, "veo3-fast-text-to-video"),
        (TauriVideoModel::Veo3p1, "veo3.1-full-text-to-video"),
        (TauriVideoModel::Veo3p1Fast, "veo3.1-fast-text-to-video"),
        (TauriVideoModel::ViduQ3, "vidu-q3-text-to-video"),
        (TauriVideoModel::MinimaxH3, "minimax-h3-text-to-video"),
        (TauriVideoModel::Flux3, "flux-3-text-to-video"),
        (TauriVideoModel::HappyHorse1p0, "happyhorse-1-0-text-to-video"),
      ];
      for (tauri_model, expected) in cases {
        let spec = venice_video_model_spec(tauri_model).expect("model should be supported");
        assert_eq!(spec.text_to_video, expected);
        assert!(spec.image_to_video.is_some());
      }
    }

    #[test]
    fn reference_to_video_variants_match() {
      let cases = [
        (TauriVideoModel::Seedance2p0, "seedance-2-0-reference-to-video-basic"),
        (TauriVideoModel::Seedance2p5, "seedance-2-5-reference-to-video-basic"),
        (TauriVideoModel::Wan3p0, "wan-3-0-reference-to-video"),
        (TauriVideoModel::Wan3p0Prime, "wan-3-0-prime-reference-to-video"),
        (TauriVideoModel::MinimaxH3, "minimax-h3-reference-to-video"),
      ];
      for (tauri_model, expected) in cases {
        let spec = venice_video_model_spec(tauri_model).expect("model should be supported");
        assert_eq!(spec.reference_to_video, Some(expected));
      }

      assert_eq!(venice_video_model_spec(TauriVideoModel::Veo3p1).unwrap().reference_to_video, None);
      assert_eq!(venice_video_model_spec(TauriVideoModel::ViduQ3).unwrap().reference_to_video, None);
    }

    #[test]
    fn first_last_frame_variant_exists_for_flux3() {
      let spec = venice_video_model_spec(TauriVideoModel::Flux3).unwrap();
      assert_eq!(spec.first_last_frame, Some("flux-3-first-last-frame-to-video"));
    }

    #[test]
    fn rejects_unmapped_models() {
      for model in [TauriVideoModel::Sora2, TauriVideoModel::Kling16Pro, TauriVideoModel::GrokVideo, TauriVideoModel::Seedance10Lite] {
        assert!(venice_video_model_spec(model).is_none());
      }
    }

    #[test]
    fn model_ids_match_the_spec_table() {
      for id in [
        "seedance_2p0",
        "seedance_2p0_fast",
        "seedance_1p5_pro",
        "seedance_2p5",
        "wan_3p0",
        "wan_3p0_prime",
        "kling_2p5_turbo_pro",
        "kling_2p6_pro",
        "kling_3p0_standard",
        "kling_3p0_pro",
        "veo_3",
        "veo_3_fast",
        "veo_3p1",
        "veo_3p1_fast",
        "vidu_q3",
        "minimax_h3",
        "flux_3",
        "happy_horse_1p0",
      ] {
        assert!(venice_supports_video_model_id(id), "{} should be supported", id);
        let parsed: TauriVideoModel = serde_json::from_str(&format!("\"{}\"", id)).unwrap();
        assert!(venice_video_model_spec(parsed).is_some(), "{} should have a spec", id);
      }
      assert!(!venice_supports_video_model_id("sora_2"));
      assert!(!venice_supports_video_model_id("seedance_2p5_u"));
      assert!(!venice_supports_video_model_id("vidu_q3_turbo"));
    }
  }

  mod parameter_mapping {
    use super::*;

    #[test]
    fn aspect_ratios_map_to_expected_strings() {
      assert_eq!(map_aspect_ratio(RouterAspectRatio::Square), Some("1:1"));
      assert_eq!(map_aspect_ratio(RouterAspectRatio::SquareHd), Some("1:1"));
      assert_eq!(map_aspect_ratio(RouterAspectRatio::WideSixteenByNine), Some("16:9"));
      assert_eq!(map_aspect_ratio(RouterAspectRatio::TallNineBySixteen), Some("9:16"));
      assert_eq!(map_aspect_ratio(RouterAspectRatio::WideTwentyOneByNine), Some("21:9"));
      assert_eq!(map_aspect_ratio(RouterAspectRatio::Auto), None);
      assert_eq!(map_aspect_ratio(RouterAspectRatio::Auto4k), None);
    }

    #[test]
    fn resolutions_map_to_expected_strings() {
      assert_eq!(map_resolution(RouterResolution::FourEightyP), Some("480p"));
      assert_eq!(map_resolution(RouterResolution::SevenTwentyP), Some("720p"));
      assert_eq!(map_resolution(RouterResolution::TenEightyP), Some("1080p"));
      assert_eq!(map_resolution(RouterResolution::TwoK), Some("2k"));
      assert_eq!(map_resolution(RouterResolution::FourK), Some("4k"));
      assert_eq!(map_resolution(RouterResolution::OneK), None);
    }

    #[test]
    fn resolution_matcher_is_case_insensitive_and_lenient() {
      assert_eq!(resolve_supported_resolution(RouterResolution::TwoK, &["768P", "2K"]), Some("2K".to_string()));
      assert_eq!(resolve_supported_resolution(RouterResolution::SevenTwentyP, &["768P", "2K"]), Some("768P".to_string()));
      assert_eq!(resolve_supported_resolution(RouterResolution::FourK, &["1080p", "720p", "480p"]), None);
      assert_eq!(resolve_supported_resolution(RouterResolution::TenEightyP, &[]), None);
    }

    #[test]
    fn aspect_matcher_omits_unmatched_values() {
      assert_eq!(resolve_supported_aspect_ratio(RouterAspectRatio::WideSixteenByNine, &["16:9", "9:16"]), Some("16:9".to_string()));
      assert_eq!(resolve_supported_aspect_ratio(RouterAspectRatio::WideFourByThree, &["16:9", "9:16"]), None);
      assert_eq!(resolve_supported_aspect_ratio(RouterAspectRatio::Auto, &["16:9"]), None);
      assert_eq!(resolve_supported_aspect_ratio(RouterAspectRatio::Square, &[]), None);
    }
  }
}
