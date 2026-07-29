use std::sync::Arc;

use logging::error;

use crate::Audio;
use crate::sound::SharedMixer;

pub struct AudioHandle {
    mixer: Arc<SharedMixer>,
}

impl AudioHandle {
    pub(crate) fn new(mixer: Arc<SharedMixer>) -> Self {
        Self { mixer }
    }

    pub fn set_master_volume(&self, vol: f32) {
        if let Err(e) = self.mixer.set_master_volume(vol) {
            error!("Failed to set master volume: {}", e);
        }
    }

    pub fn play(&self, audio: &Audio) {
        if let Err(e) = self.mixer.play(audio) {
            error!("Failed to play audio: {}", e);
        }
    }
}
