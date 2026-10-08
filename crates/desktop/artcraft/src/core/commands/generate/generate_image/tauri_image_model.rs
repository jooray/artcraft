use serde_derive::Deserialize;

/// Unified image model enum covering text-to-image, image edit, and inpainting.
///
/// This is used in the Tauri command bridge.
/// Don't change the serializations without coordinating with the frontend.
///
/// MIGRATION (2026-07): we're moving the frontend to send storyteller-web omni
/// identifiers (`CommonImageModel` serde strings). Variants whose legacy Tauri
/// id differs carry the omni id as a `#[serde(alias)]` so BOTH deserialize.
/// Once the frontend is 100% omni ids, the legacy renames can be dropped.
#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TauriImageModel {
  // Text-to-image models

  #[serde(rename = "flux_1_dev")]
  Flux1Dev,
  #[serde(rename = "flux_1_schnell")]
  Flux1Schnell,
  #[serde(rename = "flux_pro_11", alias = "flux_pro_1p1")]
  FluxPro11,
  #[serde(rename = "flux_pro_11_ultra", alias = "flux_pro_1p1_ultra")]
  FluxPro11Ultra,
  #[serde(rename = "grok_image", alias = "grok_imagine_image")]
  GrokImage,
  #[serde(rename = "recraft_3")]
  Recraft3,
  #[serde(rename = "gpt_image_1")]
  GptImage1,
  #[serde(rename = "gpt_image_1p5")]
  GptImage1p5,
  #[serde(rename = "gpt_image_2")]
  GptImage2,
  #[serde(rename = "gpt_image_2p5_flare")]
  GptImage2p5Flare,
  #[serde(rename = "gpt_image_2p5_sunburst")]
  GptImage2p5Sunburst,
  #[serde(rename = "gemini_25_flash")]
  Gemini25Flash,
  #[serde(rename = "nano_banana")]
  NanoBanana,
  #[serde(rename = "nano_banana_2")]
  NanoBanana2,
  #[serde(rename = "nano_banana_pro")]
  NanoBananaPro,
  #[serde(rename = "seedream_4")]
  Seedream4,
  #[serde(rename = "seedream_4p5")]
  Seedream4p5,
  #[serde(rename = "seedream_5_lite")]
  Seedream5Lite,

  // Venice-only models (BYOK): ids match Venice's own model ids.
  #[serde(rename = "flux-2-pro")]
  Flux2Pro,
  #[serde(rename = "flux-2-max")]
  Flux2Max,
  #[serde(rename = "flux-3-image")]
  Flux3Image,
  #[serde(rename = "krea-2-turbo")]
  Krea2Turbo,
  #[serde(rename = "ideogram-v4-5")]
  Ideogram4p5,
  #[serde(rename = "recraft-v4")]
  Recraft4,
  #[serde(rename = "luma-uni-1-max")]
  LumaUni1Max,
  #[serde(rename = "muse-image")]
  MuseImage,
  #[serde(rename = "nano-banana-2-1")]
  NanoBanana2p1,
  #[serde(rename = "seedream-v5-pro")]
  Seedream5Pro,
  #[serde(rename = "qwen-image-3-pro")]
  QwenImage3Pro,
  #[serde(rename = "venice-sd35")]
  VeniceSd35,
  #[serde(rename = "z-image-turbo")]
  ZImageTurbo,
  // Generic Midjourney experience, served via the native Midjourney provider.
  #[serde(rename = "midjourney")]
  Midjourney,

  // Versioned Midjourney models, served by storyteller-web via the
  // Artcraft provider (dispatched through artcraft_router).
  #[serde(rename = "midjourney_7")]
  Midjourney7,
  #[serde(rename = "midjourney_7_niji")]
  Midjourney7Niji,
  #[serde(rename = "midjourney_8")]
  Midjourney8,

  // Image edit models

  #[serde(rename = "flux_pro_kontext_max")]
  FluxProKontextMax,
  #[serde(rename = "qwen_edit_2511_angles")]
  QwenEdit2511Angles,
  #[serde(rename = "flux_2_lora_angles")]
  Flux2LoraAngles,

  // Inpainting models

  #[serde(rename = "flux_dev_juggernaut")]
  FluxDevJuggernaut,
  #[serde(rename = "flux_pro_1")]
  FluxPro1,
}
