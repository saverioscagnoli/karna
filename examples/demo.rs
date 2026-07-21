#![allow(unused)]

use karna::App;
use karna::Context;
use karna::Scene;
use karna::SceneView;
use karna::WindowBuilder;
use karna::input::Keycode;
use karna::logging::Config;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;

const DEMO_SCENE: usize = 0;

struct Demo {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
}

impl Scene for Demo {
    fn load(&mut self, ctx: &mut Context, scene: &mut SceneView) {
        ctx.time.set_target_fps(120);
    }

    fn fixed_update(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn update(&mut self, ctx: &mut Context, scene: &mut SceneView) {
        let dt = ctx.time.delta();
        let max_speed = 400.0;
        let responsiveness = 10.0;

        let mut dx = 0.0;
        let mut dy = 0.0;

        if ctx.input.key_held(Keycode::W) {
            dy -= 1.0;
        }

        if ctx.input.key_held(Keycode::S) {
            dy += 1.0;
        }

        if ctx.input.key_held(Keycode::A) {
            dx -= 1.0;
        }

        if ctx.input.key_held(Keycode::D) {
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
    }

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {
        draw.set_color(Color::Red);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);
    }
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Demo window")
                .with_size((1280, 720))
                .with_scene(
                    DEMO_SCENE,
                    Demo {
                        pos: Vector2::new(10.0, 19.9),
                        vel: Vector2::zero(),
                    },
                )
                .with_active_scene(DEMO_SCENE),
        )
        .build()
        .run();
}
