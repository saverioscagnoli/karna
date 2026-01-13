use karna::{
    AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder,
    input::KeyCode,
    math::{Vector3, rng},
    render::{Color, Geometry, Layer, Material, Mesh, Transform3d},
    utils::Handle,
};

const MAX_PARTICLES: usize = 10_000;
const PARTICLES_PER_FRAME: usize = 50;
const GRAVITY: f32 = 9.8;

struct Particle {
    handle: Handle<Mesh>, // Handle to the mesh in the scene
    pos: Vector3,
    vel: Vector3,
    active: bool,
}

struct Demo {
    particles: Vec<Particle>, // Fixed-size pool
    yaw: f32,
    pitch: f32,
    fov: f32,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            yaw: 0.0,
            pitch: 0.0,
            fov: 60.0,
        }
    }
}

impl Scene for Demo {
    fn load(&mut self, ctx: &mut Context) {
        ctx.time.uncap_fps();
        ctx.window.capture_mouse(true);

        // Setup Camera
        let camera = ctx.scene.camera_mut();
        camera.set_projection(karna::render::Projection::standard_3d(
            ctx.window.size(),
            self.fov,
            0.1,
            100.0,
        ));
        camera.set_position([0.0, 3.0, 10.0]);
        camera.look_at([0.0, 2.0, 0.0].into());

        let shared_geometry = Geometry::cube(0.1);

        let palette = [
            Material::new_color(Color::Red),
            Material::new_color(Color::Green),
            Material::new_color(Color::Blue),
            Material::new_color(Color::Yellow),
            Material::new_color(Color::Cyan),
            Material::new_color(Color::Magenta),
        ];

        for i in 0..MAX_PARTICLES {
            let material = palette[i % palette.len()].clone();

            let geometry = shared_geometry.clone();

            let mesh = Mesh::new(
                geometry,
                material,
                Transform3d::default().with_position(Vector3::new(0.0, -1000.0, 0.0)), // Hide initially
            );

            let handle = ctx.scene.add_mesh(mesh);

            self.particles.push(Particle {
                handle,
                pos: Vector3::new(0.0, -1000.0, 0.0),
                vel: Vector3::zeros(),
                active: false,
            });
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();

        // 1. Activate new particles from the pool
        let mut activated_count = 0;
        for p in self.particles.iter_mut() {
            if !p.active && activated_count < PARTICLES_PER_FRAME {
                p.active = true;
                p.pos = Vector3::new(0.0, 0.0, 0.0);

                // Proper cone fountain math using built-in rng
                let angle = rng(0.0..=std::f32::consts::TAU);
                let radius = rng(0.0..=0.5);

                p.vel = Vector3::new(
                    angle.cos() * radius + rng(-0.2..=0.2), // X
                    rng(8.0..=12.0),                        // Y (Up)
                    angle.sin() * radius + rng(-0.2..=0.2), // Z
                );

                activated_count += 1;
            }
        }

        // 2. Update Physics
        for p in self.particles.iter_mut() {
            if !p.active {
                continue;
            }

            p.vel.y -= GRAVITY * dt;
            p.pos += p.vel * dt;

            // Bounce floor
            if p.pos.y < -2.0 {
                p.pos.y = -2.0;
                p.vel.y *= -0.5;
                p.vel.x *= 0.8;
                p.vel.z *= 0.8;
            }

            // Deactivate if stopped
            if p.pos.y <= -1.9 && p.vel.length_squared() < 0.2 {
                p.active = false;
                p.pos.y = -1000.0; // Move out of view
            }

            // 3. Update the mesh in the scene
            // Because we re-use the handles, the engine doesn't need to re-allocate
            if let Some(mesh) = ctx.scene.get_mesh_mut(p.handle) {
                *mesh.position_mut() = p.pos;
            }
        }

        self.update_camera(ctx);
    }

    fn render(&mut self, ctx: &RenderContext, draw: &mut Draw) {
        draw.set_layer(Layer::Ui);
        draw.debug_text(format!("FPS: {:.0}", ctx.time.fps()), 10.0, 10.0);

        let active = self.particles.iter().filter(|p| p.active).count();
        draw.debug_text(format!("Active Particles: {}", active), 10.0, 30.0);

        // If instancing/batching is working, this number should be low (e.g., 6 draw calls for 6 colors)
        draw.debug_text(
            format!("Draw Calls: {}", ctx.profiling.render.draw_calls()),
            10.0,
            50.0,
        );

        draw.set_layer(Layer::World);
    }
}

// Camera logic separated for cleanliness
impl Demo {
    fn update_camera(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();
        let mouse_sensitivity = 0.003;
        let mouse_delta = ctx.input.mouse_delta();

        self.yaw += mouse_delta.x * mouse_sensitivity;
        self.pitch -= mouse_delta.y * mouse_sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);

        let vel = 10.0 * dt;
        let camera = ctx.scene.camera_mut();

        let (sin_y, cos_y) = self.yaw.sin_cos();
        let forward = Vector3::new(sin_y, 0.0, -cos_y);
        let right = Vector3::new(cos_y, 0.0, sin_y);
        let mut move_dir = Vector3::zeros();

        if ctx.input.key_held(&KeyCode::KeyW) {
            move_dir += forward;
        }
        if ctx.input.key_held(&KeyCode::KeyS) {
            move_dir -= forward;
        }
        if ctx.input.key_held(&KeyCode::KeyD) {
            move_dir += right;
        }
        if ctx.input.key_held(&KeyCode::KeyA) {
            move_dir -= right;
        }
        if ctx.input.key_held(&KeyCode::Space) {
            move_dir.y += 1.0;
        }
        if ctx.input.key_held(&KeyCode::ShiftLeft) {
            move_dir.y -= 1.0;
        }

        if move_dir.length_squared() > 0.0 {
            *camera.position_mut() += move_dir.normalized() * vel;
        }

        let (sin_p, cos_p) = self.pitch.sin_cos();
        let look_dir = Vector3::new(sin_y * cos_p, sin_p, -cos_y * cos_p);
        let current_pos = *camera.position();
        camera.look_at(current_pos + look_dir);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Auto-Instanced Particle Fountain")
                .with_size((1280, 720))
                .with_resizable(false)
                .with_initial_scene(Demo::default()),
        )
        .build()
        .run();
}
