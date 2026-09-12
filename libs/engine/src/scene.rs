use nostd::alloc::boxed::Box;
use utils::Label;

use crate::context::DrawContext;
use crate::context::LoadContext;
use crate::context::UpdateContext;
use crate::render::Draw;

pub type SceneId = Label;

#[rustfmt::skip]
#[allow(unused)]
pub trait Scene: 'static {
    fn load(ctx: &mut LoadContext) -> Self where Self: Sized;
    fn unload(&mut self, ctx: &mut LoadContext) {}

    fn fixed_update(&mut self, ctx: &mut UpdateContext) {}
    fn update(&mut self, ctx: &mut UpdateContext);

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw);
}

pub type BoxedScene = Box<dyn Scene>;
pub type SceneBuilder = Box<dyn Fn(&mut LoadContext) -> BoxedScene>;
