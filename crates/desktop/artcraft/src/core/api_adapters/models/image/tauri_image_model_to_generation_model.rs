use crate::core::commands::generate::generate_image::tauri_image_model::TauriImageModel;
use crate::core::events::generation_events::common::GenerationModel;

/// Map TauriImageModel to GenerationModel for frontend events.
pub fn tauri_image_model_to_generation_model(model: TauriImageModel) -> GenerationModel {
  match model {
    TauriImageModel::Flux1Dev => GenerationModel::Flux1Dev,
    TauriImageModel::Flux1Schnell => GenerationModel::Flux1Schnell,
    TauriImageModel::FluxPro11 => GenerationModel::FluxPro11,
    TauriImageModel::FluxPro11Ultra => GenerationModel::FluxPro11Ultra,
    TauriImageModel::GrokImage => GenerationModel::GrokImage,
    TauriImageModel::Recraft3 => GenerationModel::Flux1Dev, // Fallback
    TauriImageModel::GptImage1 => GenerationModel::GptImage1,
    TauriImageModel::GptImage1p5 => GenerationModel::GptImage1p5,
    TauriImageModel::GptImage2 => GenerationModel::GptImage2,
    TauriImageModel::GptImage2p5Flare => GenerationModel::GptImage2p5Flare,
    TauriImageModel::GptImage2p5Sunburst => GenerationModel::GptImage2p5Sunburst,
    TauriImageModel::Gemini25Flash => GenerationModel::NanoBanana,
    TauriImageModel::NanoBanana => GenerationModel::NanoBanana,
    TauriImageModel::NanoBanana2 => GenerationModel::NanoBanana2,
    TauriImageModel::NanoBananaPro => GenerationModel::NanoBananaPro,
    TauriImageModel::Seedream4 => GenerationModel::Seedream4,
    TauriImageModel::Seedream4p5 => GenerationModel::Seedream4p5,
    TauriImageModel::Seedream5Lite => GenerationModel::Seedream5Lite,
    TauriImageModel::Midjourney => GenerationModel::Midjourney,
    TauriImageModel::Midjourney7 => GenerationModel::Midjourney7,
    TauriImageModel::Midjourney7Niji => GenerationModel::Midjourney7Niji,
    TauriImageModel::Midjourney8 => GenerationModel::Midjourney8,
    TauriImageModel::FluxProKontextMax => GenerationModel::FluxProKontextMax,
    TauriImageModel::QwenEdit2511Angles => GenerationModel::QwenEdit2511Angles,
    TauriImageModel::Flux2LoraAngles => GenerationModel::Flux2LoraAngles,
    TauriImageModel::FluxDevJuggernaut => GenerationModel::FluxDevJuggernaut,
    TauriImageModel::FluxPro1 => GenerationModel::FluxPro1,
    // Venice-only models carry their Venice model id through events/tasks.
    TauriImageModel::Flux2Pro => GenerationModel::Unknown("flux-2-pro".to_string()),
    TauriImageModel::Flux2Max => GenerationModel::Unknown("flux-2-max".to_string()),
    TauriImageModel::Flux3Image => GenerationModel::Unknown("flux-3-image".to_string()),
    TauriImageModel::Krea2Turbo => GenerationModel::Unknown("krea-2-turbo".to_string()),
    TauriImageModel::Ideogram4p5 => GenerationModel::Unknown("ideogram-v4-5".to_string()),
    TauriImageModel::Recraft4 => GenerationModel::Unknown("recraft-v4".to_string()),
    TauriImageModel::LumaUni1Max => GenerationModel::Unknown("luma-uni-1-max".to_string()),
    TauriImageModel::MuseImage => GenerationModel::Unknown("muse-image".to_string()),
    TauriImageModel::NanoBanana2p1 => GenerationModel::Unknown("nano-banana-2-1".to_string()),
    TauriImageModel::Seedream5Pro => GenerationModel::Unknown("seedream-v5-pro".to_string()),
    TauriImageModel::QwenImage3Pro => GenerationModel::Unknown("qwen-image-3-pro".to_string()),
    TauriImageModel::VeniceSd35 => GenerationModel::Unknown("venice-sd35".to_string()),
    TauriImageModel::ZImageTurbo => GenerationModel::Unknown("z-image-turbo".to_string()),
  }
}
