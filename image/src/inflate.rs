//! DEFLATE (RFC 1951) and zlib (RFC 1950) decompression.
//!
//! Enough to replace flate2 for PNG. No streaming API: PNG hands you the whole
//! stream anyway once you've concatenated the IDATs.

use crate::err::ImageError;

type Res<T> = Result<T, ImageError>;

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    buf: u64,
    cnt: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader {
            data,
            pos: 0,
            buf: 0,
            cnt: 0,
        }
    }

    #[inline]
    fn refill(&mut self) {
        // Top up to at least 32 bits when we can do it cheaply.
        while self.cnt <= 56 {
            match self.data.get(self.pos) {
                Some(&b) => {
                    self.buf |= (b as u64) << self.cnt;
                    self.cnt += 8;
                    self.pos += 1;
                }
                None => break,
            }
        }
    }

    #[inline]
    fn bits(&mut self, n: u32) -> Res<u32> {
        if n == 0 {
            return Ok(0);
        }

        if self.cnt < n {
            self.refill();

            if self.cnt < n {
                return Err(ImageError::Deflate("unexpected end of stream"));
            }
        }

        let v = (self.buf & ((1u64 << n) - 1)) as u32;

        self.buf >>= n;
        self.cnt -= n;

        Ok(v)
    }

    /// Drop bits up to the next byte boundary (stored blocks are byte-aligned).
    fn align(&mut self) {
        let drop = self.cnt % 8;

        self.buf >>= drop;
        self.cnt -= drop;
    }

    fn byte(&mut self) -> Res<u8> {
        if self.cnt >= 8 {
            let v = (self.buf & 0xff) as u8;

            self.buf >>= 8;
            self.cnt -= 8;

            return Ok(v);
        }

        match self.data.get(self.pos) {
            Some(&b) => {
                self.pos += 1;
                Ok(b)
            }
            None => Err(ImageError::Deflate("unexpected end of stream")),
        }
    }
}
struct Huffman {
    counts: [u16; MAX_BITS + 1],
    symbols: Vec<u16>,
}

const MAX_BITS: usize = 15;

impl Huffman {
    fn new(lengths: &[u8]) -> Res<Huffman> {
        let mut counts = [0u16; MAX_BITS + 1];

        for &l in lengths {
            if l as usize > MAX_BITS {
                return Err(ImageError::Deflate("code length exceeds 15"));
            }

            counts[l as usize] += 1;
        }

        counts[0] = 0; // length 0 means "symbol unused"

        let mut left = 1i32;

        for len in 1..=MAX_BITS {
            left <<= 1;
            left -= counts[len] as i32;

            if left < 0 {
                return Err(ImageError::Deflate("over-subscribed huffman code"));
            }
        }

        let mut offs = [0u16; MAX_BITS + 2];

        for len in 1..=MAX_BITS {
            offs[len + 1] = offs[len] + counts[len];
        }

        let mut symbols = vec![0u16; lengths.len()];

        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offs[l as usize] as usize] = sym as u16;
                offs[l as usize] += 1;
            }
        }

        Ok(Huffman { counts, symbols })
    }

    fn decode(&self, br: &mut BitReader) -> Res<u16> {
        let mut code = 0i32; // the bits seen so far, MSB-first
        let mut first = 0i32; // first canonical code of the current length
        let mut index = 0i32; // where that code's symbol lives in `symbols`

        for len in 1..=MAX_BITS {
            code |= br.bits(1)? as i32;
            let count = self.counts[len] as i32;

            if code - count < first {
                return Ok(self.symbols[(index + (code - first)) as usize]);
            }

            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }

        Err(ImageError::Deflate("invalid huffman code"))
    }
}

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];

const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];

const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

/// The order in which the 19 code-length code lengths are transmitted.
/// It's shuffled so the rarely-used ones land at the end and can be omitted.
const CLEN_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn fixed_tables() -> Res<(Huffman, Huffman)> {
    let mut lit = [0u8; 288];
    for (i, l) in lit.iter_mut().enumerate() {
        *l = match i {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }
    Ok((Huffman::new(&lit)?, Huffman::new(&[5u8; 30])?))
}

fn dynamic_tables(br: &mut BitReader) -> Res<(Huffman, Huffman)> {
    let hlit = br.bits(5)? as usize + 257;
    let hdist = br.bits(5)? as usize + 1;
    let hclen = br.bits(4)? as usize + 4;

    let mut clen = [0u8; 19];

    for i in 0..hclen {
        clen[CLEN_ORDER[i]] = br.bits(3)? as u8;
    }

    let clen_huff = Huffman::new(&clen)?;

    let total = hlit + hdist;
    let mut lengths = vec![0u8; total];
    let mut i = 0;

    while i < total {
        let sym = clen_huff.decode(br)?;
        let (value, run) = match sym {
            0..=15 => (sym as u8, 1),
            16 => {
                if i == 0 {
                    return Err(ImageError::Deflate("repeat with no previous length"));
                }

                (lengths[i - 1], 3 + br.bits(2)? as usize)
            }
            17 => (0, 3 + br.bits(3)? as usize),
            18 => (0, 11 + br.bits(7)? as usize),
            _ => return Err(ImageError::Deflate("bad code length symbol")),
        };

        if i + run > total {
            return Err(ImageError::Deflate("code length run overflows"));
        }

        for _ in 0..run {
            lengths[i] = value;
            i += 1;
        }
    }

    if lengths[256] == 0 {
        return Err(ImageError::Deflate("no end-of-block code"));
    }

    Ok((
        Huffman::new(&lengths[..hlit])?,
        Huffman::new(&lengths[hlit..])?,
    ))
}

fn inflate_block(
    br: &mut BitReader,
    out: &mut Vec<u8>,
    limit: usize,
    lit: &Huffman,
    dist: &Huffman,
) -> Res<()> {
    loop {
        let sym = lit.decode(br)?;

        match sym {
            0..=255 => {
                if out.len() >= limit {
                    return Err(ImageError::Deflate("output exceeds limit"));
                }

                out.push(sym as u8);
            }
            256 => return Ok(()), // end of block
            257..=285 => {
                let i = sym as usize - 257;
                let len = LEN_BASE[i] as usize + br.bits(LEN_EXTRA[i] as u32)? as usize;
                let d = dist.decode(br)? as usize;

                if d >= 30 {
                    return Err(ImageError::Deflate("invalid distance symbol"));
                }

                let distance = DIST_BASE[d] as usize + br.bits(DIST_EXTRA[d] as u32)? as usize;

                if distance > out.len() {
                    return Err(ImageError::Deflate("distance before start of output"));
                }

                if out.len() + len > limit {
                    return Err(ImageError::Deflate("output exceeds limit"));
                }

                let start = out.len() - distance;

                for k in 0..len {
                    let b = out[start + k];
                    out.push(b);
                }
            }

            _ => return Err(ImageError::Deflate("invalid literal/length symbol")),
        }
    }
}

pub fn inflate(data: &[u8], limit: usize) -> Res<Vec<u8>> {
    let mut br = BitReader::new(data);
    let mut out = Vec::with_capacity(limit.min(1 << 20));

    loop {
        let is_final = br.bits(1)?;

        match br.bits(2)? {
            0 => {
                br.align();
                let len = br.byte()? as usize | ((br.byte()? as usize) << 8);
                let nlen = br.byte()? as usize | ((br.byte()? as usize) << 8);

                if len != (!nlen & 0xffff) {
                    return Err(ImageError::Deflate("stored block length check failed"));
                }

                if out.len() + len > limit {
                    return Err(ImageError::Deflate("output exceeds limit"));
                }

                for _ in 0..len {
                    let b = br.byte()?;
                    out.push(b);
                }
            }
            1 => {
                let (l, d) = fixed_tables()?;
                inflate_block(&mut br, &mut out, limit, &l, &d)?;
            }
            2 => {
                let (l, d) = dynamic_tables(&mut br)?;
                inflate_block(&mut br, &mut out, limit, &l, &d)?;
            }
            _ => return Err(ImageError::Deflate("reserved block type")),
        }

        if is_final == 1 {
            return Ok(out);
        }
    }
}

pub fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);

    for chunk in data.chunks(5552) {
        for &byte in chunk {
            a += byte as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }

    (b << 16) | a
}

pub fn zlib_decompress(data: &[u8], limit: usize) -> Res<Vec<u8>> {
    if data.len() < 6 {
        return Err(ImageError::Deflate("zlib stream too short"));
    }

    let (cmf, flg) = (data[0], data[1]);
    if cmf & 0x0f != 8 {
        return Err(ImageError::Deflate("not deflate compression"));
    }

    // The two header bytes as a big-endian u16 must be a multiple of 31.
    if ((cmf as u16) << 8 | flg as u16) % 31 != 0 {
        return Err(ImageError::Deflate("bad zlib header check"));
    }

    if flg & 0x20 != 0 {
        return Err(ImageError::Deflate("preset dictionary unsupported"));
    }

    let out = inflate(&data[2..data.len() - 4], limit)?;
    let t = &data[data.len() - 4..];
    let expect = u32::from_be_bytes([t[0], t[1], t[2], t[3]]);

    if adler32(&out) != expect {
        return Err(ImageError::ChecksumMismatch);
    }

    Ok(out)
}
