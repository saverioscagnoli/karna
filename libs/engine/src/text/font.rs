use nostd::alloc::string::String;

pub struct Font {
    pub(crate) family: String,
    pub(crate) size: f32,
}

impl Font {
    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn size(&self) -> f32 {
        self.size
    }
}
