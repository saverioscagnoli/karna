use math::Size;

use crate::context::Context;

#[allow(unused)]
pub trait Scene: Send {
    fn load(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context);
    fn render(&mut self, ctx: &mut Context);

    // Optional
    fn fixed_update(&mut self, ctx: &mut Context) {}
    fn on_resize(&mut self, size: Size<u32>, ctx: &mut Context) {}
}
