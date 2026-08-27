use std::ops::Index;
use std::ops::IndexMut;

use utils::BitSet;

use crate::Key;
use crate::events::MouseButton;

#[derive(Default)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeySet([u64; 8]);

impl Index<usize> for KeySet {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for KeySet {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl KeySet {
    fn split(k: Key) -> (usize, u64) {
        let i = (k as usize).min(511);
        (i / 64, 1u64 << (i % 64))
    }
}

impl BitSet for KeySet {
    type Item = Key;

    fn insert(&mut self, k: Key) {
        let (w, b) = Self::split(k);
        self[w] |= b;
    }

    fn remove(&mut self, k: Key) {
        let (w, b) = Self::split(k);
        self[w] &= !b;
    }

    fn contains(&self, k: Key) -> bool {
        let (w, b) = Self::split(k);
        self[w] & b != 0
    }

    fn clear(&mut self) {
        self.0 = [0; 8];
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MouseSet(u8);

impl BitSet for MouseSet {
    type Item = MouseButton;

    fn insert(&mut self, btn: MouseButton) {
        self.0 |= btn.mask();
    }

    fn remove(&mut self, btn: MouseButton) {
        self.0 &= !btn.mask();
    }

    fn contains(&self, btn: MouseButton) -> bool {
        self.0 & btn.mask() != 0
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }
}
