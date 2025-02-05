use karna::{
    input::Mouse,
    math::{rng, Vec2},
    render::Color,
    traits::Scene,
    App, Context,
};

struct Particle {
    pos: Vec2,
    vel: Vec2,
}

struct S {
    particles: Vec<Particle>,
}

impl Scene<Context> for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.mouse_down(Mouse::Left) {
            let mouse_pos = ctx.input.mouse_position();

            for _ in 0..rng(25..=50) {
                self.particles.push(Particle {
                    pos: mouse_pos,
                    vel: Vec2::new(rng(-1.0..=1.0), rng(-1.0..=1.0)),
                });
            }
        }

        for particle in self.particles.iter_mut() {
            particle.pos += particle.vel;
        }
    }

    fn draw(&self, ctx: &mut Context) {
        ctx.render.clear_background(Color::BLACK);

        ctx.render.set_color(Color::CYAN);

        for particle in self.particles.iter() {
            ctx.render.draw_pixel_v(particle.pos);
        }
    }
}

fn main() {
    App::window("Demo window", (800, 600)).run(S {
        particles: Vec::new(),
    });
}
