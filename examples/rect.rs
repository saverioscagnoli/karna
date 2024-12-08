use karna::{
    input::Key,
    math::Vec2,
    render::Color,
    traits::{Draw, Load, Update},
    App, Context,
};

struct Game {
    pos: Vec2,
    vel: Vec2,
}

impl Load for Game {
    fn load(&mut self, _ctx: &mut Context) {}
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        let speed = 250.0;

        if ctx.input.key_down(Key::W) {
            self.vel.y = -speed;
        }

        if ctx.input.key_down(Key::S) {
            self.vel.y = speed;
        }

        if ctx.input.key_down(Key::A) {
            self.vel.x = -speed;
        }

        if ctx.input.key_down(Key::D) {
            self.vel.x = speed;
        }

        self.vel *= 0.9;
        self.pos += self.vel * ctx.time.delta();
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::RED);
        ctx.render.fill_rect(self.pos, (50, 50));
        ctx.render.set_color(Color::BLACK);

        ctx.render.fill_text(
            format!("pos: ({:.1}, {:.1})", self.pos.x, self.pos.y),
            (10, 10),
            Color::CYAN,
        );

        ctx.render.fill_text(
            format!("vel: ({:.1}, {:.1})", self.vel.x, self.vel.y),
            (10, 30),
            Color::WHITE,
        );
    }
}

fn main() {
    App::new("basic window", (800, 600))
        .unwrap()
        .run(&mut Game {
            pos: Vec2::new(100, 100),
            vel: Vec2::zero(),
        });
}
