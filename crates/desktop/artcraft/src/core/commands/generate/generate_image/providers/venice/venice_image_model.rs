use enums::common::generation::common_aspect_ratio::CommonAspectRatio;
use enums::common::generation::common_quality::CommonQuality;
use enums::common::generation::common_resolution::CommonResolution;
use enums::common::generation_provider::GenerationProvider;
use log::warn;

use crate::core::commands::generate::generate_image::tauri_image_model::TauriImageModel;

/// Venice accepts at most 4 variants per image request.
pub const VENICE_MAX_VARIANTS: u32 = 4;

/// Base long side for pixel-based models (e.g. `venice-sd35`).
const PIXEL_BASE_SIDE: u32 = 1024;

/// Venice caps `width`/`height` at 1280 pixels.
const PIXEL_MAX_SIDE: u32 = 1280;

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
  /// `width`/`height` in pixels, rounded to the model's divisor.
  Pixels { divisor: u32 },
}

/// A desktop image model that can be dispatched to Venice, and its counterpart.
#[derive(Clone, Copy, Debug)]
pub struct VeniceImageModelSpec {
  /// The Venice API model id for generation.
  pub venice_model: &'static str,
  /// The Venice API model id for single/multi-image edits, when one exists.
  pub edit_model: Option<&'static str>,
  pub sizing: VeniceSizingMode,
  /// The Venice resolution tiers the model supports (empty = the model takes
  /// no resolution).
  pub resolutions: &'static [&'static str],
  /// Whether the model honors the `quality` parameter (low/medium/high).
  pub supports_quality: bool,
}

/// Map a desktop model to its Venice counterpart.
///
/// Includes the models that exist under (roughly) the same identity on both
/// sides, plus Venice-only models offered as desktop additions.
pub fn venice_model_spec(tauri_model: TauriImageModel) -> Option<VeniceImageModelSpec> {
  const TIERS: VeniceSizingMode = VeniceSizingMode::AspectRatioAndResolution;
  const ASPECT_ONLY: VeniceSizingMode = VeniceSizingMode::AspectRatioOnly;
  const TIERS_1K_2K: &[&str] = &["1K", "2K"];
  const TIERS_1K_2K_4K: &[&str] = &["1K", "2K", "4K"];
  const NO_TIERS: &[&str] = &[];

  let spec = match tauri_model {
    TauriImageModel::NanoBanana2 => VeniceImageModelSpec { venice_model: "nano-banana-2", edit_model: Some("nano-banana-2-edit"), sizing: TIERS, resolutions: TIERS_1K_2K_4K, supports_quality: false },
    TauriImageModel::NanoBananaPro => VeniceImageModelSpec { venice_model: "nano-banana-pro", edit_model: Some("nano-banana-pro-edit"), sizing: TIERS, resolutions: TIERS_1K_2K_4K, supports_quality: false },
    TauriImageModel::GptImage1p5 => VeniceImageModelSpec { venice_model: "gpt-image-1-5", edit_model: Some("gpt-image-1-5-edit"), sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::GptImage2 => VeniceImageModelSpec { venice_model: "gpt-image-2", edit_model: Some("gpt-image-2-edit"), sizing: TIERS, resolutions: TIERS_1K_2K_4K, supports_quality: true },
    TauriImageModel::GptImage2p5Flare => VeniceImageModelSpec { venice_model: "gpt-image-2-5-flare", edit_model: Some("gpt-image-2-5-flare-edit"), sizing: TIERS, resolutions: TIERS_1K_2K_4K, supports_quality: true },
    TauriImageModel::GptImage2p5Sunburst => VeniceImageModelSpec { venice_model: "gpt-image-2-5-sunburst", edit_model: Some("gpt-image-2-5-sunburst-edit"), sizing: TIERS, resolutions: TIERS_1K_2K_4K, supports_quality: true },
    TauriImageModel::Seedream4 => VeniceImageModelSpec { venice_model: "seedream-v4", edit_model: Some("seedream-v4-edit"), sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::Seedream5Lite => VeniceImageModelSpec { venice_model: "seedream-v5-lite", edit_model: Some("seedream-v5-lite-edit"), sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    // Venice-only models.
    TauriImageModel::Flux2Pro => VeniceImageModelSpec { venice_model: "flux-2-pro", edit_model: None, sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::Flux2Max => VeniceImageModelSpec { venice_model: "flux-2-max", edit_model: Some("flux-2-max-edit"), sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::Flux3Image => VeniceImageModelSpec { venice_model: "flux-3-image", edit_model: Some("flux-3-image-edit"), sizing: TIERS, resolutions: TIERS_1K_2K_4K, supports_quality: false },
    TauriImageModel::Krea2Turbo => VeniceImageModelSpec { venice_model: "krea-2-turbo", edit_model: None, sizing: TIERS, resolutions: TIERS_1K_2K, supports_quality: false },
    TauriImageModel::Ideogram4p5 => VeniceImageModelSpec { venice_model: "ideogram-v4-5", edit_model: Some("ideogram-v4-5-edit"), sizing: TIERS, resolutions: TIERS_1K_2K, supports_quality: false },
    TauriImageModel::Recraft4 => VeniceImageModelSpec { venice_model: "recraft-v4", edit_model: None, sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::LumaUni1Max => VeniceImageModelSpec { venice_model: "luma-uni-1-max", edit_model: Some("luma-uni-1-max-edit"), sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::MuseImage => VeniceImageModelSpec { venice_model: "muse-image", edit_model: Some("muse-image-edit"), sizing: ASPECT_ONLY, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::NanoBanana2p1 => VeniceImageModelSpec { venice_model: "nano-banana-2-1", edit_model: Some("nano-banana-2-1-edit"), sizing: TIERS, resolutions: TIERS_1K_2K_4K, supports_quality: false },
    TauriImageModel::Seedream5Pro => VeniceImageModelSpec { venice_model: "seedream-v5-pro", edit_model: Some("seedream-v5-pro-edit"), sizing: TIERS, resolutions: TIERS_1K_2K, supports_quality: false },
    TauriImageModel::QwenImage3Pro => VeniceImageModelSpec { venice_model: "qwen-image-3-pro", edit_model: Some("qwen-image-3-pro-edit"), sizing: TIERS, resolutions: TIERS_1K_2K, supports_quality: false },
    TauriImageModel::VeniceSd35 => VeniceImageModelSpec { venice_model: "venice-sd35", edit_model: None, sizing: VeniceSizingMode::Pixels { divisor: 16 }, resolutions: NO_TIERS, supports_quality: false },
    TauriImageModel::ZImageTurbo => VeniceImageModelSpec { venice_model: "z-image-turbo", edit_model: None, sizing: VeniceSizingMode::Pixels { divisor: 8 }, resolutions: NO_TIERS, supports_quality: false },
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
  matches!(
    model_id,
    "nano_banana_2"
      | "nano_banana_pro"
      | "gpt_image_1p5"
      | "gpt_image_2"
      | "gpt_image_2p5_flare"
      | "gpt_image_2p5_sunburst"
      | "seedream_4"
      | "seedream_5_lite"
      | "flux-2-pro"
      | "flux-2-max"
      | "flux-3-image"
      | "krea-2-turbo"
      | "ideogram-v4-5"
      | "recraft-v4"
      | "luma-uni-1-max"
      | "muse-image"
      | "nano-banana-2-1"
      | "seedream-v5-pro"
      | "qwen-image-3-pro"
      | "venice-sd35"
      | "z-image-turbo"
  )
}

/// Venice image model ids this build already maps (generate + edit variants).
/// The runtime catalog only surfaces ids OUTSIDE this list.
pub const VENICE_STATIC_IMAGE_MODEL_IDS: &[&str] = &[
  "nano-banana-2",
  "nano-banana-2-edit",
  "nano-banana-pro",
  "nano-banana-pro-edit",
  "gpt-image-1-5",
  "gpt-image-1-5-edit",
  "gpt-image-2",
  "gpt-image-2-edit",
  "gpt-image-2-5-flare",
  "gpt-image-2-5-flare-edit",
  "gpt-image-2-5-sunburst",
  "gpt-image-2-5-sunburst-edit",
  "seedream-v4",
  "seedream-v4-edit",
  "seedream-v5-lite",
  "seedream-v5-lite-edit",
  "flux-2-pro",
  "flux-2-max",
  "flux-2-max-edit",
  "flux-3-image",
  "flux-3-image-edit",
  "krea-2-turbo",
  "ideogram-v4-5",
  "ideogram-v4-5-edit",
  "recraft-v4",
  "luma-uni-1-max",
  "luma-uni-1-max-edit",
  "muse-image",
  "muse-image-edit",
  "nano-banana-2-1",
  "nano-banana-2-1-edit",
  "seedream-v5-pro",
  "seedream-v5-pro-edit",
  "qwen-image-3-pro",
  "qwen-image-3-pro-edit",
  "venice-sd35",
  "z-image-turbo",
];

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

/// Pick the model's supported resolution tier matching the requested value.
///
/// Lenient: when the requested tier is not offered by the model, omit the
/// parameter and let Venice use its default instead of failing the request.
pub fn resolve_supported_resolution(value: CommonResolution, supported: &[&str]) -> Option<String> {
  if supported.is_empty() {
    return None;
  }

  let mapped = map_resolution(value)?;

  if let Some(match_value) = supported.iter().find(|s| s.eq_ignore_ascii_case(mapped)) {
    return Some((*match_value).to_string());
  }

  warn!("Venice model does not support resolution {:?} (supported: {:?}); omitting", mapped, supported);
  None
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

/// Compute `width`/`height` for pixel-based models from the aspect ratio.
///
/// The long side is 1024 (capped at Venice's 1280 limit) and both sides round
/// down to the model's divisor.
pub fn pixel_dimensions(maybe_aspect: Option<CommonAspectRatio>, divisor: u32) -> (u32, u32) {
  let divisor = divisor.max(1);

  let (width_ratio, height_ratio) = maybe_aspect
    .and_then(|value| parse_aspect_ratio(map_aspect_ratio(value)))
    .unwrap_or((1, 1));

  let (mut width, mut height) = if width_ratio >= height_ratio {
    (PIXEL_BASE_SIDE, (PIXEL_BASE_SIDE * height_ratio / width_ratio).max(1))
  } else {
    ((PIXEL_BASE_SIDE * width_ratio / height_ratio).max(1), PIXEL_BASE_SIDE)
  };

  width = round_down_to_multiple(width, divisor).clamp(divisor, PIXEL_MAX_SIDE);
  height = round_down_to_multiple(height, divisor).clamp(divisor, PIXEL_MAX_SIDE);

  (width, height)
}

fn parse_aspect_ratio(value: &str) -> Option<(u32, u32)> {
  let (width, height) = value.split_once(':')?;
  let width = width.trim().parse::<u32>().ok()?;
  let height = height.trim().parse::<u32>().ok()?;
  if width == 0 || height == 0 {
    return None;
  }
  Some((width, height))
}

fn round_down_to_multiple(value: u32, divisor: u32) -> u32 {
  (value / divisor) * divisor
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
      let cases = [
        (TauriImageModel::NanoBanana2, "nano-banana-2", Some("nano-banana-2-edit")),
        (TauriImageModel::NanoBananaPro, "nano-banana-pro", Some("nano-banana-pro-edit")),
        (TauriImageModel::GptImage1p5, "gpt-image-1-5", Some("gpt-image-1-5-edit")),
        (TauriImageModel::GptImage2, "gpt-image-2", Some("gpt-image-2-edit")),
        (TauriImageModel::GptImage2p5Flare, "gpt-image-2-5-flare", Some("gpt-image-2-5-flare-edit")),
        (TauriImageModel::GptImage2p5Sunburst, "gpt-image-2-5-sunburst", Some("gpt-image-2-5-sunburst-edit")),
        (TauriImageModel::Seedream4, "seedream-v4", Some("seedream-v4-edit")),
        (TauriImageModel::Seedream5Lite, "seedream-v5-lite", Some("seedream-v5-lite-edit")),
      ];
      for (tauri_model, expected_model, expected_edit_model) in cases {
        let spec = venice_model_spec(tauri_model).expect("model should be supported");
        assert_eq!(spec.venice_model, expected_model);
        assert_eq!(spec.edit_model, expected_edit_model);
        assert!(venice_supports_tauri_model(tauri_model));
      }
    }

    #[test]
    fn maps_the_venice_only_models() {
      let cases = [
        (TauriImageModel::Flux2Pro, "flux-2-pro", None),
        (TauriImageModel::Flux2Max, "flux-2-max", Some("flux-2-max-edit")),
        (TauriImageModel::Flux3Image, "flux-3-image", Some("flux-3-image-edit")),
        (TauriImageModel::Krea2Turbo, "krea-2-turbo", None),
        (TauriImageModel::Ideogram4p5, "ideogram-v4-5", Some("ideogram-v4-5-edit")),
        (TauriImageModel::Recraft4, "recraft-v4", None),
        (TauriImageModel::LumaUni1Max, "luma-uni-1-max", Some("luma-uni-1-max-edit")),
        (TauriImageModel::MuseImage, "muse-image", Some("muse-image-edit")),
        (TauriImageModel::NanoBanana2p1, "nano-banana-2-1", Some("nano-banana-2-1-edit")),
        (TauriImageModel::Seedream5Pro, "seedream-v5-pro", Some("seedream-v5-pro-edit")),
        (TauriImageModel::QwenImage3Pro, "qwen-image-3-pro", Some("qwen-image-3-pro-edit")),
        (TauriImageModel::VeniceSd35, "venice-sd35", None),
        (TauriImageModel::ZImageTurbo, "z-image-turbo", None),
      ];
      for (tauri_model, expected_model, expected_edit_model) in cases {
        let spec = venice_model_spec(tauri_model).expect("model should be supported");
        assert_eq!(spec.venice_model, expected_model);
        assert_eq!(spec.edit_model, expected_edit_model);
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
    fn venice_only_ids_deserialize() {
      for id in ["flux-2-pro", "flux-2-max", "flux-3-image", "krea-2-turbo", "ideogram-v4-5", "recraft-v4", "luma-uni-1-max", "muse-image", "nano-banana-2-1", "seedream-v5-pro", "qwen-image-3-pro", "venice-sd35", "z-image-turbo"] {
        assert!(venice_supports_model_id(id), "{} should be supported", id);
        let parsed: TauriImageModel = serde_json::from_str(&format!("\"{}\"", id)).unwrap();
        assert!(venice_model_spec(parsed).is_some(), "{} should have a spec", id);
      }
    }

    #[test]
    fn static_id_list_matches_the_spec_table() {
      let variants = [
        TauriImageModel::NanoBanana2,
        TauriImageModel::NanoBananaPro,
        TauriImageModel::GptImage1p5,
        TauriImageModel::GptImage2,
        TauriImageModel::GptImage2p5Flare,
        TauriImageModel::GptImage2p5Sunburst,
        TauriImageModel::Seedream4,
        TauriImageModel::Seedream5Lite,
        TauriImageModel::Flux2Pro,
        TauriImageModel::Flux2Max,
        TauriImageModel::Flux3Image,
        TauriImageModel::Krea2Turbo,
        TauriImageModel::Ideogram4p5,
        TauriImageModel::Recraft4,
        TauriImageModel::LumaUni1Max,
        TauriImageModel::MuseImage,
        TauriImageModel::NanoBanana2p1,
        TauriImageModel::Seedream5Pro,
        TauriImageModel::QwenImage3Pro,
        TauriImageModel::VeniceSd35,
        TauriImageModel::ZImageTurbo,
      ];

      let mut expected: Vec<&str> = Vec::new();
      for variant in variants {
        let spec = venice_model_spec(variant).expect("variant should have a spec");
        expected.push(spec.venice_model);
        if let Some(model) = spec.edit_model {
          expected.push(model);
        }
      }
      expected.sort_unstable();

      let mut listed = VENICE_STATIC_IMAGE_MODEL_IDS.to_vec();
      listed.sort_unstable();
      assert_eq!(listed, expected);
    }

    #[test]
    fn resolution_tier_models_are_marked_correctly() {
      assert_eq!(venice_model_spec(TauriImageModel::NanoBananaPro).unwrap().sizing, VeniceSizingMode::AspectRatioAndResolution,);
      assert_eq!(venice_model_spec(TauriImageModel::Seedream4).unwrap().sizing, VeniceSizingMode::AspectRatioOnly,);
      assert_eq!(venice_model_spec(TauriImageModel::VeniceSd35).unwrap().sizing, VeniceSizingMode::Pixels { divisor: 16 },);
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
    fn resolution_matcher_is_lenient() {
      assert_eq!(resolve_supported_resolution(CommonResolution::TwoK, &["1K", "2K"]), Some("2K".to_string()));
      assert_eq!(resolve_supported_resolution(CommonResolution::FourK, &["1K", "2K"]), None);
      assert_eq!(resolve_supported_resolution(CommonResolution::OneK, &[]), None);
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

    #[test]
    fn pixel_dimensions_respect_the_divisor() {
      assert_eq!(pixel_dimensions(Some(CommonAspectRatio::Square), 16), (1024, 1024));
      assert_eq!(pixel_dimensions(Some(CommonAspectRatio::WideSixteenByNine), 16), (1024, 576));
      assert_eq!(pixel_dimensions(Some(CommonAspectRatio::TallNineBySixteen), 16), (576, 1024));
      assert_eq!(pixel_dimensions(None, 16), (1024, 1024));

      let (width, height) = pixel_dimensions(Some(CommonAspectRatio::TallFourByFive), 16);
      assert_eq!(width % 16, 0);
      assert_eq!(height % 16, 0);
      assert!(width < height);

      let (width, height) = pixel_dimensions(Some(CommonAspectRatio::WideTwentyOneByNine), 8);
      assert_eq!(width, 1024);
      assert_eq!(height, 432);
    }
  }
}
