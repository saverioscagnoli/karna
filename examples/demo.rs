use karna::{render::Color, traits::Scene, App, Context};

struct S;

impl Scene<Context> for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        println!("delta time: {}", ctx.time.elapsed());
    }

    fn draw(&self, ctx: &mut Context) {
        ctx.render.clear_background(Color::CYAN);
        ctx.render._present();
    }
}

fn main() {
    App::window("Demo window", (800, 600)).run(S);
}
