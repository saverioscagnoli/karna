use karna::{
    context::Context,
    core::App,
    input::Mouse,
    math::{random::rng, Vec2},
    render::Color,
    traits::{Load, Render, Update},
};

#[derive(Clone, Copy)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    size: u32,
}

struct Game {
    particles: Vec<Particle>,
}

impl Load for Game {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render
            .load_font("default", "examples/assets/font.ttf", 20);
        ctx.render.set_font("default");
    }
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();

        if ctx.input.mouse_down(Mouse::Left) {
            let pos = ctx.input.mouse_position();

            for _ in 0..rng(25, 50) {
                let x = rng(-200.0, 200.0);

                self.particles.push(Particle {
                    pos,
                    size: 2,
                    vel: (x, 0.0).into(),
                });
            }
        }

        let size = ctx.window.size();

        for p in self.particles.iter_mut() {
            p.vel.y += 500.0 * dt;
            p.pos += p.vel * dt;

            if p.pos.y > ctx.window.size().height as f32 {
                p.vel.y *= -0.8;
                p.pos.y = ctx.window.size().height as f32;
            }

            if p.pos.x < 0.0 {
                p.vel.x *= -1.0;
                p.pos.x = 0.0;
            }

            if p.pos.x > size.width as f32 {
                p.vel.x *= -1.0;
                p.pos.x = size.width as f32;
            }
        }
    }
}

impl Render for Game {
    fn render(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::RED);
        ctx.render
            .fill_rects(self.particles.iter().map(|p| (p.pos, (p.size, p.size))));

        ctx.render.fill_text(ctx.time.fps(), (10, 10), Color::WHITE);
        ctx.render.fill_text(
            format!("Particles: {}", self.particles.len()),
            (10, 35),
            Color::WHITE,
        );

        ctx.render.set_color(Color::BLACK);
    }
}

fn main() {
    App::new()
        .unwrap()
        .window("particles", (1280, 720))
        .run(&mut Game { particles: vec![] });
}
