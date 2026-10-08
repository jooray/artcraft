use artcraft_router::api::router_aspect_ratio::RouterAspectRatio;
use artcraft_router::api::router_resolution::RouterResolution;

use crate::core::commands::generate::generate_video::request::TauriVideoModel;

/// The maximum number of videos Venice queues per request (one).
pub const VENICE_MAX_VIDEO_BATCH: u16 = 1;

/// A desktop video model that can be dispatched to Venice, and its Venice
/// counterparts. Venus exposes separate text-to-video and image-to-video model
/// ids, plus per-model supported aspect ratios and resolutions.
#[derive(Clone, Copy, Debug)]
pub struct VeniceVideoModelSpec {
  pub text_to_video: &'static str,
  pub image_to_video: Option<&'static str>,
  /// Supported aspect ratios for the text-to-video model (empty = the model
  /// takes no aspect ratio).
  pub text_to_video_aspect_ratios: &'static [&'static str],
  /// Supported resolutions for the text-to-video model (empty = the model
  /// takes no resolution).
  pub text_to_video_resolutions: &'static [&'static str],
  /// Supported resolutions for the image-to-video model (image-to-video models
  /// derive the aspect ratio from the input image).
  pub image_to_video_resolutions: &'static [&'static str],
}

pub fn venice_video_model_spec(model: TauriVideoModel) -> Option<VeniceVideoModelSpec> {
  const WIDE_TALL_SQUARE: &[&str] = &["16:9", "9:16", "1:1"];
  const SEEDANCE_ASPECTS: &[&str] = &["21:9", "16:9", "4:3", "1:1", "3:4", "9:16"];
  const VEO_ASPECTS: &[&str] = &["16:9", "9:16"];
  const SEEDANCE_2_RES: &[&str] = &["4k", "1080p", "720p", "480p"];
  const SEEDANCE_2_FAST_RES: &[&str] = &["720p", "480p"];
  const SEEDANCE_1_5_RES: &[&str] = &["1080p", "720p", "480p"];
  const VEO_RES: &[&str] = &["720p", "1080p"];
  const VEO_3P1_RES: &[&str] = &["720p", "1080p", "4k"];
  const NO_RESOLUTIONS: &[&str] = &[];
  const HAPPY_HORSE_RES: &[&str] = &["1080p", "720p"];

  let spec = match model {
    TauriVideoModel::Seedance2p0 => VeniceVideoModelSpec {
      text_to_video: "seedance-2-0-text-to-video-basic",
      image_to_video: Some("seedance-2-0-image-to-video-basic"),
      text_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      text_to_video_resolutions: SEEDANCE_2_RES,
      image_to_video_resolutions: SEEDANCE_2_RES,
    },
    TauriVideoModel::Seedance2p0Fast => VeniceVideoModelSpec {
      text_to_video: "seedance-2-0-fast-text-to-video-basic",
      image_to_video: Some("seedance-2-0-fast-image-to-video-basic"),
      text_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      text_to_video_resolutions: SEEDANCE_2_FAST_RES,
      image_to_video_resolutions: SEEDANCE_2_FAST_RES,
    },
    TauriVideoModel::Seedance1p5Pro => VeniceVideoModelSpec {
      text_to_video: "seedance-1-5-pro-text-to-video-basic",
      image_to_video: Some("seedance-1-5-pro-image-to-video-basic"),
      text_to_video_aspect_ratios: SEEDANCE_ASPECTS,
      text_to_video_resolutions: SEEDANCE_1_5_RES,
      image_to_video_resolutions: SEEDANCE_1_5_RES,
    },
    TauriVideoModel::Kling2p5TurboPro => VeniceVideoModelSpec {
      text_to_video: "kling-2.5-turbo-pro-text-to-video",
      image_to_video: Some("kling-2.5-turbo-pro-image-to-video"),
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_resolutions: NO_RESOLUTIONS,
    },
    TauriVideoModel::Kling2p6Pro => VeniceVideoModelSpec {
      text_to_video: "kling-2.6-pro-text-to-video",
      image_to_video: Some("kling-2.6-pro-image-to-video"),
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_resolutions: NO_RESOLUTIONS,
    },
    TauriVideoModel::Kling3p0Standard => VeniceVideoModelSpec {
      text_to_video: "kling-v3-standard-text-to-video",
      image_to_video: Some("kling-v3-standard-image-to-video"),
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_resolutions: NO_RESOLUTIONS,
    },
    TauriVideoModel::Kling3p0Pro => VeniceVideoModelSpec {
      text_to_video: "kling-v3-pro-text-to-video",
      image_to_video: Some("kling-v3-pro-image-to-video"),
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: NO_RESOLUTIONS,
      image_to_video_resolutions: NO_RESOLUTIONS,
    },
    TauriVideoModel::Veo3 => VeniceVideoModelSpec {
      text_to_video: "veo3-full-text-to-video",
      image_to_video: Some("veo3-full-image-to-video"),
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_RES,
      image_to_video_resolutions: NO_RESOLUTIONS,
    },
    TauriVideoModel::Veo3Fast => VeniceVideoModelSpec {
      text_to_video: "veo3-fast-text-to-video",
      image_to_video: Some("veo3-fast-image-to-video"),
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_RES,
      image_to_video_resolutions: NO_RESOLUTIONS,
    },
    TauriVideoModel::Veo3p1 => VeniceVideoModelSpec {
      text_to_video: "veo3.1-full-text-to-video",
      image_to_video: Some("veo3.1-full-image-to-video"),
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_3P1_RES,
      image_to_video_resolutions: VEO_3P1_RES,
    },
    TauriVideoModel::Veo3p1Fast => VeniceVideoModelSpec {
      text_to_video: "veo3.1-fast-text-to-video",
      image_to_video: Some("veo3.1-fast-image-to-video"),
      text_to_video_aspect_ratios: VEO_ASPECTS,
      text_to_video_resolutions: VEO_3P1_RES,
      image_to_video_resolutions: VEO_3P1_RES,
    },
    TauriVideoModel::HappyHorse1p0 => VeniceVideoModelSpec {
      text_to_video: "happyhorse-1-0-text-to-video",
      image_to_video: Some("happyhorse-1-0-image-to-video"),
      text_to_video_aspect_ratios: WIDE_TALL_SQUARE,
      text_to_video_resolutions: HAPPY_HORSE_RES,
      image_to_video_resolutions: HAPPY_HORSE_RES,
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
      | "kling_2p5_turbo_pro"
      | "kling_2p6_pro"
      | "kling_3p0_standard"
      | "kling_3p0_pro"
      | "veo_3"
      | "veo_3_fast"
      | "veo_3p1"
      | "veo_3p1_fast"
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
        (TauriVideoModel::Kling2p5TurboPro, "kling-2.5-turbo-pro-text-to-video"),
        (TauriVideoModel::Kling2p6Pro, "kling-2.6-pro-text-to-video"),
        (TauriVideoModel::Kling3p0Standard, "kling-v3-standard-text-to-video"),
        (TauriVideoModel::Kling3p0Pro, "kling-v3-pro-text-to-video"),
        (TauriVideoModel::Veo3, "veo3-full-text-to-video"),
        (TauriVideoModel::Veo3Fast, "veo3-fast-text-to-video"),
        (TauriVideoModel::Veo3p1, "veo3.1-full-text-to-video"),
        (TauriVideoModel::Veo3p1Fast, "veo3.1-fast-text-to-video"),
        (TauriVideoModel::HappyHorse1p0, "happyhorse-1-0-text-to-video"),
      ];
      for (tauri_model, expected) in cases {
        let spec = venice_video_model_spec(tauri_model).expect("model should be supported");
        assert_eq!(spec.text_to_video, expected);
        assert!(spec.image_to_video.is_some());
      }
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
        "kling_2p5_turbo_pro",
        "kling_2p6_pro",
        "kling_3p0_standard",
        "kling_3p0_pro",
        "veo_3",
        "veo_3_fast",
        "veo_3p1",
        "veo_3p1_fast",
        "happy_horse_1p0",
      ] {
        assert!(venice_supports_video_model_id(id), "{} should be supported", id);
        let parsed: TauriVideoModel = serde_json::from_str(&format!("\"{}\"", id)).unwrap();
        assert!(venice_video_model_spec(parsed).is_some(), "{} should have a spec", id);
      }
      assert!(!venice_supports_video_model_id("sora_2"));
      assert!(!venice_supports_video_model_id("kling_1p6_pro"));
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
  }
}
