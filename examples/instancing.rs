use karna::prelude::*;

const MAX_PARTICLES: usize = 512 * 1024;
const EMIT_PER_FRAME: usize = 10;
const GRAVITY: f32 = 1.0;
const SPIN_SPEED: f32 = 3.0; // radians/sec the fountain rotates
const RAINBOW_SPEED: f32 = 0.25; // full hue cycles/sec
const PALETTE_SIZE: usize = 64;

struct Particle {
    mesh: Handle<Mesh>,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
}

struct Res {
    geometry: Handle<Geometry>,
    palette: Vec<Handle<Material>>,
    particles: Vec<Particle>,
    rng: u32,
    time: f32,
}

struct ParticleDemo;

fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

fn randf(state: &mut u32) -> f32 {
    (xorshift32(state) & 0x7FFF) as f32 / 0x7FFF as f32
}

/// h in [0,1), full saturation/value.
fn hue_to_rgb(h: f32) -> (f32, f32, f32) {
    let h = h.fract() * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    }
}

impl Scene for ParticleDemo {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let view = ctx.window.size();
        ctx.time.set_target_fps(120);
        scene.set_camera_projection(Projection::standard_3d(view, 60.0, 0.01, 50.0));
        scene.camera_mut().position = Vector3::new(0.0, 1.5, -8.0);

        let geometry = scene.add_geometry(Geometry::cube_sized(0.05));

        // Rainbow palette, hue evenly spread over the wheel.
        let palette = (0..PALETTE_SIZE)
            .map(|i| {
                let (r, g, b) = hue_to_rgb(i as f32 / PALETTE_SIZE as f32);
                // Adapt to your Color API if the constructor differs.
                scene.add_material(MaterialDesc::default().color(Color::rgb(r, g, b)))
            })
            .collect();

        ctx.resources.insert(Res {
            geometry,
            palette,
            particles: Vec::new(),
            rng: 0x12345678,
            time: 0.0,
        });
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let dt = ctx.time.delta();
        let res = ctx.resources.get_mut::<Res>();
        res.time += dt;

        // The fountain nozzle sweeps around the Y axis; current hue
        // follows time so consecutive spawns walk the rainbow.
        let base_angle = res.time * SPIN_SPEED;
        let hue = res.time * RAINBOW_SPEED;
        let material = res.palette[(hue.fract() * PALETTE_SIZE as f32) as usize % PALETTE_SIZE];

        for _ in 0..EMIT_PER_FRAME {
            if res.particles.len() >= MAX_PARTICLES {
                break;
            }
            // Small angular jitter so a frame's particles fan out
            // instead of stacking on one ray.
            let angle = base_angle + (randf(&mut res.rng) - 0.5) * 0.4;
            let radial = 0.6 + randf(&mut res.rng) * 0.4;
            let vx = angle.cos() * radial;
            let vz = angle.sin() * radial;
            let vy = randf(&mut res.rng) * 0.5 + 2.0;

            let mesh = Mesh {
                geometry: res.geometry,
                material,
                transform: Transform::default(),
                color: Color::White,
            };

            let handle = scene.add_mesh(mesh);
            res.particles.push(Particle {
                mesh: handle,
                position: Vector3::zero(),
                velocity: Vector3::new(vx, vy, vz),
            });
        }

        for particle in &mut res.particles {
            particle.velocity.y -= GRAVITY * dt;
            particle.position += particle.velocity * dt;
            if particle.position.y < -2.0 {
                particle.position.y = -1.8;
                particle.velocity.y = -particle.velocity.y;
                particle.velocity *= 0.8;
            }
            let mesh = scene.get_mesh_mut(particle.mesh);
            mesh.transform.set_position(particle.position);
        }
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        let res = ctx.resources.get::<Res>();
        draw.set_layer(Layer::Ui);
        draw.debug_text(format!("particles: {}", res.particles.len()), 10.0, 10.0);
        draw.debug_text(format!("fps: {}", ctx.time.fps()), 10.0, 30.0);
        draw.debug_text(format!("dt: {}", ctx.time.delta()), 10.0, 50.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Particle Instancing")
                .with_size((1280, 720))
                .with_scene("particles", ParticleDemo)
                .with_active_scene("particles"),
        )
        .build()
        .run();
}
