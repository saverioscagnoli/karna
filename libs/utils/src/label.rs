use core::fmt;

use crate::fnv1a;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(u64);

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "label ({})", self.0)
    }
}

impl Label {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn new_str(str: &str) -> Self {
        Self(fnv1a(str.as_bytes()))
    }

    pub const fn inner(&self) -> u64 {
        self.0
    }
}
