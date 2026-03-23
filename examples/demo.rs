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
    fn load(&mut self, ctx: karna::ContextRefMut) {}

    fn update(&mut self, ctx: karna::ContextRefMut) {
        let accel = 5000.0;
        let friction = 0.85;
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
            self.vel.x += accel * dt;
        }

        self.vel *= friction;
        self.pos += self.vel * dt;
    }

    fn draw(&self, ctx: karna::ContextRef, draw: &mut Draw) {
        draw.set_color(Color::Red);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);
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
