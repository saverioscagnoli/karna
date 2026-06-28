use crate::context::ContextRef;
use crate::context::ContextRefMut;

pub trait Scene {
    fn load(&mut self, ctx: ContextRefMut);
    fn update(&mut self, ctx: ContextRefMut);
    fn fixed_update(&mut self, ctx: ContextRefMut) {}
    fn draw(&mut self, ctx: ContextRef);
}
