use super::ToU32;

pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl<T> From<(T, T)> for Size
where
    T: ToU32,
{
    fn from((width, height): (T, T)) -> Self {
        Self::new(width.to_u32(), height.to_u32())
    }
}

impl Into<(u32, u32)> for Size {
    fn into(self) -> (u32, u32) {
        (self.width, self.height)
    }
}
