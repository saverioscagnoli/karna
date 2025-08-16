use karna::{
    input::MouseButton,
    math::{rng, Vec2},
    App, Color, Context, Scene,
};

struct Particle {
    pos: Vec2,
    vel: Vec2,
}

struct S {
    particles: Vec<Particle>,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_vsync(false);
        ctx.time.set_target_fps(120);
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.mouse_held(MouseButton::Left) {
            let mouse_pos = ctx.input.mouse_position();

            for _ in 0..rng(25..=1000) {
                self.particles.push(Particle {
                    pos: mouse_pos,
                    vel: Vec2::new(rng(-1.0..=1.0), rng(-1.0..=1.0)),
                });
            }
        }

        for particle in self.particles.iter_mut() {
            particle.pos += particle.vel;
        }

        println!(
            "fps: {}, particles: {}",
            ctx.time.fps(),
            self.particles.len()
        );
    }

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.set_draw_color(Color::Cyan);

        for particle in self.particles.iter() {
            ctx.render.draw_pixel(particle.pos);
        }
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene(
            "default",
            S {
                particles: Vec::new(),
            },
        )
        .run()
        .expect("Failed to run application");
}
