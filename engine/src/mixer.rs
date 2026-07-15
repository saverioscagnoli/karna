use assets::AssetsReader;
use assets::Audio;
use utils::Handle;

#[cfg(not(target_arch = "wasm32"))]
use rodio::DeviceSinkBuilder;

#[cfg(not(target_arch = "wasm32"))]
pub struct Mixer {
    handle: rodio::MixerDeviceSink,
    assets: AssetsReader,
}

#[cfg(not(target_arch = "wasm32"))]
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

/// Audio is not wired up on the web yet (rodio's cpal backend needs
/// WebAudio plumbing and a user gesture to start). Playback is a no-op.
#[cfg(target_arch = "wasm32")]
pub struct Mixer {
    #[allow(unused)]
    assets: AssetsReader,
}

#[cfg(target_arch = "wasm32")]
impl Mixer {
    pub(crate) fn new(assets: AssetsReader) -> Self {
        Self { assets }
    }

    pub fn play(&self, _audio: Handle<Audio>) {
        logging::warn!("Audio playback is not supported on the web yet");
    }
}
