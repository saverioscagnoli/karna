#![allow(unused)]

use karna::App;
use karna::Context;
use karna::Scene;
use karna::SceneView;
use karna::WindowBuilder;
use karna::logging::Config;
use karna::render::Color;
use karna::render::Draw;

struct ImguiDemo {
    show_demo: bool,
    clear_color: [f32; 3],
    counter: i32,
    text: String,
}

impl Scene for ImguiDemo {
    fn load(&mut self, ctx: &mut Context, scene: &mut SceneView) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {
        draw.set_clear_color([
            self.clear_color[0],
            self.clear_color[1],
            self.clear_color[2],
            1.0,
        ]);

        // Some regular engine drawing underneath the UI.
        draw.set_color(Color::Red);
        draw.rect(100.0, 100.0, 80.0, 80.0);

        let fps = ctx.time.fps();
        let dt = ctx.time.delta();

        draw.imgui(|ui| {
            ui.window("karna + imgui")
                .size([360.0, 240.0], karna::imgui::Condition::FirstUseEver)
                .position([40.0, 40.0], karna::imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text(format!("fps: {fps}"));
                    ui.text(format!("dt: {dt:.6}"));
                    ui.separator();

                    if ui.button("counter") {
                        self.counter += 1;
                    }

                    ui.same_line();
                    ui.text(format!("= {}", self.counter));

                    ui.checkbox("show demo window", &mut self.show_demo);
                    ui.color_edit3("clear color", &mut self.clear_color);
                    ui.input_text("text", &mut self.text).build();
                });

            if self.show_demo {
                ui.show_demo_window(&mut self.show_demo);
            }
        });
    }
}

struct A {
    show_about: bool,
    show_debug_log: bool,
}

impl Scene for A {
    fn load(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn update(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {
        draw.imgui(|ui| {
            ui.show_about_window(&mut self.show_about);
            ui.show_debug_log_window(&mut self.show_debug_log);
            ui.show_user_guide();
        });
    }
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("imgui")
                .with_size((1280, 720))
                .with_resizable(true)
                .with_scene(
                    0,
                    ImguiDemo {
                        show_demo: true,
                        clear_color: [0.1, 0.1, 0.12],
                        counter: 0,
                        text: String::new(),
                    },
                )
                .with_active_scene(0),
        )
        .with_window(
            WindowBuilder::new()
                .with_title("imgui 2")
                .with_size((800, 600))
                .with_resizable(true)
                .with_scene(
                    0,
                    A {
                        show_about: true,
                        show_debug_log: true,
                    },
                )
                .with_active_scene(0),
        )
        .build()
        .run();
}
