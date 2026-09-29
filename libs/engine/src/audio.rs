use sdl3::audio::AudioDevice;

pub struct AudioSystem {
    device: AudioDevice,
}

impl AudioSystem {
    pub fn new() -> Self {
        let Ok(device) = AudioDevice::open_default() else {
            panic!("Failed to open audio device");
        };

        Self { device }
    }
}
