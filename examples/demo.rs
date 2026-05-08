use karna::App;
use karna::KeyCode;
use karna::Scene;
use karna::WindowBuilder;
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

        draw.set_scale(1.0);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);

        draw.set_color(Color::Cyan);

        draw.set_scale(1.5);
        draw.rect(self.pos.x + 100.0, self.pos.y, 50.0, 50.0);

        draw.set_color(Color::Magenta);

        draw.set_scale(2.0);
        draw.rect(self.pos.x, self.pos.y + 100.0, 50.0, 50.0);
        draw.line_v([300.0, 100.0], self.pos + Vector2::new(25.0, 25.0));

        draw.reset_scale();

        draw.set_color(Color::Yellow);
        draw.debug_text(&format!("fps: {}", ctx.time.fps()), 10.0, 10.0);
        draw.debug_text(&format!("dt: {}", ctx.time.delta()), 10.0, 730.0);
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
