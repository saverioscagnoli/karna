use crate::context::ScopedContext;
use std::fmt::Debug;

pub trait Scene {
    fn load(&mut self, ctx: &mut ScopedContext);
    fn update(&mut self, ctx: &mut ScopedContext);

    #[allow(unused)]
    fn fixed_update(&mut self, ctx: &mut ScopedContext) {}

    fn render(&mut self, ctx: &mut ScopedContext);
}
