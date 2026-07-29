use sdl3::iostream::IOStream;
use sdl3::mixer::Audio as MixAudio;
use sdl3::mixer::Mixer;

use crate::Audio;

pub struct SharedMixer {
    mixer: Mixer,
}

unsafe impl Send for SharedMixer {}
unsafe impl Sync for SharedMixer {}

impl SharedMixer {
    pub fn open() -> Result<Self, sdl3::Error> {
        Mixer::open_device(None).map(|m| Self { mixer: m })
    }

    pub fn load(&self, bytes: &[u8]) -> Result<MixAudio, sdl3::Error> {
        let io = IOStream::from_bytes(bytes)?;
        self.mixer.load_audio_io(&io, true)
    }

    pub fn play(&self, audio: &Audio) -> Result<(), String> {
        self.mixer.play_audio(&audio.0).map_err(|e| e.to_string())
    }

    pub fn set_master_volume(&self, vol: f32) -> Result<(), String> {
        self.mixer.set_gain(vol).map_err(|e| e.to_string())
    }
}
