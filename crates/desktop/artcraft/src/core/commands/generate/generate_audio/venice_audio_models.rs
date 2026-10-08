/// Venice audio models the desktop app can dispatch to (music and sound
/// effects; TTS models are a separate flow and are not offered here).
///
/// The frontend offers these model ids directly in the audio model list.
pub fn venice_supports_audio_model_id(model_id: &str) -> bool {
  matches!(
    model_id,
    "ace-step-15"
      | "elevenlabs-music"
      | "elevenlabs-music-v2"
      | "elevenlabs-music-v2-5"
      | "minimax-music-v2"
      | "minimax-music-v25"
      | "minimax-music-v26"
      | "lyria-3-pro"
      | "stable-audio-25"
      | "sonilo-v1-1-music"
      | "sonilo-v1-1-sound-effects"
      | "elevenlabs-sound-effects-v2"
      | "mmaudio-v2-text-to-audio"
      | "seed-audio-1-0"
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_known_music_and_sound_models() {
    for id in ["ace-step-15", "minimax-music-v26", "lyria-3-pro", "sonilo-v1-1-sound-effects", "seed-audio-1-0"] {
      assert!(venice_supports_audio_model_id(id), "{} should be supported", id);
    }
  }

  #[test]
  fn rejects_backend_and_tts_models() {
    for id in ["suno_music", "seed_audio_1p0", "elevenlabs-tts-v4", "nope"] {
      assert!(!venice_supports_audio_model_id(id), "{} should not be supported", id);
    }
  }
}
