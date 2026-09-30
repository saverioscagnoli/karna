use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use sdl3::audio::AudioDevice;
use sdl3::audio::AudioStream;
use traccia::warn;

use crate::assets::Audio;
use crate::commands::AudioCommand;
use crate::commands::Outbox;

pub struct AudioSystem {
    device: AudioDevice,
    voices: Vec<AudioStream>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let Ok(device) = AudioDevice::open_default() else {
            panic!("Failed to open audio device");
        };

        Self {
            device,
            voices: Vec::new(),
        }
    }

    pub(crate) fn play(&mut self, audio: &Audio) {
        let stream = match AudioStream::convert(Some(audio.spec), None) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to play audio: {}", e);
                return;
            }
        };

        if let Err(e) = self.device.bind(&stream) {
            warn!("Failed to play audio: {}", e);
            return;
        }

        if let Err(e) = stream.put(&audio.pcm) {
            warn!("Failed to play audio: {}", e);
            return;
        }

        if let Err(e) = stream.flush() {
            warn!("Failed to play audio: {}", e);
            return;
        }

        self.voices.push(stream);
    }

    pub(crate) fn flush(&mut self) {
        self.voices.retain(|v| v.queued_bytes() > 0);
    }
}

pub struct AudioHandle<'a> {
    pub(crate) system: &'a AudioSystem,
    pub(crate) outbox: &'a mut Outbox<AudioCommand>,
}

impl<'a> AudioHandle<'a> {
    pub fn play(&mut self, audio: Handle<Audio>) {
        self.outbox.push(AudioCommand::Play { audio });
    }
}
