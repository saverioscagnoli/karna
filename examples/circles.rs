use karna::{
    core::EventLoop,
    math::{rng, Vector2},
    perf::{cpu, fps, mem, ups, MemUnit},
    render::{load_font, Color, Renderer},
    traits::{Load, Render, Update},
    window::{self, load_cursor, set_cursor},
};

struct Game {
    circles: Vec<(Vector2, u32, Color)>,
}

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
        for (pos, r, color) in &self.circles {
            renderer.set_color(*color);
            renderer.fill_circle(*pos, *r);
        }

        renderer.fill_text((10, 10), format!("fps: {}", fps()), Color::White);
        renderer.fill_text((10, 30), format!("ups: {}", ups()), Color::White);
        renderer.fill_text((10, 50), format!("cpu: {:.1}%", cpu()), Color::White);
        renderer.fill_text(
            (10, 70),
            format!("mem: {:.1} mb", mem(MemUnit::MB)),
            Color::White,
        );

        renderer.set_color(Color::Black);
    }
}

fn main() {
    let mut event_loop = EventLoop::new();

    event_loop.create_window("circles", 800, 600).unwrap();

    let mut circles = vec![];

    let size = window::size();

    for _ in 0..100 {
        let x = rng(0, size.width);
        let y = rng(0, size.height);
        let r = rng(25, 75);

        let color = Color::RGB(rng(0, 255), rng(0, 255), rng(0, 255));

        circles.push(((x, y).into(), r, color));
    }

    let game = Game { circles };

    event_loop.run(game);
}
