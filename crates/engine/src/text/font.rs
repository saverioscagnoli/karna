#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Font {
    pub(crate) family: String,
}

impl Font {
    pub fn family(&self) -> &str {
        &self.family
    }
}
