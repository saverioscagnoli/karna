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

struct Demo {
    pos: Vector2<f32>,
    prev_pos: Vector2<f32>,
    vel: Vector2<f32>,
    clear_color: [f32; 4],
}

impl Scene for Demo {
    fn load(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        ctx.time.set_target_fps(120);

        ctx.assets.write_scope(|a| {
            a.load_image(include_bytes!("../../examples/assets/pcb.png"));
        })
    }

    fn fixed_update(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        const VEL: f32 = 250.0;

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

    fn update(&mut self, _ctx: ContextMut, _scene: &mut SceneHandle) {}

    fn imgui_frame(&mut self, ctx: ContextMut, _scene: &mut SceneHandle, ui: &imgui::Ui) {
        ui.window("karna on the web")
            .size([300.0, 160.0], imgui::Condition::FirstUseEver)
            .position([10.0, 10.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("FPS: {}", ctx.time.fps()));
                ui.text("WASD moves the square");
                ui.color_edit4("clear color", &mut self.clear_color);
            });
    }

    fn draw(&self, ctx: ContextRef, draw: &mut Draw) {
        draw.set_clear_color(self.clear_color);

        let alpha = ctx.time.alpha();
        let render_pos = self.prev_pos.lerp(&self.pos, alpha);

        draw.set_color(Color::White);
        draw.rect(render_pos.x, render_pos.y, 50.0, 50.0);
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
                .with_title("karna web demo")
                .with_size((1280, 720))
                .with_scene(
                    "initial",
                    Demo {
                        pos: Vector2::new(100.0, 100.0),
                        prev_pos: Vector2::new(100.0, 100.0),
                        vel: Vector2::zero(),
                        clear_color: [0.1, 0.1, 0.12, 1.0],
                    },
                )
                .with_active_scene("initial"),
        )
        .build()
        .run();
}
