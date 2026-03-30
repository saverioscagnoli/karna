use std::io::Cursor;

use math::Size;

pub fn decode_png(bytes: &[u8]) -> (Vec<u8>, Size<u32>) {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);

    let mut reader = decoder.read_info().expect("Failed to read PNG info");
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).expect("Failed to decode PNG");
    buf.truncate(info.buffer_size());

    (buf, Size::new(info.width, info.height))
}
