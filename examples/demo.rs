use karna::{App, Context, Scene, render::Color};
use math::{Vector2, Vector3};
use renderer::{Mesh, Rectangle, Transform2D};

pub struct S {
    rect: Rectangle,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        self.rect.instance.position = Vector3::new(10.0, 10.0, 0.0);
    }

    fn update(&mut self, ctx: &mut Context) {
        println!("fps: {} tps: {}", ctx.time.fps(), ctx.time.tps());
        self.rect.instance.position.x += 10.0 * ctx.time.delta();
        self.rect.instance.position.y += 10.0 * ctx.time.delta();
    }

    fn render(&mut self, ctx: &mut Context) {
        self.rect.render(&mut ctx.render)
    }
}

fn main() {
    App::new()
        .with_initial_scene(
            "default",
            Box::new(S {
                rect: Rectangle::new(50.0, 50.0, Color::Red),
            }),
        )
        .run();
}
