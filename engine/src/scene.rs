use crate::context::Context;

pub trait Scene {
    fn load(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context);
    #[allow(unused)]
    fn fixed_update(&mut self, ctx: &mut Context) {}
    fn render(&self, ctx: &mut Context);
}
