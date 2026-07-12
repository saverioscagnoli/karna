mod byte_size;
mod index_map;
mod lazy;
mod macros;
mod mem;
pub mod profile;
mod sleep;
mod slot_map;
mod timer;

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

pub use crate::byte_size::*;
pub use crate::index_map::*;
pub use crate::lazy::*;
pub use crate::mem::*;
pub use crate::sleep::*;
pub use crate::slot_map::*;
pub use crate::timer::*;

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
pub type FastHashSet<V> = HashSet<V, BuildHasherDefault<rustc_hash::FxHasher>>;
