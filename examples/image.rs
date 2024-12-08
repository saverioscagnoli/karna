use karna::{
    render::Color,
    traits::{Draw, Load, Update},
    App, Context,
};

struct Game;

impl Load for Game {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.load_image("cat", "examples/assets/cat.png");
    }
}

impl Update for Game {
    fn update(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::RED);

        ctx.render.fill_rect((50, 100), (50, 50));
        ctx.render.set_color(Color::BLACK);
        ctx.render.draw_image("cat", (0, 0));
    }
}

fn main() {
    App::new("meow", (800, 600)).unwrap().run(&mut Game);
}
