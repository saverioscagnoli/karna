use karna::{input::KeyCode, math::Vec2, App, Color, Context, Scene};

struct S {
    pos: Vec2,
    vel: Vec2,
}

impl Default for S {
    fn default() -> Self {
        Self {
            pos: Vec2::zero(),
            vel: Vec2::zero(),
        }
    }
}

impl Scene for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();

        if ctx.input.key_held(KeyCode::KeyW) {
            self.vel.y = -250.0;
        }

        if ctx.input.key_held(KeyCode::KeyS) {
            self.vel.y = 250.0;
        }

        if ctx.input.key_held(KeyCode::KeyA) {
            self.vel.x = -250.0;
        }

        if ctx.input.key_held(KeyCode::KeyD) {
            self.vel.x = 250.0;
        }

        self.pos += self.vel * dt;
        self.vel *= 0.9;
    }

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.set_draw_color(Color::Cyan);
        ctx.render.fill_rect(self.pos, (50, 50));
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene("default", S::default())
        .run()
        .expect("Failed to run application");
}
