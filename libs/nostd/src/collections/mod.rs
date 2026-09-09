use core::hash::BuildHasherDefault;
use core::hash::Hasher;

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Default)]
pub struct FxHasher(u64);

impl FxHasher {
    #[inline]
    fn add(&mut self, i: u64) {
        self.0 = (self.0.rotate_left(5) ^ i).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut buf = [0u8; 8];
            buf[..chunk.len()].copy_from_slice(chunk);
            self.add(u64::from_le_bytes(buf));
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i))
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(u64::from(i))
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(u64::from(i))
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i)
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64)
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

pub type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;
