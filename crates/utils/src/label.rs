use crate::fnv1a;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(u64);

impl Label {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn new_str(str: &str) -> Self {
        Self(fnv1a(str.as_bytes()))
    }
}
