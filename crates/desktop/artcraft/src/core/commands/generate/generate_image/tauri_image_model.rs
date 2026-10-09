use serde_derive::Deserialize;

/// Unified image model enum covering text-to-image, image edit, and inpainting.
///
/// This is used in the Tauri command bridge.
/// Don't change the serializations without coordinating with the frontend.
///
/// MIGRATION (2026-07): we're moving the frontend to send storyteller-web omni
/// identifiers (`CommonImageModel` serde strings). Variants whose legacy Tauri
/// id differs accept BOTH spellings (see `From<String>`).
///
/// Runtime provider catalogs (Venice BYOK) can send model ids this build does
/// not know; those deserialize to [`TauriImageModel::Unknown`], and the raw id
/// travels in `TauriGenerateImageRequest::model_id`. Providers that only
/// handle compiled models reject `Unknown` at dispatch.
#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(from = "String")]
pub enum TauriImageModel {
  // Text-to-image models

  Flux1Dev,
  Flux1Schnell,
  FluxPro11,
  FluxPro11Ultra,
  GrokImage,
  Recraft3,
  GptImage1,
  GptImage1p5,
  GptImage2,
  GptImage2p5Flare,
  GptImage2p5Sunburst,
  Gemini25Flash,
  NanoBanana,
  NanoBanana2,
  NanoBananaPro,
  Seedream4,
  Seedream4p5,
  Seedream5Lite,

  // Venice-only models (BYOK): ids match Venice's own model ids.
  Flux2Pro,
  Flux2Max,
  Flux3Image,
  Krea2Turbo,
  Ideogram4p5,
  Recraft4,
  LumaUni1Max,
  MuseImage,
  NanoBanana2p1,
  Seedream5Pro,
  QwenImage3Pro,
  VeniceSd35,
  ZImageTurbo,

  // Generic Midjourney experience, served via the native Midjourney provider.
  Midjourney,

  // Versioned Midjourney models, served by storyteller-web via the
  // Artcraft provider (dispatched through artcraft_router).
  Midjourney7,
  Midjourney7Niji,
  Midjourney8,

  // Image edit models

  FluxProKontextMax,
  QwenEdit2511Angles,
  Flux2LoraAngles,

  // Inpainting models

  FluxDevJuggernaut,
  FluxPro1,

  /// An id from a runtime provider catalog (no compiled variant).
  Unknown,
}

impl From<String> for TauriImageModel {
  fn from(value: String) -> Self {
    match value.as_str() {
      "flux_1_dev" => Self::Flux1Dev,
      "flux_1_schnell" => Self::Flux1Schnell,
      "flux_pro_11" | "flux_pro_1p1" => Self::FluxPro11,
      "flux_pro_11_ultra" | "flux_pro_1p1_ultra" => Self::FluxPro11Ultra,
      "grok_image" | "grok_imagine_image" => Self::GrokImage,
      "recraft_3" => Self::Recraft3,
      "gpt_image_1" => Self::GptImage1,
      "gpt_image_1p5" => Self::GptImage1p5,
      "gpt_image_2" => Self::GptImage2,
      "gpt_image_2p5_flare" => Self::GptImage2p5Flare,
      "gpt_image_2p5_sunburst" => Self::GptImage2p5Sunburst,
      "gemini_25_flash" => Self::Gemini25Flash,
      "nano_banana" => Self::NanoBanana,
      "nano_banana_2" => Self::NanoBanana2,
      "nano_banana_pro" => Self::NanoBananaPro,
      "seedream_4" => Self::Seedream4,
      "seedream_4p5" => Self::Seedream4p5,
      "seedream_5_lite" => Self::Seedream5Lite,
      "flux-2-pro" => Self::Flux2Pro,
      "flux-2-max" => Self::Flux2Max,
      "flux-3-image" => Self::Flux3Image,
      "krea-2-turbo" => Self::Krea2Turbo,
      "ideogram-v4-5" => Self::Ideogram4p5,
      "recraft-v4" => Self::Recraft4,
      "luma-uni-1-max" => Self::LumaUni1Max,
      "muse-image" => Self::MuseImage,
      "nano-banana-2-1" => Self::NanoBanana2p1,
      "seedream-v5-pro" => Self::Seedream5Pro,
      "qwen-image-3-pro" => Self::QwenImage3Pro,
      "venice-sd35" => Self::VeniceSd35,
      "z-image-turbo" => Self::ZImageTurbo,
      "midjourney" => Self::Midjourney,
      "midjourney_7" => Self::Midjourney7,
      "midjourney_7_niji" => Self::Midjourney7Niji,
      "midjourney_8" => Self::Midjourney8,
      "flux_pro_kontext_max" => Self::FluxProKontextMax,
      "qwen_edit_2511_angles" => Self::QwenEdit2511Angles,
      "flux_2_lora_angles" => Self::Flux2LoraAngles,
      "flux_dev_juggernaut" => Self::FluxDevJuggernaut,
      "flux_pro_1" => Self::FluxPro1,
      _ => Self::Unknown,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(value: &str) -> TauriImageModel {
    serde_json::from_str(&format!("\"{}\"", value)).unwrap()
  }

  #[test]
  fn known_ids_parse_and_unknown_ids_fall_back() {
    assert_eq!(parse("nano_banana_2"), TauriImageModel::NanoBanana2);
    assert_eq!(parse("nano-banana-2-1"), TauriImageModel::NanoBanana2p1);
    assert_eq!(parse("flux_pro_11"), TauriImageModel::FluxPro11);
    assert_eq!(parse("flux_pro_1p1"), TauriImageModel::FluxPro11);
    assert_eq!(parse("grok_imagine_image"), TauriImageModel::GrokImage);
    // A future Venice catalog id.
    assert_eq!(parse("nano-banana-3"), TauriImageModel::Unknown);
  }
}
