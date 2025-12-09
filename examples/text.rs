use karna::{AppBuilder, Scene, WindowBuilder, label, render::Text};
use math::Vector2;
use renderer::Color;

struct S {
    text: Text,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);

        self.text.position = Vector2::new(10.0, 10.0)
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        self.text.content = format!(
            "fps: {}\ndt: {}\ntps: {}\nframe time: {:?}\ntick time: {:?}",
            ctx.time.fps(),
            ctx.time.delta(),
            ctx.time.tps(),
            ctx.time.frame(),
            ctx.time.tick()
        );
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        self.text.render(&ctx.gpu, &mut ctx.render);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(WindowBuilder::new().with_initial_scene(Box::new(S {
            text: Text::new(label!("debug"), ""),
        })))
        .build()
        .run();
}
