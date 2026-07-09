use assets::Audio;

pub struct Sound {
    handle: rodio::MixerDeviceSink,
}

impl Sound {
    pub(crate) fn new() -> Self {
        let mut handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");

        handle.log_on_drop(false);

        Self { handle }
    }

    pub fn play(&self, audio: &Audio) {
        self.handle.mixer().add(audio.source.clone());
    }
}
