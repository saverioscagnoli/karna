use karna::{
    core::EventLoop,
    input::{self, Key},
    math::Vector2,
    perf::fps,
    render::{load_font, Color, Renderer},
    traits::{Load, Render, Update},
    window::{load_cursor, set_cursor},
};

const ACCELERATION: f32 = 0.5;
const FRICTION: f32 = 0.9;

struct Game {
    pos: Vector2,
    vel: Vector2,
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
    fn update(&mut self, _step: f32) {
        if self.vel.x.abs() < 0.1 {
            self.vel.x = 0.0;
        }

        if self.vel.y.abs() < 0.1 {
            self.vel.y = 0.0;
        }

        if input::key_down(Key::W) {
            self.vel.y -= ACCELERATION;
        }

        if input::key_down(Key::S) {
            self.vel.y += ACCELERATION;
        }

        if input::key_down(Key::A) {
            self.vel.x -= ACCELERATION;
        }

        if input::key_down(Key::D) {
            self.vel.x += ACCELERATION;
        }

        self.pos += self.vel;
        self.vel *= FRICTION;
    }
}

impl Render for Game {
    fn render(&mut self, renderer: &mut karna::render::Renderer) {
        renderer.set_color(Color::Red);
        renderer.fill_rect(self.pos, (50, 50));

        renderer.fill_text((10, 10), format!("fps: {}", fps()), Color::White);
        renderer.fill_text(
            (10, 30),
            format!("pos: (x: {:.1} y: {:.1})", self.pos.x, self.pos.y),
            Color::White,
        );
        renderer.fill_text(
            (10, 50),
            format!("vel: (x: {:.1} y: {:.1})", self.vel.x, self.vel.y),
            Color::White,
        );

        renderer.set_color(Color::Black);
    }
}

fn main() {
    let mut event_loop = EventLoop::new();

    event_loop.create_window("basic window", 800, 600).unwrap();

    let game = Game {
        pos: Vector2::new(400.0, 300.0),
        vel: Vector2::new(0.0, 0.0),
    };

    event_loop.run(game);
}
