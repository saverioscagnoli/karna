use karna::{render::Color, traits::Scene, App, Context};

struct FirstScene;

impl Scene for FirstScene {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.load_image("cat", "examples/assets/cat.png");
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::RED);

        ctx.render.fill_rect((50, 100), (50, 50));
        ctx.render.set_color(Color::BLACK);
        ctx.render.draw_image("cat", (0, 0));
    }
}

fn main() {
    App::window("meow", (800, 600)).run(FirstScene);
}
