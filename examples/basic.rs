use karna::{
    render::Color,
    traits::{Draw, Load, Update},
    App, Context,
};

struct Game;

impl Load for Game {
    fn load(&mut self, _ctx: &mut Context) {
        println!("Game loaded!");
    }
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        println!("dt: {}", ctx.time.delta());
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render
            .fill_text("Hello, world!", (10.0, 10.0), Color::WHITE);

        ctx.render.set_color(Color::BLACK);
    }
}

fn main() {
    App::new("Basic window", (800, 600)).unwrap().run(&mut Game);
}
