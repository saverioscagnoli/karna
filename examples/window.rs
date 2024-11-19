use karna::{
    context::Context,
    core::App,
    math::lerp,
    traits::{Load, Render, Update},
};

const LERP_FACTOR: f32 = 0.25;

struct Game;

impl Load for Game {
    fn load(&mut self, ctx: &mut Context) {
        ctx.window.set_always_on_top(true);
        ctx.window.set_opacity(0.5).unwrap();
        ctx.window.set_decorations(false);
    }
}

impl Update for Game {
    fn fixed_update(&mut self, ctx: &mut Context) {
        let mouse = ctx.input.mouse_position();
        let win_pos = ctx.window.position();

        let new_pos = lerp(win_pos, mouse, LERP_FACTOR);

        ctx.window.set_position(new_pos);
    }
}

impl Render for Game {
    fn render(&mut self, _ctx: &mut Context) {}
}

fn main() {
    App::new()
        .unwrap()
        .window("funny", (800, 600))
        .run(&mut Game);
}
