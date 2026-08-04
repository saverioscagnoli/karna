#![allow(unused)]

use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::imgui;
use karna::input::Key;
use karna::math::Vector2;
use karna::math::Vector4;
use karna::render::Color;
use karna::render::Draw;
use karna::render::SceneRef;
use logging::Config;
use logging::LevelFilter;

struct ImguiDemo {
    pos: Vector2<f32>,
    prev_pos: Vector2<f32>,
    vel: Vector2<f32>,
    fps_history: Vec<f32>,
    clear_color: Vector4<f32>,
}

impl Scene for ImguiDemo {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(175);

        Self {
            pos: Vector2::new(10.0, 10.0),
            prev_pos: Vector2::new(10.0, 10.0),
            vel: Vector2::zero(),
            fps_history: vec![0.0; 120],
            clear_color: Color::Black.into(),
        }
    }

    fn fixed_update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        const VEL: f32 = 250.0;

        self.prev_pos = self.pos;

        if ctx.input.key_down(Key::W) {
            self.vel.y = -VEL;
        }

        if ctx.input.key_down(Key::A) {
            self.vel.x = -VEL;
        }

        if ctx.input.key_down(Key::S) {
            self.vel.y = VEL;
        }

        if ctx.input.key_down(Key::D) {
            self.vel.x = VEL;
        }

        self.pos += self.vel * ctx.time.fixed_delta();
        self.vel *= 0.9;

        if self.vel.length_sq() < 0.01 {
            self.vel.set([0.0, 0.0]);
        }
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        scene.set_clear_color(self.clear_color);
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.rect(40.0 + i as f32 * 10.0, 100.0 + j as f32 * 10.0, 1.0, 1.0);
            }
        }

        let alpha = ctx.time.alpha();
        let render_pos = self.prev_pos.lerp(&self.pos, alpha);

        draw.set_color(Color::White);
        draw.rect(render_pos.x, render_pos.y, 50.0, 50.0);

        draw.imgui(|ui| {
            self.fps_history.remove(0);
            self.fps_history.push(ctx.time.fps() as f32);

            ui.window("Panel")
                .size([320.0, 500.0], imgui::Condition::FirstUseEver)
                .position([10.0, 10.0], imgui::Condition::FirstUseEver)
                .build(|ui| {
                    let fps = ctx.time.fps();
                    let dt = ctx.time.delta();
                    let fixed_dt = ctx.time.fixed_delta();
                    let tps = ctx.time.tps();

                    let fps_color = if fps >= 55 {
                        [0.2, 1.0, 0.2, 1.0]
                    } else if fps >= 30 {
                        [1.0, 0.8, 0.2, 1.0]
                    } else {
                        [1.0, 0.2, 0.2, 1.0]
                    };

                    ui.text_colored(fps_color, format!("FPS: {:.1}", fps));

                    ui.separator();
                    ui.text(format!("Frame step (delta):  {:.4} ms", dt * 1000.0));
                    ui.text(format!("Tick step (fixed dt): {:.4} ms", fixed_dt * 1000.0));
                    ui.text(format!("Ticks per second:    {:.1}", tps));
                    ui.text(format!("Interp alpha:        {:.3}", ctx.time.alpha()));

                    ui.color_edit4("clear", &mut self.clear_color)
                });
        });
    }
}

struct ImguiAtlasDemo;

impl Scene for ImguiAtlasDemo {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(175);
        Self
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        _ = ctx;
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        _ = ctx;
        draw.imgui(|ui| {
            ui.show_demo_window(&mut true);
            ui.show_metrics_window(&mut true);
        });
    }
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_scene::<ImguiDemo>(0)
                .with_active_scene(0),
        )
        .with_window(
            WindowBuilder::new()
                .with_size((1024, 1024))
                .with_scene::<ImguiAtlasDemo>(0)
                .with_active_scene(0),
        )
        .build()
        .run();
}
