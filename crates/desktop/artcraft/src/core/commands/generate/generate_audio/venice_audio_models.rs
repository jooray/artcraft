/// Venice audio models the desktop app can dispatch to (music and sound
/// effects; TTS models are a separate flow and are not offered here).
///
/// The frontend offers these model ids directly in the audio model list.
pub fn venice_supports_audio_model_id(model_id: &str) -> bool {
  venice_audio_model_spec(model_id).is_some()
}

/// Lyrics capabilities for a Venice audio model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VeniceAudioModelSpec {
  /// Accepts a `lyrics_prompt` (custom lyrics).
  pub supports_lyrics: bool,
  /// Lyrics are mandatory (no auto-written lyrics).
  pub lyrics_required: bool,
  /// Can auto-write lyrics from the prompt when none are supplied.
  pub supports_lyrics_optimizer: bool,
}

/// Mirrors `GET /models?type=music` capability flags (October 2026).
pub fn venice_audio_model_spec(model_id: &str) -> Option<VeniceAudioModelSpec> {
  let (supports_lyrics, lyrics_required, supports_lyrics_optimizer) = match model_id {
    "ace-step-15" => (true, false, false),
    "minimax-music-v2" => (true, true, false),
    "minimax-music-v25" => (true, false, true),
    "minimax-music-v26" => (true, false, false),
    "elevenlabs-music"
    | "elevenlabs-music-v2"
    | "elevenlabs-music-v2-5"
    | "lyria-3-pro"
    | "stable-audio-25"
    | "sonilo-v1-1-music"
    | "sonilo-v1-1-sound-effects"
    | "elevenlabs-sound-effects-v2"
    | "mmaudio-v2-text-to-audio"
    | "seed-audio-1-0" => (false, false, false),
    _ => return None,
  };

  Some(VeniceAudioModelSpec { supports_lyrics, lyrics_required, supports_lyrics_optimizer })
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

  #[test]
  fn lyrics_capabilities_match_the_api() {
    let minimax_v26 = venice_audio_model_spec("minimax-music-v26").unwrap();
    assert!(minimax_v26.supports_lyrics);
    assert!(!minimax_v26.lyrics_required);
    assert!(!minimax_v26.supports_lyrics_optimizer);

    let minimax_v2 = venice_audio_model_spec("minimax-music-v2").unwrap();
    assert!(minimax_v2.lyrics_required);

    let minimax_v25 = venice_audio_model_spec("minimax-music-v25").unwrap();
    assert!(minimax_v25.supports_lyrics_optimizer);

    let sound_effects = venice_audio_model_spec("elevenlabs-sound-effects-v2").unwrap();
    assert!(!sound_effects.supports_lyrics);
  }
}
