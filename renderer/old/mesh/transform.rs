use macros::{Get, Set, With};
use math::{Vector2, Vector3};

#[derive(Debug, Clone, Copy)]
#[derive(Get, Set, With)]
pub struct Transform {
    #[get]
    #[set(into)]
    #[with(into)]
    pub position: Vector3,

    #[get]
    #[set(into)]
    #[with(into)]
    pub rotation: Vector3,

    #[get]
    #[set(into)]
    #[with(into)]
    pub scale: Vector3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            rotation: Vector3::zeros(),
            scale: Vector3::ones(),
        }
    }
}

impl Transform {
    #[inline]
    pub fn new<P, R, S>(position: P, rotation: R, scale: S) -> Self
    where
        P: Into<Vector3>,
        R: Into<Vector3>,
        S: Into<Vector3>,
    {
        Self {
            position: position.into(),
            rotation: rotation.into(),
            scale: scale.into(),
        }
    }

    #[inline]
    pub fn new_2d<P, S>(position: P, rotation: f32, scale: S) -> Self
    where
        P: Into<Vector2>,
        S: Into<Vector2>,
    {
        let position: Vector3 = position.into().extend(0.0);
        let rotation = Vector3::new(0.0, 0.0, rotation);
        let scale: Vector3 = scale.into().extend(0.0);

        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn combine(&self, local: &Transform) -> Transform {
        Transform {
            position: self.position + self.rotation * (self.scale * local.position),
            rotation: self.rotation * local.rotation,
            scale: self.scale * local.scale,
        }
    }
}
