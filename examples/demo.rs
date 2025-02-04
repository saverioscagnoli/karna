use karna::{traits::Scene, App, Context};

struct S;

impl Scene<Context> for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&self, _ctx: &mut Context) {}
}

fn main() {
    App::window("Demo window", (800, 600)).run(S);
}
