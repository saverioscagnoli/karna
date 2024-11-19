use crate::traits::ToU32;

#[derive(Debug, Clone, Copy)]
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

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn one() -> Self {
        Self::new(1, 1)
    }

    pub fn set<U: ToU32>(&mut self, width: U, height: U) {
        self.width = width.to_u32();
        self.height = height.to_u32();
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl<U: ToU32> From<(U, U)> for Size {
    fn from((width, height): (U, U)) -> Self {
        Self::new(width, height)
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.width.to_u32(), size.height.to_u32())
    }
}
