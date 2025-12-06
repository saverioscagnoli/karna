use crate::context::Context;

pub trait Scene: Send {
    fn load(&mut self, ctx: &mut Context);
    #[allow(unused)]
    fn fixed_update(&mut self, ctx: &mut Context) {}
    fn update(&mut self, ctx: &mut Context);
    fn render(&mut self, ctx: &mut Context);
}
