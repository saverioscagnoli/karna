use macros::{Get, Set, With};
use math::Vector2;

#[derive(Debug, Clone, Copy)]
#[derive(Get, Set, With)]
pub struct Transform2D {
    #[get]
    #[get(prop = x, ty = f32, copied)]
    #[get(prop = y, ty = f32, copied)]
    #[set(into, ty = Vector2)]
    #[set(prop = x, ty = f32)]
    #[set(prop = y, ty = f32)]
    #[with(into, ty = Vector2)]
    #[with(prop = x, ty = f32)]
    #[with(prop = y, ty = f32)]
    pub position: Vector2,

    #[get]
    #[get(prop = x, ty = f32, copied)]
    #[get(prop = y, ty = f32, copied)]
    #[set(into, ty = Vector2)]
    #[set(prop = x, ty = f32)]
    #[set(prop = y, ty = f32)]
    #[with(into, ty = Vector2)]
    #[with(prop = x, ty = f32)]
    #[with(prop = y, ty = f32)]
    pub scale: Vector2,

    #[get]
    #[set(ty = f32)]
    #[with(ty = f32)]
    pub rotation: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vector2::zeros().into(),
            scale: Vector2::ones().into(),
            rotation: 0.0.into(),
        }
    }
}
