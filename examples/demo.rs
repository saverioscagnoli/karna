use karna::{App, AppBuilder, Context, Scene, WindowBuilder, input::KeyCode, render::Color};

pub struct S;

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::Red);
        ctx.time.set_recommended_fps();
    }

    fn update(&mut self, ctx: &mut Context) {
        println!("fps {}, dt {}", ctx.time.fps(), ctx.time.delta());
    }

    fn render(&mut self, ctx: &mut Context) {}
}

pub struct S2;

impl Scene for S2 {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_clear_color(Color::Green);
    }

    fn update(&mut self, ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {}
}

fn main() {
    AppBuilder::new()
        .with_window(WindowBuilder::new().with_initial_scene(Box::new(S)))
        // .with_window(WindowBuilder::new().with_initial_scene(Box::new(S2)))
        .build()
        .run();
}
