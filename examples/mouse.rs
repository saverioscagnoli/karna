use karna::{
    core::EventLoop,
    input,
    perf::fps,
    render::{load_font, Color, Renderer},
    traits::{Load, Render, Update},
    window::{load_cursor, set_cursor},
};

struct Game;

impl Load for Game {
    fn load(&mut self, renderer: &mut Renderer) {
        load_font("default", "assets/font.ttf", 16);
        renderer.set_font("default");

        load_cursor("default", "assets/cursor.png");
        set_cursor("default");
    }
}

impl Update for Game {
    fn update(&mut self, _step: f32) {}
}

impl Render for Game {
    fn render(&mut self, renderer: &mut karna::render::Renderer) {
        renderer.set_color(Color::Red);

        let pos = input::mouse_position() - 25;

        renderer.fill_rect(pos, (50, 50));

        renderer.fill_text((10, 10), fps(), Color::White);

        renderer.set_color(Color::Black);
    }
}

fn main() {
    let mut event_loop = EventLoop::new();

    event_loop.create_window("basic window", 800, 600).unwrap();

    let game = Game;

    event_loop.run(game);
}
