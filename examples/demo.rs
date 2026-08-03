#![allow(unused)]

use std::u32;

use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::gpu::PresentMode;
use karna::imgui::Condition;
use karna::input::Key;
use karna::logging::Config;
use karna::logging::LevelFilter;
use karna::render::Color;
use karna::render::Draw;
use karna::render::SceneRef;
use math::Vector2;

struct Demo {
    position: math::Vector2<f32>,
    velocity: math::Vector2<f32>,
    show_demo: bool,
}

impl Demo {
    const SIZE: f32 = 64.0;
    const SPEED: f32 = 400.0;
    const SMOOTHING: f32 = 0.999;
}

impl Scene for Demo {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        Self {
            position: Vector2::new(100.0, 100.0),
            velocity: Vector2::zero(),
            show_demo: true,
        }
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        let dt = ctx.time.delta();

        let mut dir = math::Vector2::<f32>::zero();

        if ctx.input.key_down(Key::W) {
            dir.y -= 1.0;
        }
        if ctx.input.key_down(Key::S) {
            dir.y += 1.0;
        }
        if ctx.input.key_down(Key::A) {
            dir.x -= 1.0;
        }
        if ctx.input.key_down(Key::D) {
            dir.x += 1.0;
        }

        if ctx.input.key_pressed(Key::Space) {
            if ctx.time.present_mode() == PresentMode::Vsync {
                ctx.time.set_present_mode(PresentMode::Mailbox);
            } else {
                ctx.time.set_present_mode(PresentMode::Vsync);
            }
        }

        let len = (dir.x * dir.x + dir.y * dir.y).sqrt();

        if len > 0.0 {
            dir.x /= len;
            dir.y /= len;
        }

        let target = math::Vector2::new(dir.x * Self::SPEED, dir.y * Self::SPEED);

        let t = 1.0 - (1.0 - Self::SMOOTHING).powf(dt);

        self.velocity.x += (target.x - self.velocity.x) * t;
        self.velocity.y += (target.y - self.velocity.y) * t;

        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.debug_text("Hello world!", 10.0, 10.0);
        draw.debug_text(format!("fps {}", ctx.time.fps()), 10.0, 40.0);
        draw.debug_text(format!("dt {}", ctx.time.delta()), 10.0, 70.0);

        draw.set_color(Color::Cyan);
        draw.rect(self.position.x, self.position.y, Self::SIZE, Self::SIZE);

        draw.imgui(|ui| {
            ui.window("Demo")
                .size([300.0, 160.0], Condition::FirstUseEver)
                .build(|ui| {
                    ui.text(format!("fps    {:.0}", ctx.time.fps()));
                    ui.text(format!(
                        "pos    {:.1}, {:.1}",
                        self.position.x, self.position.y
                    ));
                    ui.separator();

                    if ui.button("Reset", [80.0, 0.0]) {
                        self.position = math::Vector2::new(100.0, 100.0);
                        self.velocity = math::Vector2::zero();
                    }
                });

            ui.show_demo_window(&mut self.show_demo);
        });
    }
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo")
                .with_size((1280, 720))
                .with_scene::<Demo>(0)
                .with_active_scene(0),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
