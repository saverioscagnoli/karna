use macros::{Get, Set, With};
use math::Vector2;

#[derive(Debug, Clone, Copy)]
#[derive(Get, Set, With)]
pub struct Transform {
    #[get]
    #[with(into)]
    #[with(prop = x, ty = f32)]
    #[with(prop = y, ty = f32)]
    pub(crate) position: Vector2,

    #[get]
    #[with(into)]
    #[with(prop = x, ty = f32)]
    #[with(prop = y, ty = f32)]
    pub(crate) scale: Vector2,

    #[get(copied)]
    #[with]
    pub(crate) rotation: f32,
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
    pub fn new<P: Into<Vector2>, S: Into<Vector2>>(position: P, scale: S, rotation: f32) -> Self {
        Self {
            position: position.into(),
            scale: scale.into(),
            rotation,
        }
    }
}
