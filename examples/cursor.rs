use karna::{input::Mouse, render::Color, traits::Scene, App, Context, Cursor};

struct FirstScene;

impl Scene for FirstScene {
    fn load(&mut self, ctx: &mut Context) {
        ctx.window
            .load_cursor("custom", "examples/assets/cursor.png");

        ctx.window.set_cursor(Cursor::Custom("custom"));
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.mouse_clicked(Mouse::Left) {
            ctx.window.set_cursor(Cursor::Custom("custom"));
        }

        if ctx.input.mouse_clicked(Mouse::Right) {
            ctx.window.set_cursor(Cursor::Hand);
        }
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.fill_text(
            "Right click to change into system hand cursor!",
            (10, 10),
            Color::WHITE,
        );

        ctx.render.fill_text(
            "Left click to change into custom cursor!",
            (10, 30),
            Color::WHITE,
        );
    }
}

fn main() {
    App::window("basic window", (800, 600)).run(FirstScene);
}
