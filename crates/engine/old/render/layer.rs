/// Nominative for render layers
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Layer(u64);

#[allow(non_upper_case_globals)]
impl Layer {
    pub const World: Self = Self::new_label("world");
    pub const Ui: Self = Self::new_label("ui");
    pub const Debug: Self = Self::new_label("debug");

    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn new_label(label: &'static str) -> Self {
        Self(utils::fnv1a(label.as_bytes()))
    }
}

pub struct RenderLayer {}
