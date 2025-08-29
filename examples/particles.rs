use karna::{
    input::MouseButton,
    math::{rng, Vec2},
    App, Color, Context, Point, Rect, Scene,
};

struct Particle {
    p: Point,
    vel: Vec2,
}

struct S {
    color: Color,
    particles: Vec<Particle>,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut Context) {
        ctx.render.set_vsync(false);
        ctx.time.set_target_fps(120);
    }

    fn fixed_update(&mut self, ctx: &mut Context) {}

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
                    p: Point::new(mouse_pos.x, mouse_pos.y).with_color(Color::rgb(r, g, b)),
                    vel: Vec2::new(rng(-2.5..=2.5), rng(-2.5..=2.5)),
                });
            }
        }

        let window_size = ctx.window.size();
        let width = window_size.width as f32;
        let height = window_size.height as f32;

        for particle in self.particles.iter_mut() {
            particle.p.pos += particle.vel;

            // // Bounce off left and right edges
            // if particle.p.pos.x <= 0.0 || particle.p.pos.x >= width {
            //     particle.vel.x = -particle.vel.x;
            //     // Clamp position to stay within bounds
            //     particle.p.pos.x = particle.p.pos.x.clamp(0.0, width);
            // }

            // // Bounce off top and bottom edges
            // if particle.p.pos.y <= 0.0 || particle.p.pos.y >= height {
            //     particle.vel.y = -particle.vel.y;
            //     // Clamp position to stay within bounds
            //     particle.p.pos.y = particle.p.pos.y.clamp(0.0, height);
            // }
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
                color: Color::Red,
            },
        )
        .run()
        .expect("Failed to run application");
}
