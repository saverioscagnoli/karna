use std::hash::{BuildHasher, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(u32);

impl Label {
    const FNV_OFFSET: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    /// Create a label from a string slice at compile time
    pub const fn new(s: &str) -> Self {
        Self(Self::hash(s))
    }

    /// Get the raw hash value
    pub const fn raw(&self) -> u32 {
        self.0
    }

    pub const fn hash(s: &str) -> u32 {
        let bytes = s.as_bytes();
        let mut hash = Self::FNV_OFFSET;
        let mut i = 0;

        while i < bytes.len() {
            hash ^= bytes[i] as u32;
            hash = hash.wrapping_mul(Self::FNV_PRIME);
            i += 1;
        }

        hash
    }
}

#[macro_export]
macro_rules! label {
    ($s:literal) => {{
        const LABEL: $crate::map::Label = $crate::map::Label::new($s);
        LABEL
    }};
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

pub type LabelMap<V> = hashbrown::HashMap<Label, V, IdentityHasherBuilder>;
