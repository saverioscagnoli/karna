use karna::prelude::*;

const MAX_PARTICLES: usize = 10_000;
const EMIT_PER_FRAME: usize = 10;
const GRAVITY: f32 = 1.0;

struct Particle {
    mesh: Handle<Mesh>,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
}

struct Res {
    geometry: Handle<Geometry>,
    material: Handle<Material>,
    particles: Vec<Particle>,
    rng: u32,
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

impl Scene for ParticleDemo {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let view = ctx.window.size();

        scene.set_camera_projection(Projection::standard_3d(view, 60.0, 0.01, 50.0));

        scene.camera_mut().position = Vector3::new(0.0, 1.5, -8.0);

        let geometry = scene.add_geometry(Geometry::cube_sized(0.05));

        let material = scene.add_material(MaterialDesc::default().color(Color::White));

        ctx.resources.insert(Res {
            geometry,
            material,
            particles: Vec::new(),
            rng: 0x12345678,
        });
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let dt = ctx.time.delta();

        let res = ctx.resources.get_mut::<Res>();

        // Spawn particles
        for _ in 0..EMIT_PER_FRAME {
            if res.particles.len() >= MAX_PARTICLES {
                break;
            }

            let vx = randf(&mut res.rng) - 0.5;
            let vy = randf(&mut res.rng) * 0.5 + 2.0;
            let vz = randf(&mut res.rng) - 0.5;

            let mut mesh = Mesh {
                geometry: res.geometry,
                material: res.material,
                transform: Transform::default(),
            };

            mesh.transform.set_position(Vector3::zero());

            let handle = scene.add_mesh(mesh);

            res.particles.push(Particle {
                mesh: handle,
                position: Vector3::zero(),
                velocity: Vector3::new(vx, vy, vz),
            });
        }

        // Update particles
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
