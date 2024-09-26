use crate::render::Renderer;

pub trait Load {
    fn load(&mut self, renderer: &mut Renderer);
}

pub trait Update {
    fn update(&mut self, step: f32);
}

pub trait Render {
    fn render(&mut self, renderer: &mut Renderer);
}
