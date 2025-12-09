use common::utils::Label;
use rodio::{Decoder, Source};
use std::io::{BufReader, Cursor};
use traccia::{info, warn};
use wgpu::naga::FastHashMap;

pub struct Audio {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
    assets: FastHashMap<Label, &'static [u8]>,
}

impl Audio {
    pub(crate) fn new() -> Self {
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .expect("Failed to get default stream");
        let sink = rodio::Sink::connect_new(stream.mixer());

        Self {
            _stream: stream,
            sink,
            assets: FastHashMap::default(),
        }
    }

    #[inline]
    pub fn load_from_bytes(&mut self, label: Label, data: &'static [u8]) {
        info!("Loading sound with id '{}'", label.raw());
        self.assets.insert(label, data);
    }

    pub fn play(&self, label: Label) {
        if let Some(data) = self.assets.get(&label) {
            let cursor = Cursor::new(*data);

            match Decoder::new(BufReader::new(cursor)) {
                Ok(source) => {
                    self.sink.append(source);
                }

                Err(e) => {
                    warn!("Failed to play sound: id={} {}", label.raw(), e);
                }
            }
        } else {
            warn!("Failed to play sound: id={} not found", label.raw());
        }
    }
}
