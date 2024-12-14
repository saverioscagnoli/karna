use rodio::{source::Buffered, Decoder, OutputStream, OutputStreamHandle, Source};
use std::{collections::HashMap, io::Cursor, path::Path};

pub struct Audio {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    audios: HashMap<String, Buffered<Decoder<Cursor<Vec<u8>>>>>,
}

impl Audio {
    pub(crate) fn new() -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();

        Self {
            _stream,
            handle,
            audios: HashMap::new(),
        }
    }

    pub fn load<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P) {
        let cursor =
            std::io::Cursor::new(std::fs::read(path.as_ref()).expect("Failed to read audio file"));

        let source = Decoder::new(cursor)
            .expect("Failed to decode audio file")
            .buffered();

        self.audios.insert(label.to_string(), source);
    }

    pub fn play<L: ToString>(&self, label: L) {
        let source = self
            .audios
            .get(&label.to_string())
            .expect("Audio not found");

        self.handle
            .play_raw(source.clone().convert_samples())
            .unwrap();
    }
}
