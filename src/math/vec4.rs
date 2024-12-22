use super::ToF32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    /// Creates a new 4D vector.
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn as_ptr(&self) -> *const f32 {
        &self.x as *const f32
    }

    /// Sets the x, y, x, w components of the vector at the same time.
    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
    }
}

/// Create a Vec4 from a tuple.
/// # Example
/// ```
/// let a = (1.0, 2.0, 3.0, 4.0);
/// let b = Vec4::from(a);
///
/// assert_eq!(b, Vec4::new(1.0, 2.0, 3.0, 4.0));
impl<F: ToF32> From<(F, F, F, F)> for Vec4 {
    fn from(t: (F, F, F, F)) -> Self {
        Self::new(t.0.to_f32(), t.1.to_f32(), t.2.to_f32(), t.3.to_f32())
    }
}

/// Creates a tuple from a Vec4.
/// # Example
/// ```
/// let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
/// let b = (1.0, 2.0, 3.0, 4.0);
///
/// assert_eq!(a.into(), b);
/// ```
impl From<Vec4> for (f32, f32, f32, f32) {
    fn from(v: Vec4) -> Self {
        (v.x, v.y, v.z, v.w)
    }
}
