#![allow(unused)]

use karna::App;
use karna::ContextRef;
use karna::Scene;
use karna::WindowBuilder;
use karna::input::Key;
use karna::logging::Config;
use karna::logging::LevelFilter;
use karna::math::Size;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use karna::render::SceneRef;
use math::Vector3;

struct Demo {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
}

impl Scene for Demo {
    fn load(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        let dt = ctx.time.fixed_delta();
        let max_speed = 400.0;
        let responsiveness = 10.0;

        let mut dx = 0.0;
        let mut dy = 0.0;

        if ctx.input.key_down(Key::W) {
            dy -= 1.0;
        }

        if ctx.input.key_down(Key::S) {
            dy += 1.0;
        }

        if ctx.input.key_down(Key::A) {
            dx -= 1.0;
        }

        if ctx.input.key_down(Key::D) {
            dx += 1.0;
        }

        let len = ((dx * dx + dy * dy) as f32).sqrt();

        if len > 0.0 {
            dx /= len;
            dy /= len;
        }

        let target_x = dx * max_speed;
        let target_y = dy * max_speed;
        let t = 1.0 - (-responsiveness * dt).exp();

        self.vel.x += (target_x - self.vel.x) * t;
        self.vel.y += (target_y - self.vel.y) * t;
        self.pos += self.vel * dt;

        let camera = scene.camera_mut();
        let view = ctx.window.size().as_f32();
        let target = Vector3::new(self.pos.x + 25.0, self.pos.y + 25.0, 0.0)
            - Vector3::new(view.width / 2.0, view.height / 2.0, 0.0);
        let cam_t = 1.0 - (-2.5 * ctx.time.fixed_delta()).exp();
        camera.position = camera.position.lerp(&target, cam_t);
    }

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {
        draw.set_color(Color::Red);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);
    }
}

fn main() {
    karna::init_logging(Config::default().with_min_level(LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo window")
                .with_size((1280, 720))
                .with_scene(
                    "demo",
                    Demo {
                        pos: Vector2::new(10.0, 10.0),
                        vel: Vector2::zero(),
                    },
                    true,
                ),
        )
        .build()
        .run();
}
