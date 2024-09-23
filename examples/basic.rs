use std::time::Duration;

use karna::{
    core::EventLoop,
    input::{self, Key, Mouse},
    math::{Easing, Tween, Vector2},
    perf::fps,
    render::{load_font, Color, Renderer},
    traits::{Load, Render, Update},
    window,
};

struct Game {
    clicks: u32,
    pos: Vector2,
    vel: Vector2,
    tween: Option<Tween<f32>>,
}

impl Load for Game {
    fn load(&mut self, renderer: &mut Renderer) {
        load_font("default", "assets/font.ttf", 16);
        renderer.set_font("default");
    }
}

impl Update for Game {
    fn update(&mut self, dt: f32) {
        if input::key_pressed(Key::Space) {
            self.clicks += 1;
            window::set_title(self.clicks);
        }

        let speed = 50.0;
        let friction = 0.9;
        let acceleration = 0.5;

        if input::key_down(Key::W) {
            self.vel.y -= acceleration;
        }

        if input::key_down(Key::S) {
            self.vel.y += acceleration;
        }

        if input::key_down(Key::A) {
            self.vel.x -= acceleration;
        }

        if input::key_down(Key::D) {
            self.vel.x += acceleration;
        }

        self.vel *= friction;
        self.pos += self.vel * speed * dt;

        if let Some(tween) = &mut self.tween {
            window::set_size((800, tween.update(dt) as u32));
        }

        if input::click(Mouse::Left) {
            let pos = input::mouse_position();
            self.tween = Some(Tween::new(
                600.0,
                pos.y,
                Duration::from_secs_f32(2.0),
                Easing::BounceOut,
            ));
        }
    }
}

impl Render for Game {
    fn render(&mut self, renderer: &mut Renderer) {
        renderer.set_color(Color::Red);
        renderer.fill_rect(self.pos, (50, 50));

        renderer.fill_text(format!("fps: {}", fps()), (10, 10), Color::White);

        renderer.set_color(Color::Black);
    }
}

fn main() {
    let mut event_loop = EventLoop::new("pislo", 800, 600);
    let game = Game {
        clicks: 0,
        pos: Vector2::zero(),
        vel: Vector2::zero(),
        tween: None,
    };

    event_loop.run(game);
}
