use karna::{
    App, Context, Scene,
    input::{KeyCode, MouseButton},
    math::Vector2,
    render::{Color, Mesh, MeshGeometry, Transform2D},
};
use math::rng;

struct Pixel {
    mesh: Mesh,
    vel: Vector2,
}

pub struct S {
    rect: Mesh,
    vel: Vector2,
    rects: Vec<Mesh>,
    pixels: Vec<Pixel>,
}

impl Scene for S {
    fn load(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();
        let vel = 300.0;

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y = -vel;
        }
        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x = -vel;
        }
        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y = vel;
        }
        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x = vel;
        }

        self.rect.position += self.vel * dt;
        self.vel *= 0.9;
        if self.vel.length_squared() < 0.001 {
            self.vel.set(0.0, 0.0);
        }

        if ctx.input.mouse_held(&MouseButton::Left) {
            let mouse_position = ctx.input.mouse_position();
            for _ in 0..500 {
                let angle = rng(0.0..std::f32::consts::TAU);
                let speed = rng(50.0..300.0);
                self.pixels.push(Pixel {
                    mesh: Mesh::new(
                        MeshGeometry::pixel(),
                        Color::Cyan,
                        Transform2D::default()
                            .with_position_x(mouse_position.x)
                            .with_position_y(mouse_position.y),
                    ),
                    vel: Vector2::new(angle.cos() * speed, angle.sin() * speed),
                });
            }
        }

        let screen_size = ctx.window.size();
        let screen_width = screen_size.width as f32;
        let screen_height = screen_size.height as f32;

        for pixel in &mut self.pixels {
            // Apply gravity
            pixel.vel.y += 200.0 * dt;

            // Apply friction
            pixel.vel *= 0.98;

            // Update position
            pixel.mesh.position += pixel.vel * dt;

            // Bounce off edges
            if pixel.mesh.position.x < 0.0 && pixel.vel.x < 0.0 {
                pixel.mesh.position.x = 0.0;
                pixel.vel.x *= -0.7;
            }
            if pixel.mesh.position.x > screen_width && pixel.vel.x > 0.0 {
                pixel.mesh.position.x = screen_width;
                pixel.vel.x *= -0.7;
            }
            if pixel.mesh.position.y < 0.0 && pixel.vel.y < 0.0 {
                pixel.mesh.position.y = 0.0;
                pixel.vel.y *= -0.7;
            }
            if pixel.mesh.position.y > screen_height && pixel.vel.y > 0.0 {
                pixel.mesh.position.y = screen_height;
                pixel.vel.y *= -0.7;
            }

            // Stop if moving very slowly
            if pixel.vel.length_squared() < 1.0 {
                pixel.vel.set(0.0, 0.0);
            }
        }

        if ctx.time.elapsed().as_secs_f32() % 1.0 < 0.01 {
            println!("fps {} part {}", ctx.time.fps(), self.pixels.len());
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        self.rect.render(&mut ctx.render);
        for rect in &mut self.rects {
            rect.render(&mut ctx.render);
        }
        for pixel in &mut self.pixels {
            pixel.mesh.render(&mut ctx.render);
        }
    }
}

fn main() {
    App::new()
        .with_initial_scene(
            "default",
            Box::new(S {
                rect: Mesh::new(
                    MeshGeometry::rect(),
                    Color::Red,
                    Transform2D::default()
                        .with_position([10.0, 10.0])
                        .with_scale([50.0, 50.0]),
                ),
                vel: Vector2::zeros(),
                rects: Vec::new(),
                pixels: Vec::new(),
            }),
        )
        .run();
}
