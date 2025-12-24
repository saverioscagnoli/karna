use karna::{App, Scene, WindowBuilder, input::KeyCode, utils::Lazy};
use math::{Easing, LoopMode, Tween, Vector2};
use renderer::{Color, Geometry, Material, Mesh, Text, Transform};
use std::time::Duration;
use utils::label;

struct TweenDemo {
    rect: Mesh,
    tween: Lazy<Tween<Vector2>>,
    ui_text: Text,
    fps_text: Text,
    easing_text: Text,
    ui_panel: Mesh,
    ui_tween: Lazy<Tween<Vector2>>,
    ui_panel_initialized: bool,
}

impl Scene for TweenDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.time.set_target_fps(120);
        ctx.render.set_clear_color(Color::Black);

        let window_size = ctx.window.size();
        let target = Vector2::new(
            window_size.width() as f32 - 100.0,
            window_size.height() as f32 - 100.0,
        );

        self.tween.set(
            Tween::new(
                *self.rect.position(),
                target,
                Easing::Linear,
                Duration::from_secs_f32(3.0),
            )
            .with_loop_mode(LoopMode::Repeat),
        );

        self.tween.on_complete(move |tween| {
            tween.set_easing(Easing::random());
            tween.set_target(target);
        });

        self.tween.start();

        self.ui_text.set_position([10.0, 10.0]);
        self.fps_text.set_position([10.0, 30.0]);
        self.easing_text.set_position([10.0, 50.0]);

        // Panel dimensions
        let panel_width = 800.0;
        let panel_height = 500.0;

        // Center horizontally, start below the screen
        let center_x = ctx.window.width() as f32 / 2.0 - panel_width / 2.0;
        let initial_panel_y = ctx.window.height() as f32; // Start just below screen
        let initial_panel_pos = Vector2::new(center_x, initial_panel_y);

        self.ui_panel.set_position(initial_panel_pos);

        // Target: centered on screen
        let target_panel_y = ctx.window.height() as f32 / 2.0 - panel_height / 2.0;
        let target_panel = Vector2::new(center_x, target_panel_y);

        self.ui_tween.set(Tween::new(
            initial_panel_pos,
            target_panel,
            Easing::QuadOut,
            Duration::from_millis(150),
        ));
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        if ctx.time.elapsed().as_secs() % 1 == 0 {
            self.fps_text.set_content(format!("fps {}", ctx.time.fps()));
        }

        if ctx.input.key_pressed(&KeyCode::KeyE) {
            if !self.ui_panel_initialized {
                self.ui_tween.start();
                self.ui_panel_initialized = true;
            } else {
                self.ui_tween.toggle_direction();
            }
        }

        if !self.tween.is_complete() {
            self.tween.update(ctx.time.delta());
            *self.rect.position_mut() = self.tween.value();
        }

        self.easing_text
            .set_content(format!("easing: {}", self.tween.easing().to_string()));

        if !self.ui_tween.is_complete() {
            self.ui_tween.update(ctx.time.delta());
            *self.ui_panel.position_mut() = self.ui_tween.value();
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_mesh(&self.rect);
        ctx.render.draw_mesh(&self.ui_panel);
        ctx.render.draw_text(&mut self.ui_text);
        ctx.render.draw_text(&mut self.fps_text);
        ctx.render.draw_text(&mut self.easing_text);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("tween demo")
                .with_resizable(false)
                .with_size((1280, 720))
                .with_initial_scene(TweenDemo {
                    rect: Mesh::new(
                        Geometry::rect(50.0, 50.0),
                        Material::new_color(Color::Cyan),
                        Transform::default().with_position([100.0, 100.0]),
                    ),
                    tween: Lazy::new(),
                    ui_text: Text::new(label!("debug"))
                        .with_content("press 'E' to show the panel!"),
                    fps_text: Text::new(label!("debug")).with_content("fps 0"),
                    easing_text: Text::new(label!("debug")).with_content("easing: Linear"),
                    ui_panel: Mesh::new(
                        Geometry::rect(800.0, 500.0),
                        Material::new_color(Color::Red),
                        Transform::default(),
                    ),
                    ui_tween: Lazy::new(),
                    ui_panel_initialized: false,
                }),
        )
        .build()
        .run();
}
