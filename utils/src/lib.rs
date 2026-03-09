mod lazy;
mod macros;
mod timer;

use std::{
    collections::HashMap,
    hash::{BuildHasher, BuildHasherDefault, Hasher},
    mem,
};

pub use lazy::*;
pub use timer::*;

pub fn as_u8_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe { ::core::slice::from_raw_parts(slice.as_ptr() as *const u8, mem::size_of_val(slice)) }
}

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
