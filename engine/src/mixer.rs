use assets::AssetsReader;
use assets::Audio;
use rodio::DeviceSinkBuilder;
use utils::Handle;

pub struct Mixer {
    handle: rodio::MixerDeviceSink,
    assets: AssetsReader,
}

impl Mixer {
    pub(crate) fn new(assets: AssetsReader) -> Self {
        let mut handle =
            DeviceSinkBuilder::open_default_sink().expect("Failed to open audio stream");

        handle.log_on_drop(false);

        Self { handle, assets }
    }

    pub fn play(&self, audio: Handle<Audio>) {
        let lock = self.assets.read();
        let audio = lock.get_audio(audio);

        // Rodio uses Arc<[f32]>
        // Cloning is negligible
        self.handle.mixer().add(audio.source.clone());
    }
}
