use karna::{input::Key, math::Vec2, render::Color, traits::Scene, App, Context};

struct FirstScene {
    pos: Vec2,
    vel: Vec2,
    using_keyboard: bool,
}

impl Scene for FirstScene {
    fn load(&mut self, _ctx: &mut Context) {
        _ctx.window.set_resizable(true);
    }

    fn update(&mut self, ctx: &mut Context) {
        let speed = 250.0;

        if ctx.input.key_down(Key::W) {
            self.vel.y = -speed;
            self.using_keyboard = true;
        }

        if ctx.input.key_down(Key::S) {
            self.vel.y = speed;
            self.using_keyboard = true;
        }

        if ctx.input.key_down(Key::A) {
            self.vel.x = -speed;
            self.using_keyboard = true;
        }

        if ctx.input.key_down(Key::D) {
            self.vel.x = speed;
            self.using_keyboard = true;
        }

        if !self.using_keyboard {
            let left_stick = ctx.input.left_stick();

            self.vel = Vec2::new(left_stick.x, left_stick.y) * speed;
        }

        self.using_keyboard = false;

        self.vel *= 0.9;
        self.pos += self.vel * ctx.time.delta();
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::RED);
        ctx.render.fill_rect(self.pos, (50, 50));
        ctx.render.set_color(Color::BLACK);

        ctx.render.fill_text(
            format!("pos: ({:.1}, {:.1})", self.pos.x, self.pos.y),
            (10, 10),
            Color::WHITE,
        );

        ctx.render.fill_text(
            format!("vel: ({:.1}, {:.1})", self.vel.x, self.vel.y),
            (10, 30),
            Color::WHITE,
        );
    }
}

fn main() {
    App::window("Press WASD to move!", (800, 600)).run(FirstScene {
        pos: Vec2::new(100.0, 100.0),
        vel: Vec2::ZERO,
        using_keyboard: false,
    });
}
