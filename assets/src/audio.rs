use std::io::Cursor;

use rodio::Decoder;
use rodio::Source;
use rodio::buffer::SamplesBuffer;
use utils::Handle;
use utils::SlotMap;

#[derive(Clone)]
pub struct Audio {
    pub source: rodio::buffer::SamplesBuffer,
}

#[derive(Clone)]
pub struct AudioRegistry {
    pub registry: SlotMap<Audio>,
}

impl AudioRegistry {
    pub fn new() -> Self {
        Self {
            registry: SlotMap::new(),
        }
    }

    pub fn load_audio(&mut self, bytes: &[u8]) -> Handle<Audio> {
        let cursor = Cursor::new(bytes.to_vec());

        let decoder = Decoder::new(cursor).unwrap();

        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();

        let samples: Vec<f32> = decoder.collect();

        let audio = Audio {
            source: SamplesBuffer::new(channels, sample_rate, samples),
        };

        self.registry.insert(audio)
    }
}
