use crate::context::Context;

pub trait Load {
    fn load(&mut self, ctx: &mut Context);
}

pub trait Update {
    fn update(&mut self, _ctx: &mut Context) {}
    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

pub trait Render {
    fn render(&mut self, ctx: &mut Context);
}

pub trait ToU32 {
    fn to_u32(&self) -> u32;
}

impl ToU32 for f32 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for i32 {
    fn to_u32(&self) -> u32 {
        *self as u32
    }
}

impl ToU32 for u32 {
    fn to_u32(&self) -> u32 {
        *self
    }
}

pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

impl ToF32 for f32 {
    fn to_f32(&self) -> f32 {
        *self
    }
}

impl ToF32 for i32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for u32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}
