use karna::{
    input::Key,
    math::ToF32,
    render::Color,
    traits::{Draw, Load, Update},
    App, Context,
};

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
        let text = "Press SPACE to play the audio!";
        let center = ctx.window.center_position();
        let text_size = ctx.render.text_size(text);

        let x = center.x - text_size.width.to_f32() / 2.0;
        let y = center.y - text_size.height.to_f32() / 2.0;

        ctx.render
            .fill_text(text, (x.round(), y.round()), Color::WHITE);
    }
}

fn main() {
    App::new("GET OUT", (800, 600)).unwrap().run(&mut Game);
}
