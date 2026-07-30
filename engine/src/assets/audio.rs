use std::fs;
use std::path::Path;
use std::time::Duration;

use sdl3::mixer::Audio as MixAudio;
use sdl3::mixer::MIX_DURATION_INFINITE;

use crate::sound::SharedMixer;

pub fn decode_audio(mixer: &SharedMixer, path: &Path) -> Result<Audio, String> {
    fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|b| mixer.load(&b).map_err(|e| e.to_string()))
        .map(Audio)
}

pub fn decode_audio_bytes(mixer: &SharedMixer, bytes: &[u8]) -> Result<Audio, String> {
    mixer.load(bytes).map_err(|e| e.to_string()).map(Audio)
}

#[derive(Debug, Clone, Copy)]
pub enum AudioLength {
    Finite(Duration),
    Infinite,
    Unknown,
}

pub struct Audio(pub(crate) MixAudio);

unsafe impl Send for Audio {}

impl Audio {
    pub fn length(&self) -> AudioLength {
        let frames = self.0.duration();

        if frames == MIX_DURATION_INFINITE as i64 {
            return AudioLength::Infinite;
        }

        if frames < 0 {
            return AudioLength::Unknown;
        }

        let Ok(spec) = self.0.format() else {
            return AudioLength::Unknown;
        };

        if spec.freq <= 0 {
            return AudioLength::Unknown;
        }

        AudioLength::Finite(Duration::from_secs_f64(frames as f64 / spec.freq as f64))
    }
}
