use karna::{
    input::MouseButton,
    math::{rng, Vec2},
    App, Context, Scene,
};

struct Particle {
    p: Pixel,
    vel: Vec2,
}

struct S {
    particles: Vec<Particle>,
}

impl Scene for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.mouse_held(MouseButton::Left) {
            let mouse_pos = ctx.input.mouse_position();

            let time = ctx.time.elapsed();

            // Create animated color using sin for smooth transitions (0-1 range)
            let r = time.sin() * 0.5 + 0.5;
            let g = (time + 2.0).sin() * 0.5 + 0.5;
            let b = (time + 4.0).sin() * 0.5 + 0.5;

            for _ in 0..rng(25..=1000) {
                self.particles.push(Particle {
                    p: Pixel::new(mouse_pos).with_color(Color::rgb(r, g, b)),
                    vel: Vec2::new(rng(-2.5..=2.5), rng(-2.5..=2.5)),
                });
            }
        }

        let size = ctx.window.size();

        for particle in self.particles.iter_mut() {
            // do not update if out of bounds
            if particle.p.position.x < 0.0
                || particle.p.position.x > size.width as f32
                || particle.p.position.y < 0.0
                || particle.p.position.y > size.height as f32
            {
                continue;
            }

            particle.p.position += particle.vel;
        }

        println!(
            "fps: {}, particles: {}",
            ctx.time.fps(),
            self.particles.len()
        );
    }

    fn render(&mut self, ctx: &mut Context) {
        for particle in self.particles.iter() {
            particle.p.render(&mut ctx.render);
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
