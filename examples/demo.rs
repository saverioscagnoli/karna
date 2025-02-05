use karna::{input::Key, math::Vec2, render::Color, traits::Scene, App, Context};

struct S {
    pos: Vec2,
    vel: Vec2,
}

impl Scene<Context> for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();

        if ctx.input.key_down(Key::W) {
            self.vel.y = -250.0;
        }

        if ctx.input.key_down(Key::S) {
            self.vel.y = 250.0;
        }

        if ctx.input.key_down(Key::A) {
            self.vel.x = -250.0;
        }

        if ctx.input.key_down(Key::D) {
            self.vel.x = 250.0;
        }

        self.pos += self.vel * dt;
        self.vel *= 0.9;
    }

    fn draw(&self, ctx: &mut Context) {
        ctx.render.clear_background(Color::BLACK);

        ctx.render.set_color(Color::CYAN);
        ctx.render.fill_rect_v(self.pos, (50, 50));
    }
}

fn main() {
    App::window("Demo window", (800, 600)).run(S {
        pos: Vec2::zero(),
        vel: Vec2::zero(),
    });
}
