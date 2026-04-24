use karna::App;
use karna::KeyCode;
use karna::Scene;
use karna::WindowBuilder;
use karna::log::info;
use karna::math::Size;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;

struct S {
    pos: Vector2,
    vel: Vector2,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {
        ctx.window.set_title(format!("fps: {}", ctx.time.fps()));

        let accel = 5000.0;
        let dt = ctx.time.delta();

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y -= accel * dt;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x -= accel * dt;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y += accel * dt;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x += accel * dt
        }

        self.vel *= 0.85f32.powf(60.0).powf(dt);
        self.pos += self.vel * dt;
    }

    fn draw(&self, ctx: karna::ContextRef, draw: &mut Draw) {
        draw.set_color(Color::White);
        draw.circle(self.pos.x, self.pos.y, 50.0);

        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.point(i as f32 * 10.0, j as f32 * 10.0);
            }
        }

        draw.set_color(Color::Magenta);
        draw.line_v([300.0, 100.0], self.pos);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_initial_scene(S {
                    pos: Vector2::new(50.0, 50.0),
                    vel: Vector2::zeros(),
                }),
        )
        .build()
        .run();
}
