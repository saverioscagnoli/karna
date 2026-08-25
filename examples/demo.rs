#![allow(unused)]

use karna::prelude::*;
use math::Size;
use math::Vector2;

const ACCEL: f32 = 250.0;

struct Demo {
    size: Size<f32>,
    pos: Vector2<f32>,
    vel: Vector2<f32>,
}

impl Scene for Demo {
    fn load(ctx: LoadContext, scene: &mut SceneView) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);

        Self {
            size: Size::new(50.0, 50.0),
            pos: Vector2::new(10.0, 10.0),
            vel: Vector2::zero(),
        }
    }

    fn update(&mut self, ctx: UpdateContext, scene: &mut SceneView) {
        let dt = ctx.time.delta();

        if ctx.input.key_down(Key::W) {
            self.vel.y = -ACCEL;
        }

        if ctx.input.key_down(Key::A) {
            self.vel.x = -ACCEL;
        }

        if ctx.input.key_down(Key::S) {
            self.vel.y = ACCEL;
        }

        if ctx.input.key_down(Key::D) {
            self.vel.x = ACCEL;
        }

        self.pos += self.vel * dt;
        self.vel *= 0.9;

        if self.vel.length() < 0.25 {
            self.vel.set([0.0, 0.0]);
        }
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.set_color(Color::RED);
        draw.rect_v(self.pos, self.size);
    }
}

fn main() {
    init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo")
                .with_scene::<Demo>(SceneId::new_label("demo"))
                .with_active_scene(SceneId::new_label("demo")),
        )
        .build()
        .run();
}
