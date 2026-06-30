use karna::App;
use karna::ContextRef;
use karna::ContextRefMut;
use karna::Scene;
use karna::WindowBuilder;
use karna::input::KeyCode;
use karna::logging;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use log::info;
use math::Vector;

struct S {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextRefMut) {
        info!("Scene loaded");
        ctx.time.set_target_fps(u32::MAX);
    }

    fn fixed_update(&mut self, ctx: ContextRefMut) {
        const VEL: f32 = 250.0;

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y = -VEL;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x = -VEL;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y = VEL;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x = VEL;
        }

        self.pos += self.vel * ctx.time.fixed_delta();
        self.vel *= 0.9;

        if self.vel.length_sq() < 0.01 {
            self.vel.set([0.0, 0.0]);
        }

        println!("fps {}", ctx.time.fps());
    }

    fn update(&mut self, ctx: ContextRefMut) {}

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {
        draw.set_color(Color::Cyan);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);
    }
}

fn main() {
    logging::init(logging::Config::default()).expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_scene(
                    "initial",
                    S {
                        pos: Vector2::new(10.0, 10.0),
                        vel: Vector2::zero(),
                    },
                )
                .with_active_scene("initial"),
        )
        .build()
        .run();
}
