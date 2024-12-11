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
    fn update(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::GREEN);
        ctx.render.draw_circle((100, 100), 50);

        ctx.render.set_color(Color::RED);

        ctx.render.draw_aa_circle((500, 300), 100);

        ctx.render.draw_circle((200, 200), 50);

        ctx.render.set_color(Color::BLUE);

        ctx.render.fill_circle((500, 500), 40);

        ctx.render.set_color(Color::CYAN);

        ctx.render.fill_aa_circle((300, 400), 75);

        ctx.render.set_color(Color::BLACK);
    }
}

fn main() {
    App::new("Basic window", (800, 600)).unwrap().run(&mut Game);
}
