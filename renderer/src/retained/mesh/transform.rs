use macros::With;
use math::Vector3;

#[derive(Debug, Clone, Copy)]
#[derive(With)]
pub struct Transform3d {
    #[with(into)]
    pub position: Vector3,

    #[with(into)]
    pub rotation: Vector3,

    #[with(into)]
    pub scale: Vector3,
}

impl Default for Transform3d {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            rotation: Vector3::zeros(),
            scale: Vector3::ones(),
        }
    }
}
