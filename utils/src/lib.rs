mod index_map;
mod macros;
mod sleep;

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

pub use index_map::*;
pub use sleep::*;

pub struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {
        panic!("IdentityHasher only works with u32")
    }

    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }
}

#[derive(Default)]
pub struct IdentityHasherBuilder;

impl BuildHasher for IdentityHasherBuilder {
    type Hasher = IdentityHasher;

    fn build_hasher(&self) -> Self::Hasher {
        IdentityHasher(0)
    }
}

pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;
