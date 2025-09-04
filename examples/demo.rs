use karna::{App, Context, Scene};

struct S;

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {}

    fn render(&mut self, ctx: &mut Context) {}
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene("default", S)
        .run()
        .expect("Failed to run application");
}
