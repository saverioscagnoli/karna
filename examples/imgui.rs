#![allow(unused)]

use karna::App;
use karna::ContextMut;
use karna::ContextRef;
use karna::Scene;
use karna::SceneHandle;
use karna::WindowBuilder;
use karna::imgui;
use karna::input::Keycode;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;

struct S {
    pos: Vector2<f32>,
    prev_pos: Vector2<f32>,
    vel: Vector2<f32>,
    fps_history: Vec<f32>,
    clear_color: [f32; 4],
}

impl Scene for S {
    fn load(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        ctx.time.set_target_fps(120);
    }

    fn fixed_update(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        const VEL: f32 = 250.0;

        // snapshot before mutating this tick
        self.prev_pos = self.pos;

        if ctx.input.key_held(Keycode::KeyW) {
            self.vel.y = -VEL;
        }

        if ctx.input.key_held(Keycode::KeyA) {
            self.vel.x = -VEL;
        }

        if ctx.input.key_held(Keycode::KeyS) {
            self.vel.y = VEL;
        }

        if ctx.input.key_held(Keycode::KeyD) {
            self.vel.x = VEL;
        }

        self.pos += self.vel * ctx.time.fixed_delta();
        self.vel *= 0.9;

        if self.vel.length_sq() < 0.01 {
            self.vel.set([0.0, 0.0]);
        }
    }

    fn update(&mut self, _ctx: ContextMut, scene: &mut SceneHandle) {}

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {
        draw.set_clear_color(self.clear_color);
        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.point(40.0 + i as f32 * 10.0, 100.0 + j as f32 * 10.0);
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
                .build(|| {
                    let fps = ctx.time.fps();
                    let dt = ctx.time.delta();
                    let fixed_dt = ctx.time.fixed_delta();
                    let tps = ctx.time.tps();

                    // color-code fps: green when healthy, yellow/red when struggling
                    let fps_color = if fps >= 55 {
                        [0.2, 1.0, 0.2, 1.0]
                    } else if fps >= 30 {
                        [1.0, 0.8, 0.2, 1.0]
                    } else {
                        [1.0, 0.2, 0.2, 1.0]
                    };

                    ui.text_colored(fps_color, format!("FPS: {:.1}", fps));
                    ui.plot_lines_config("##fps_history", &self.fps_history)
                        .scale_min(0.0)
                        .graph_size([280.0, 60.0])
                        .build();

                    ui.separator();
                    ui.text(format!("Frame step (delta):  {:.4} ms", dt * 1000.0));
                    ui.text(format!("Tick step (fixed dt): {:.4} ms", fixed_dt * 1000.0));
                    ui.text(format!("Ticks per second:    {:.1}", tps));
                    ui.text(format!("Interp alpha:        {:.3}", ctx.time.alpha()));

                    ui.color_picker4_config("clear color", &mut self.clear_color)
                        .picker_mode(imgui::ColorPickerMode::HueWheel)
                        .build();
                });
        });
    }
}

struct AtlasScene;

impl Scene for AtlasScene {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {}

    fn draw(&mut self, _ctx: ContextRef, draw: &mut Draw) {
        draw.imgui(|ui| {
            ui.show_demo_window(&mut true);
            ui.show_about_window(&mut true);
        });
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
                .with_size((1280, 720))
                .with_scene(
                    "initial",
                    S {
                        pos: Vector2::new(10.0, 10.0),
                        prev_pos: Vector2::new(10.0, 10.0),
                        vel: Vector2::zero(),
                        fps_history: vec![0.0; 120],
                        clear_color: [0.0, 0.0, 0.0, 0.0],
                    },
                )
                .with_active_scene("initial"),
        )
        .with_window(
            WindowBuilder::new()
                .with_size((1024, 1024))
                .with_scene("atlas", AtlasScene)
                .with_active_scene("atlas"),
        )
        .build()
        .run();
}
