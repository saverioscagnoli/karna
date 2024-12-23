use karna::{
    input::Mouse,
    math::{rng, Size, Vec2},
    render::Color,
    shaders::{Shader, ShaderKind, Uniform},
    traits::Scene,
    App, Context,
};

#[derive(Clone, Copy)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    size: u32,
}

struct FirstScene {
    particles: Vec<Particle>,
}

impl Scene for FirstScene {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.load_shader(
            "square",
            Shader::from_str(include_str!("./assets/sq.vert"), ShaderKind::Vertex),
            Shader::from_str(include_str!("./assets/sq.frag"), ShaderKind::Fragment),
        )
    }

    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();

        if ctx.input.mouse_down(Mouse::Left) {
            let pos = ctx.input.mouse_position();

            for _ in 0..rng(25..=50) {
                let x = rng(-200.0..=200.0);

                self.particles.push(Particle {
                    pos,
                    size: 2,
                    vel: Vec2::new(x, 0.0),
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

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_shader("square");
        ctx.render.set_shader_uniform(
            "time",
            Uniform::Float(ctx.time.started_at().elapsed().as_secs_f32()),
        );
        ctx.render.set_color(Color::WHITE);

        ctx.render
            .fill_rects(self.particles.iter().map(|p| (p.pos, (p.size, p.size))));

        ctx.render.reset_shader();
        ctx.render.set_color(Color::BLACK);

        ctx.render.fill_text(ctx.time.fps(), (10, 10), Color::WHITE);
        ctx.render.fill_text(
            format!("Particles: {}", self.particles.len()),
            (10, 30),
            Color::WHITE,
        );
    }
}

fn main() {
    App::window("particles", Size::new(1280.0, 720.0)).run(FirstScene { particles: vec![] });
}
