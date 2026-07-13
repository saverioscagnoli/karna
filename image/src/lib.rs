mod err;
mod inflate;

pub use crate::err::ImageError;
use crate::inflate::zlib_decompress;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const PNG_MIN_CHUNK_LEN: usize = 12;
const PNG_HEADER_LEN: usize = 13;

// Adam7 lattice: starting offset and step for each of the seven passes.
const X0: [u32; 7] = [0, 4, 0, 2, 0, 1, 0];
const Y0: [u32; 7] = [0, 0, 4, 0, 2, 0, 1];
const DX: [u32; 7] = [8, 8, 4, 4, 2, 2, 1];
const DY: [u32; 7] = [8, 8, 8, 4, 4, 2, 2];

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression: u8,
    pub filter: u8,
    pub interlace: u8,
}

impl Header {
    fn from_chunk(chunk: &Chunk) -> Result<Self, ImageError> {
        if &chunk.ctype != b"IHDR" || chunk.data.len() != PNG_HEADER_LEN {
            return Err(ImageError::PngHeaderInvalid);
        }

        let d = chunk.data;
        let h = Self {
            width: u32::from_be_bytes([d[0], d[1], d[2], d[3]]),
            height: u32::from_be_bytes([d[4], d[5], d[6], d[7]]),
            bit_depth: d[8],
            color_type: d[9],
            compression: d[10],
            filter: d[11],
            interlace: d[12],
        };

        if h.width == 0 || h.height == 0 {
            return Err(ImageError::PngHeaderInvalid);
        }

        if h.compression != 0 || h.filter != 0 || h.interlace > 1 {
            return Err(ImageError::PngHeaderInvalid);
        }

        if h.channels() == 0 {
            return Err(ImageError::PngUnsupported(format!(
                "color type {}",
                h.color_type
            )));
        }

        if !matches!(h.bit_depth, 1 | 2 | 4 | 8 | 16) {
            return Err(ImageError::PngUnsupported(format!(
                "bit depth {}",
                h.bit_depth
            )));
        }

        Ok(h)
    }

    pub fn channels(&self) -> usize {
        match self.color_type {
            0 | 3 => 1, // grayscale, palette index
            4 => 2,     // gray + alpha
            2 => 3,     // RGB
            6 => 4,     // RGBA
            _ => 0,
        }
    }

    fn bits_per_pixel(&self) -> usize {
        self.channels() * self.bit_depth as usize
    }

    /// Bytes in one scanline of `w` pixels (excluding the filter byte).
    fn line_bytes(&self, w: u32) -> usize {
        (w as usize * self.bits_per_pixel() + 7) / 8
    }

    /// The filter's "left neighbour" offset, in bytes. Minimum 1.
    fn filter_stride(&self) -> usize {
        ((self.bits_per_pixel() + 7) / 8).max(1)
    }

    /// Pass geometry for Adam7. Returns (width, height) of pass `p`.
    fn pass_size(&self, p: usize) -> (u32, u32) {
        let pw = (self.width + DX[p] - 1 - X0[p]) / DX[p];
        let ph = (self.height + DY[p] - 1 - Y0[p]) / DY[p];
        (pw, ph)
    }

    /// Exactly how many bytes the inflated IDAT stream must be.
    /// Compute this BEFORE decompressing: it's both a memory bound and a
    /// free integrity check.
    pub fn expected_raw_len(&self) -> usize {
        if self.interlace == 0 {
            self.height as usize * (1 + self.line_bytes(self.width))
        } else {
            (0..7)
                .map(|p| {
                    let (pw, ph) = self.pass_size(p);
                    if pw == 0 || ph == 0 {
                        0
                    } else {
                        ph as usize * (1 + self.line_bytes(pw))
                    }
                })
                .sum()
        }
    }
}

// ---------------------------------------------------------------------------

pub struct Chunk<'a> {
    pub ctype: [u8; 4],
    pub data: &'a [u8],
}

impl Chunk<'_> {
    pub fn is_critical(&self) -> bool {
        self.ctype[0].is_ascii_uppercase()
    }
}

pub struct ChunkReader<'a> {
    bytes: &'a [u8],
    pos: usize,
    done: bool,
}

impl<'a> ChunkReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            done: false,
        }
    }
}

impl<'a> Iterator for ChunkReader<'a> {
    type Item = Result<Chunk<'a>, ImageError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if self.pos + PNG_MIN_CHUNK_LEN > self.bytes.len() {
            self.done = true;
            // Ran out of bytes without ever seeing IEND.
            return Some(Err(ImageError::PngTruncated));
        }

        let length = u32::from_be_bytes([
            self.bytes[self.pos],
            self.bytes[self.pos + 1],
            self.bytes[self.pos + 2],
            self.bytes[self.pos + 3],
        ]) as usize;

        let ctype = [
            self.bytes[self.pos + 4],
            self.bytes[self.pos + 5],
            self.bytes[self.pos + 6],
            self.bytes[self.pos + 7],
        ];

        let data_start = self.pos + 8;
        let data_end = data_start + length;

        // Re-check bounds now that we know it, or the slice below panics.
        if data_end + 4 > self.bytes.len() {
            self.done = true;
            return Some(Err(ImageError::PngTruncated));
        }

        let data = &self.bytes[data_start..data_end];
        self.pos = data_end + 4; // TODO: verify the 4-byte CRC here

        if &ctype == b"IEND" {
            self.done = true;
        }
        Some(Ok(Chunk { ctype, data }))
    }
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    /// Row-major RGBA8, 4 bytes per pixel
    pub pixels: Vec<u8>,
}

pub fn decode_png(raw: &[u8]) -> Result<Image, ImageError> {
    if raw.len() < 8 || raw[..8] != PNG_SIGNATURE {
        return Err(ImageError::NotAPng);
    }

    let mut reader = ChunkReader::new(&raw[8..]);

    // --- Step 1: IHDR is always the first chunk. Parse it before anything else,
    // because everything downstream depends on it.
    let first = reader.next().ok_or(ImageError::PngTruncated)??;
    let header = Header::from_chunk(&first)?;

    // --- Step 2: stream the IDATs straight into the decoder as we walk.
    // We know the exact output size from IHDR, so preallocate it.
    let expected = header.expected_raw_len();
    let mut idat: Vec<u8> = Vec::new();

    let mut palette: Vec<[u8; 3]> = Vec::new();
    let mut trns: Option<Vec<u8>> = None;
    let mut saw_idat = false;

    for chunk in reader {
        let chunk = chunk?;
        match &chunk.ctype {
            b"IHDR" => return Err(ImageError::PngHeaderInvalid), // duplicate
            b"PLTE" => {
                palette = chunk
                    .data
                    .chunks_exact(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect();
            }
            b"tRNS" => trns = Some(chunk.data.to_vec()),
            b"IDAT" => {
                saw_idat = true;
                idat.extend_from_slice(chunk.data);
            }
            b"IEND" => break,
            // An unknown chunk we don't understand: only safe to skip if the
            // first letter is lowercase (ancillary).
            other if !chunk.is_critical() => {
                let _ = other;
            }
            other => return Err(ImageError::PngUnknownCriticalChunk(*other)),
        }
    }

    if !saw_idat {
        return Err(ImageError::PngNoImageData);
    }
    if header.color_type == 3 && palette.is_empty() {
        return Err(ImageError::PngMissingPalette);
    }

    // `expected` doubles as the output limit: a stream that inflates larger
    // than IHDR says it should is malformed, so stop rather than allocate.
    let data = zlib_decompress(&idat, expected)?;

    // --- Step 3: the size check that would have caught your interlace bug
    // immediately, instead of 200 lines downstream.
    if data.len() != expected {
        return Err(ImageError::PngSizeMismatch {
            expected,
            got: data.len(),
        });
    }

    // --- Step 4: unfilter, then unpack to RGBA.
    let pal = build_palette(&header, palette, &trns);
    let mut pixels = vec![0u8; header.width as usize * header.height as usize * 4];

    if header.interlace == 0 {
        let stride = header.line_bytes(header.width);
        let lines = unfilter(&header, &data, header.height as usize, stride)?;
        for y in 0..header.height as usize {
            let line = &lines[y * stride..(y + 1) * stride];
            for x in 0..header.width as usize {
                let px = to_rgba(line, x, &header, &pal, &trns)?;
                let o = (y * header.width as usize + x) * 4;
                pixels[o..o + 4].copy_from_slice(&px);
            }
        }
    } else {
        // Adam7: the SAME unfilter routine, run seven times on sub-rectangles.
        // Each pass has its own stride and its own implicit zero row above it.
        let mut off = 0;
        for p in 0..7 {
            let (pw, ph) = header.pass_size(p);
            if pw == 0 || ph == 0 {
                continue;
            }
            let stride = header.line_bytes(pw);
            let take = ph as usize * (1 + stride);
            let lines = unfilter(&header, &data[off..off + take], ph as usize, stride)?;
            off += take;

            for row in 0..ph as usize {
                let line = &lines[row * stride..(row + 1) * stride];
                for col in 0..pw as usize {
                    let px = to_rgba(line, col, &header, &pal, &trns)?;
                    // Scatter into the real grid.
                    let x = X0[p] as usize + col * DX[p] as usize;
                    let y = Y0[p] as usize + row * DY[p] as usize;
                    let o = (y * header.width as usize + x) * 4;
                    pixels[o..o + 4].copy_from_slice(&px);
                }
            }
        }
    }

    Ok(Image {
        width: header.width,
        height: header.height,
        pixels,
    })
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let (pa, pb, pc) = (
        (p - a as i16).abs(),
        (p - b as i16).abs(),
        (p - c as i16).abs(),
    );

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn unfilter(h: &Header, data: &[u8], rows: usize, stride: usize) -> Result<Vec<u8>, ImageError> {
    let fs = h.filter_stride();
    let mut out = vec![0u8; rows * stride];

    for y in 0..rows {
        let ft = data[y * (stride + 1)];
        let src = &data[y * (stride + 1) + 1..(y + 1) * (stride + 1)];

        for x in 0..stride {
            let a = if x >= fs { out[y * stride + x - fs] } else { 0 };
            let b = if y > 0 { out[(y - 1) * stride + x] } else { 0 };
            let c = if y > 0 && x >= fs {
                out[(y - 1) * stride + x - fs]
            } else {
                0
            };

            out[y * stride + x] = match ft {
                0 => src[x],
                1 => src[x].wrapping_add(a),
                2 => src[x].wrapping_add(b),
                3 => src[x].wrapping_add(((a as u16 + b as u16) / 2) as u8),
                4 => src[x].wrapping_add(paeth(a, b, c)),
                _ => return Err(ImageError::PngBadFilter(ft)),
            };
        }
    }

    Ok(out)
}

struct Palette {
    rgb: Vec<[u8; 3]>,
    alpha: Vec<u8>,
}

fn build_palette(h: &Header, rgb: Vec<[u8; 3]>, trns: &Option<Vec<u8>>) -> Palette {
    let mut alpha = vec![255u8; rgb.len()];
    if h.color_type == 3 {
        if let Some(t) = trns {
            for (i, &a) in t.iter().take(rgb.len()).enumerate() {
                alpha[i] = a;
            }
        }
    }
    Palette { rgb, alpha }
}

fn sample(line: &[u8], i: usize, depth: u8) -> u16 {
    match depth {
        16 => ((line[i * 2] as u16) << 8) | line[i * 2 + 1] as u16,
        8 => line[i] as u16,
        d => {
            let bits = d as usize;
            let bit = i * bits;
            let shift = 8 - bits - (bit % 8);
            ((line[bit / 8] >> shift) & ((1u16 << bits) - 1) as u8) as u16
        }
    }
}

fn to_rgba(
    line: &[u8],
    i: usize,
    h: &Header,
    pal: &Palette,
    trns: &Option<Vec<u8>>,
) -> Result<[u8; 4], ImageError> {
    let ch = h.channels();
    let max = ((1u32 << h.bit_depth) - 1) as u16;
    let scale = |v: u16| -> u8 {
        if h.bit_depth == 16 {
            (v >> 8) as u8
        } else {
            ((v as u32 * 255 + max as u32 / 2) / max as u32) as u8
        }
    };

    let s = |c: usize| sample(line, i * ch + c, h.bit_depth);

    Ok(match h.color_type {
        0 => {
            let g = s(0);
            let opaque = match trns {
                Some(t) if t.len() >= 2 => g != ((t[0] as u16) << 8 | t[1] as u16),
                _ => true,
            };
            let v = scale(g);
            [v, v, v, if opaque { 255 } else { 0 }]
        }

        2 => {
            let (r, g, b) = (s(0), s(1), s(2));
            let opaque = match trns {
                Some(t) if t.len() >= 6 => {
                    let k = |n: usize| (t[n] as u16) << 8 | t[n + 1] as u16;
                    !(r == k(0) && g == k(2) && b == k(4))
                }
                _ => true,
            };
            [scale(r), scale(g), scale(b), if opaque { 255 } else { 0 }]
        }

        3 => {
            let idx = s(0) as usize;
            let c = *pal
                .rgb
                .get(idx)
                .ok_or_else(|| ImageError::PngUnsupported("palette index out of range".into()))?;
            [c[0], c[1], c[2], *pal.alpha.get(idx).unwrap_or(&255)]
        }

        4 => {
            let v = scale(s(0));
            [v, v, v, scale(s(1))]
        }

        6 => [scale(s(0)), scale(s(1)), scale(s(2)), scale(s(3))],

        c => return Err(ImageError::PngUnsupported(format!("color type {}", c))),
    })
}
