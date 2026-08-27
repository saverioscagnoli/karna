use utils::Label;

use crate::Draw;
use crate::window::DrawContext;
use crate::window::LoadContext;
use crate::window::UpdateContext;

pub type SceneId = Label;

#[allow(unused)]
pub trait Scene: 'static {
    fn load(ctx: LoadContext) -> Self
    where
        Self: Sized;

    fn update(&mut self, ctx: UpdateContext);
    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw);

    fn fixed_update(&mut self, ctx: UpdateContext) {}
    fn unload(&mut self, ctx: LoadContext) {}
}

pub type BoxedScene = Box<dyn Scene>;
pub type SceneBuilder = Box<dyn Fn(LoadContext) -> BoxedScene>;
