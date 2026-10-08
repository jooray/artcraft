use enums::common::generation::common_aspect_ratio::CommonAspectRatio;
use enums::common::generation::common_quality::CommonQuality;
use enums::common::generation::common_resolution::CommonResolution;
use enums::common::generation_provider::GenerationProvider;

use crate::core::commands::generate::generate_image::tauri_image_model::TauriImageModel;

/// Venice accepts at most 4 variants per image request.
pub const VENICE_MAX_VARIANTS: u32 = 4;

/// How a Venice model takes its sizing parameters.
///
/// See the Venice docs ("Sizing Options"): mixing sizing fields across models
/// can be rejected with a 400, so only the supported fields are sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VeniceSizingMode {
  /// `aspect_ratio` + `resolution` ("1K"/"2K"/"4K").
  AspectRatioAndResolution,
  /// `aspect_ratio` only; Venice exposes no resolution tiers for the model.
  AspectRatioOnly,
}

/// A desktop image model that can be dispatched to Venice, and its counterpart.
#[derive(Clone, Copy, Debug)]
pub struct VeniceImageModelSpec {
  /// The Venice API model id for generation.
  pub venice_model: &'static str,
  /// The Venice API model id for single/multi-image edits.
  pub edit_model: &'static str,
  pub sizing: VeniceSizingMode,
  /// Whether the model honors the `quality` parameter (low/medium/high).
  pub supports_quality: bool,
}

/// Map a desktop model to its Venice counterpart.
///
/// Only models that exist under (roughly) the same identity on both sides are
/// offered; anything else returns `None` and is rejected up front.
pub fn venice_model_spec(tauri_model: TauriImageModel) -> Option<VeniceImageModelSpec> {
  const TIERS: VeniceSizingMode = VeniceSizingMode::AspectRatioAndResolution;
  const ASPECT_ONLY: VeniceSizingMode = VeniceSizingMode::AspectRatioOnly;

  let spec = match tauri_model {
    TauriImageModel::NanoBanana2 => VeniceImageModelSpec { venice_model: "nano-banana-2", edit_model: "nano-banana-2-edit", sizing: TIERS, supports_quality: false },
    TauriImageModel::NanoBananaPro => VeniceImageModelSpec { venice_model: "nano-banana-pro", edit_model: "nano-banana-pro-edit", sizing: TIERS, supports_quality: false },
    TauriImageModel::GptImage1p5 => VeniceImageModelSpec { venice_model: "gpt-image-1-5", edit_model: "gpt-image-1-5-edit", sizing: ASPECT_ONLY, supports_quality: false },
    TauriImageModel::GptImage2 => VeniceImageModelSpec { venice_model: "gpt-image-2", edit_model: "gpt-image-2-edit", sizing: TIERS, supports_quality: true },
    TauriImageModel::GptImage2p5Flare => VeniceImageModelSpec { venice_model: "gpt-image-2-5-flare", edit_model: "gpt-image-2-5-flare-edit", sizing: TIERS, supports_quality: true },
    TauriImageModel::GptImage2p5Sunburst => VeniceImageModelSpec { venice_model: "gpt-image-2-5-sunburst", edit_model: "gpt-image-2-5-sunburst-edit", sizing: TIERS, supports_quality: true },
    TauriImageModel::Seedream4 => VeniceImageModelSpec { venice_model: "seedream-v4", edit_model: "seedream-v4-edit", sizing: ASPECT_ONLY, supports_quality: false },
    TauriImageModel::Seedream5Lite => VeniceImageModelSpec { venice_model: "seedream-v5-lite", edit_model: "seedream-v5-lite-edit", sizing: ASPECT_ONLY, supports_quality: false },
    _ => return None,
  };

  Some(spec)
}

/// Whether Venice can serve this model at all.
pub fn venice_supports_tauri_model(tauri_model: TauriImageModel) -> bool {
  venice_model_spec(tauri_model).is_some()
}

/// Whether Venice can serve this API model id (the string ids in requests).
pub fn venice_supports_model_id(model_id: &str) -> bool {
  matches!(model_id, "nano_banana_2" | "nano_banana_pro" | "gpt_image_1p5" | "gpt_image_2" | "gpt_image_2p5_flare" | "gpt_image_2p5_sunburst" | "seedream_4" | "seedream_5_lite")
}

// ── Parameter mapping ──

/// Map the desktop aspect ratio to Venice's `aspect_ratio` string.
///
/// Venice is lenient about undocumented ratios (it accepted 5:4 and auto when
/// tested), so no per-model allowlist is enforced here.
pub fn map_aspect_ratio(value: CommonAspectRatio) -> &'static str {
  match value {
    CommonAspectRatio::Auto | CommonAspectRatio::Auto2k | CommonAspectRatio::Auto3k | CommonAspectRatio::Auto4k => "auto",
    CommonAspectRatio::Square | CommonAspectRatio::SquareHd => "1:1",
    CommonAspectRatio::WideThreeByTwo => "3:2",
    CommonAspectRatio::WideFourByThree => "4:3",
    CommonAspectRatio::WideFiveByFour => "5:4",
    CommonAspectRatio::WideSixteenByNine => "16:9",
    CommonAspectRatio::WideTwentyOneByNine => "21:9",
    CommonAspectRatio::TallTwoByThree => "2:3",
    CommonAspectRatio::TallThreeByFour => "3:4",
    CommonAspectRatio::TallFourByFive => "4:5",
    CommonAspectRatio::TallNineBySixteen => "9:16",
    CommonAspectRatio::TallNineByTwentyOne => "9:21",
    CommonAspectRatio::Wide => "16:9",
    CommonAspectRatio::Tall => "9:16",
  }
}

/// Map the desktop resolution to Venice's `resolution` tier.
///
/// Video resolutions (480p/720p/1080p) are not image tiers and map to `None`.
pub fn map_resolution(value: CommonResolution) -> Option<&'static str> {
  match value {
    CommonResolution::OneK => Some("1K"),
    CommonResolution::TwoK => Some("2K"),
    CommonResolution::ThreeK => Some("3K"),
    CommonResolution::FourK => Some("4K"),
    CommonResolution::HalfK => Some("0.5K"),
    CommonResolution::FourEightyP | CommonResolution::SevenTwentyP | CommonResolution::TenEightyP => None,
  }
}

/// Map the desktop quality to Venice's `quality` tiers (low/medium/high).
///
/// Venice only documents low/medium/high ("auto" omits the parameter; x-high
/// and max fall back to high, the best Venice tier).
pub fn map_quality(value: CommonQuality) -> Option<&'static str> {
  match value {
    CommonQuality::Low => Some("low"),
    CommonQuality::Medium => Some("medium"),
    CommonQuality::High | CommonQuality::XHigh | CommonQuality::Max => Some("high"),
    CommonQuality::Auto => None,
  }
}

/// The provider this module dispatches to (used by task/event plumbing).
pub const VENICE_PROVIDER: GenerationProvider = GenerationProvider::Venice;

#[cfg(test)]
mod tests {
  use super::*;

  mod model_mapping {
    use super::*;

    #[test]
    fn maps_the_expected_models() {
      let cases = [(TauriImageModel::NanoBanana2, "nano-banana-2", "nano-banana-2-edit"), (TauriImageModel::NanoBananaPro, "nano-banana-pro", "nano-banana-pro-edit"), (TauriImageModel::GptImage1p5, "gpt-image-1-5", "gpt-image-1-5-edit"), (TauriImageModel::GptImage2, "gpt-image-2", "gpt-image-2-edit"), (TauriImageModel::GptImage2p5Flare, "gpt-image-2-5-flare", "gpt-image-2-5-flare-edit"), (TauriImageModel::GptImage2p5Sunburst, "gpt-image-2-5-sunburst", "gpt-image-2-5-sunburst-edit"), (TauriImageModel::Seedream4, "seedream-v4", "seedream-v4-edit"), (TauriImageModel::Seedream5Lite, "seedream-v5-lite", "seedream-v5-lite-edit")];
      for (tauri_model, expected_model, expected_edit_model) in cases {
        let spec = venice_model_spec(tauri_model).expect("model should be supported");
        assert_eq!(spec.venice_model, expected_model);
        assert_eq!(spec.edit_model, expected_edit_model);
        assert!(venice_supports_tauri_model(tauri_model));
      }
    }

    #[test]
    fn rejects_unmapped_models() {
      for model in [TauriImageModel::Flux1Dev, TauriImageModel::Midjourney, TauriImageModel::NanoBanana] {
        assert!(venice_model_spec(model).is_none());
      }
    }

    #[test]
    fn supported_model_ids_match_the_spec_table() {
      for id in ["nano_banana_2", "nano_banana_pro", "gpt_image_1p5", "gpt_image_2", "gpt_image_2p5_flare", "gpt_image_2p5_sunburst", "seedream_4", "seedream_5_lite"] {
        assert!(venice_supports_model_id(id), "{} should be supported", id);
        let parsed: TauriImageModel = serde_json::from_str(&format!("\"{}\"", id)).unwrap();
        assert!(venice_model_spec(parsed).is_some(), "{} should have a spec", id);
      }
      assert!(!venice_supports_model_id("flux_1_dev"));
      assert!(!venice_supports_model_id("grok_imagine_image"));
    }

    #[test]
    fn resolution_tier_models_are_marked_correctly() {
      assert_eq!(venice_model_spec(TauriImageModel::NanoBananaPro).unwrap().sizing, VeniceSizingMode::AspectRatioAndResolution,);
      assert_eq!(venice_model_spec(TauriImageModel::Seedream4).unwrap().sizing, VeniceSizingMode::AspectRatioOnly,);
    }
  }

  mod parameter_mapping {
    use super::*;

    #[test]
    fn every_aspect_ratio_maps() {
      for value in CommonAspectRatio::all_variants() {
        let mapped = map_aspect_ratio(value);
        assert!(!mapped.is_empty(), "aspect ratio {:?} did not map", value);
      }
    }

    #[test]
    fn known_aspect_ratios_map_to_expected_strings() {
      assert_eq!(map_aspect_ratio(CommonAspectRatio::Square), "1:1");
      assert_eq!(map_aspect_ratio(CommonAspectRatio::SquareHd), "1:1");
      assert_eq!(map_aspect_ratio(CommonAspectRatio::WideSixteenByNine), "16:9");
      assert_eq!(map_aspect_ratio(CommonAspectRatio::TallNineBySixteen), "9:16");
      assert_eq!(map_aspect_ratio(CommonAspectRatio::Auto), "auto");
      assert_eq!(map_aspect_ratio(CommonAspectRatio::Wide), "16:9");
      assert_eq!(map_aspect_ratio(CommonAspectRatio::Tall), "9:16");
    }

    #[test]
    fn image_resolutions_map_and_video_resolutions_do_not() {
      assert_eq!(map_resolution(CommonResolution::OneK), Some("1K"));
      assert_eq!(map_resolution(CommonResolution::TwoK), Some("2K"));
      assert_eq!(map_resolution(CommonResolution::ThreeK), Some("3K"));
      assert_eq!(map_resolution(CommonResolution::FourK), Some("4K"));
      assert_eq!(map_resolution(CommonResolution::HalfK), Some("0.5K"));
      assert_eq!(map_resolution(CommonResolution::FourEightyP), None);
      assert_eq!(map_resolution(CommonResolution::SevenTwentyP), None);
      assert_eq!(map_resolution(CommonResolution::TenEightyP), None);
    }

    #[test]
    fn quality_maps_to_the_three_venice_tiers() {
      assert_eq!(map_quality(CommonQuality::Low), Some("low"));
      assert_eq!(map_quality(CommonQuality::Medium), Some("medium"));
      assert_eq!(map_quality(CommonQuality::High), Some("high"));
      assert_eq!(map_quality(CommonQuality::XHigh), Some("high"));
      assert_eq!(map_quality(CommonQuality::Max), Some("high"));
      assert_eq!(map_quality(CommonQuality::Auto), None);
    }
  }
}
