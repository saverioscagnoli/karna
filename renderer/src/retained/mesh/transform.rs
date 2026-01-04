use math::Vector3;

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct Transform3d {
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
}
