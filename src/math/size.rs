use super::ToU32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new<U: ToU32>(width: U, height: U) -> Self {
        Self {
            width: width.to_u32(),
            height: height.to_u32(),
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

impl<T: ToU32> From<(T, T)> for Size {
    fn from((width, height): (T, T)) -> Self {
        Self {
            width: width.to_u32(),
            height: height.to_u32(),
        }
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}
