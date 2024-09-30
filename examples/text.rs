use karna::{
    core::EventLoop,
    perf::fps,
    render::{load_font, Color, Renderer},
    traits::{Load, Render, Update},
};

struct Game {
    angle: f32,
}

impl Load for Game {
    fn load(&mut self, renderer: &mut Renderer) {
        load_font("default", "assets/font.ttf", 16);
        renderer.set_font("default");
    }
}

impl Update for Game {
    fn update(&mut self, step: f32) {
        self.angle += step * 10.0;
    }
}

impl Render for Game {
    fn render(&mut self, renderer: &mut karna::render::Renderer) {
        renderer.set_color(Color::Red);
        renderer.draw_rect((300, 100), (50, 50));

        renderer.fill_text((10, 10), fps(), Color::White);
        renderer.fill_text_ex(
            (300, 300),
            "Hello, world!",
            Color::Cyan,
            Some(self.angle),
            None,
            false,
            false,
        );

        renderer.set_color(Color::Black);
    }
}

fn main() {
    let mut event_loop = EventLoop::new();

    event_loop.create_window("basic window", 800, 600).unwrap();

    let game = Game { angle: 0.0 };

    event_loop.run(game);
}
