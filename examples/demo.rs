use std::f32;

use karna::App;
use karna::ContextMut;
use karna::ContextRef;
use karna::Scene;
use karna::SceneHandle;
use karna::WindowBuilder;
use karna::assets::Audio;
use karna::input::Keycode;
use karna::math::Size;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use utils::Handle;

struct S {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    mammamia: Handle<Audio>,
}

impl Scene for S {
    fn load(&mut self, mut ctx: ContextMut, scene: &mut SceneHandle) {}

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let accel = 5000.0;
        let dt = ctx.time.delta();

        self.vel.y += ctx.input.key_axis([Keycode::KeyW, Keycode::KeyS]) * accel * dt;
        self.vel.x += ctx.input.key_axis([Keycode::KeyA, Keycode::KeyD]) * accel * dt;

        self.vel *= 0.85f32.powf(60.0).powf(dt);
        self.pos += self.vel * dt;
    }

    fn draw(&self, ctx: ContextRef, draw: &mut Draw) {
        draw.set_color(Color::White);

        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);

        draw.set_color(Color::Cyan);

        draw.push_state();
        draw.translate(self.pos.x + 100.0, self.pos.y);
        draw.rotate(f32::consts::PI / 4.0);
        draw.rect(0.0, 0.0, 50.0, 50.0);
        draw.pop_state();

        draw.set_color(Color::Magenta);

        draw.push_state();
        draw.translate(self.pos.x, self.pos.y + 100.0);
        draw.scale(2.0, 1.0);
        draw.rect(0.0, 0.0, 50.0, 50.0);
        draw.pop_state();

        draw.set_color(Color::Magenta);

        draw.line_v([300.0, 100.0], self.pos + Vector2::new(25.0, 25.0));

        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.point(i as f32 * 10.0 + 200.0, j as f32 * 10.0 + 200.0);
            }
        }
    }
}

fn main() {
    karna::logging::init(
        karna::logging::Config {
            min_level: karna::logging::LevelFilter::Debug,
            ..Default::default()
        }
        .hide_wgpu(true),
    )
    .expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_scene(
                    "demo",
                    S {
                        pos: Vector2::new(50.0, 50.0),
                        vel: Vector2::zero(),
                        mammamia: Handle::default(),
                    },
                )
                .with_active_scene("demo"),
        )
        .build()
        .run();
}
