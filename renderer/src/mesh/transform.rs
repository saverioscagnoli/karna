use macros::{Get, Set, With};
use math::{Size, Vector2};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Get, Set, With)]
pub struct Transform {
    #[get]
    #[get(prop = x, ty = f32, copied)]
    #[get(prop = y, ty = f32, copied)]
    #[set(into)]
    #[set(prop = x, ty = f32)]
    #[set(prop = y, ty = f32)]
    #[with(into)]
    #[with(prop = x, ty = f32)]
    #[with(prop = y, ty = f32)]
    pub position: Vector2,

    #[get]
    #[get(prop = x, ty = f32, copied)]
    #[get(prop = y, ty = f32, copied)]
    #[set(into)]
    #[set(prop = x, ty = f32)]
    #[set(prop = y, ty = f32)]
    #[with(into)]
    #[with(prop = x, ty = f32)]
    #[with(prop = y, ty = f32)]
    pub scale: Vector2,

    #[get(copied)]
    #[set]
    #[with]
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector2::zeros(),
            scale: Vector2::ones(),
            rotation: 0.0,
        }
    }
}

impl Transform {
    pub fn new(position: Vector2, scale: Vector2, rotation: f32) -> Self {
        Self {
            position,
            scale,
            rotation,
        }
    }

    /// Set the scale to match the given dimensions
    pub fn with_size(mut self, size: Size<u32>) -> Self {
        self.scale = Vector2::new(size.width as f32, size.height as f32);
        self
    }

    /// Set the scale to match the given dimensions
    pub fn set_size(&mut self, size: Size<u32>) {
        self.scale = Vector2::new(size.width as f32, size.height as f32);
    }
}
