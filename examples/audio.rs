use karna::{
    input::Key,
    traits::{Draw, Load, Update},
    App, Context,
};
use sdl2::pixels::Color;

struct Game;

impl Load for Game {
    fn load(&mut self, ctx: &mut Context) {
        ctx.audio
            .load("get-out", "examples/assets/tuco-get-out.mp3");
    }
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.key_pressed(Key::Space) {
            ctx.audio.play("get-out");
        }
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render
            .fill_text("Hello world!", (10, 10), Color::WHITE);
    }
}

fn main() {
    App::new("basic window", (800, 600)).unwrap().run(&mut Game);
}
