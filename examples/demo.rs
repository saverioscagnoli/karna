use karna::{App, Context, Scene, render::Color};
use math::Vector2;
use renderer::Transform2D;

pub struct S;

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::Purple);
    }

    fn update(&mut self, ctx: &mut Context) {
        println!("fps: {} tps: {}", ctx.time.fps(), ctx.time.tps());
        println!("{}", Transform2D::default().position_y());
    }

    fn render(&mut self, ctx: &mut Context) {}
}

fn main() {
    App::new().with_initial_scene("default", Box::new(S)).run();
}
