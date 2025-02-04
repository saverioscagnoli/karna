/// The most basic trait in this library, used to define the current scene to draw.
/// Implement as much scene as you want, and switch between them as you please.
///
/// This takes in a generic type `T`, which is the context that the scene will interact with.
/// It is not defined immediately because it would cause a circular dependency.
// TODO: Find a way to define the context here.
pub trait Scene<T> {
    fn load(&mut self, ctx: &mut T);
    fn update(&mut self, ctx: &mut T);
    fn draw(&self, ctx: &mut T);
}

/// Helper trait to convert a value to f32.
/// This trait in the library will be used in a way
/// to easily create a f32 value from any type.
///
/// # Example
///
/// ```no_run
/// // Creating Vectors:
/// let vec: Vec2 = [10, 10].into(); // Vec2 { x: 10.0, y: 10.0 }
/// let vec3 = Vec3::new(10, 20, 30); // Vec3 { x: 10.0, y: 20.0, z: 30.0 }
///
/// // Drawing rectangles:
/// ctx.render.draw_rect(0, 0, 100, 100); // Draw a rectangle at (0, 0) with size 100x100 (values are floats)
/// ```
/// Keep in mind that the numbers will still be f32 under the hood.
/// It might be a bit ambiguous, but I prefer it this way.
pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

impl ToF32 for f32 {
    #[inline]
    fn to_f32(&self) -> f32 {
        *self
    }
}

impl ToF32 for f64 {
    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for i32 {
    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for i64 {
    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for u32 {
    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for u64 {
    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for usize {
    #[inline]
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

/// Helper trait to convert a value to u32.
/// This trait in the library will be used in a way
/// to easily create a u32 value from any type.
pub trait ToU32 {
    fn to_u32(&self) -> u32;
}

impl ToU32 for f32 {
    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for f64 {
    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for i32 {
    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for i64 {
    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for u32 {
    #[inline]
    fn to_u32(&self) -> u32 {
        *self
    }
}

impl ToU32 for u64 {
    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for usize {
    #[inline]
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}
