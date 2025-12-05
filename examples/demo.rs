use karna::{App, context::ScopedContext, scene::Scene};

struct S;

impl Scene for S {
    fn load(&mut self, ctx: &mut ScopedContext) {
        println!("ciao!!");
    }

    fn update(&mut self, ctx: &mut ScopedContext) {}

    fn render(&mut self, ctx: &mut ScopedContext) {}
}

fn main() {
    App::new()
        .with_size((800, 600))
        .with_initial_scene(Box::new(S))
        .run();
}
