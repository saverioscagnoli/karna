use karna::{
    AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder,
    input::KeyCode,
    math::Vector3,
    render::{Color, Geometry, Layer, Material, Mesh, Transform3d},
    utils::Handle,
};
use renderer::Projection;

struct ExtremeStress {
    meshes: Vec<Handle<Mesh>>,
    grid_size: usize,
    time: f32,
    wave_speed: f32,
    paused: bool,
    show_all: bool,
    target_count: usize,
}

impl Default for ExtremeStress {
    fn default() -> Self {
        Self {
            meshes: Vec::new(),
            grid_size: 316, // ~100k instances (316x316 = 99,856)
            time: 0.0,
            wave_speed: 2.0,
            paused: false,
            show_all: true,
            target_count: 100000,
        }
    }
}

impl Scene for ExtremeStress {
    fn load(&mut self, ctx: &mut Context) {
        ctx.time.set_target_fps(175);

        self.spawn_grid(ctx);

        // Setup camera
        let camera = ctx.scene.camera_mut();
        camera.set_projection(Projection::standard_3d(
            ctx.window.size(),
            75.0,
            0.1,
            2000.0,
        ));
        camera.set_position([0.0, 100.0, 150.0]);
        camera.look_at(Vector3::new(0.0, 0.0, 0.0));
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.key_pressed(&KeyCode::Space) {
            self.paused = !self.paused;
        }

        if ctx.input.key_pressed(&KeyCode::KeyV) {
            self.show_all = !self.show_all;
        }

        if ctx.input.key_pressed(&KeyCode::Equal) || ctx.input.key_pressed(&KeyCode::NumpadAdd) {
            self.add_instances(ctx, 10000);
        }

        if ctx.input.key_pressed(&KeyCode::Minus) || ctx.input.key_pressed(&KeyCode::NumpadSubtract)
        {
            self.remove_instances(ctx, 10000);
        }

        if ctx.input.key_pressed(&KeyCode::Digit1) {
            self.set_instance_count(ctx, 50000);
        }
        if ctx.input.key_pressed(&KeyCode::Digit2) {
            self.set_instance_count(ctx, 100000);
        }
        if ctx.input.key_pressed(&KeyCode::Digit3) {
            self.set_instance_count(ctx, 150000);
        }
        if ctx.input.key_pressed(&KeyCode::Digit4) {
            self.set_instance_count(ctx, 200000);
        }
        if ctx.input.key_pressed(&KeyCode::Digit5) {
            self.set_instance_count(ctx, 250000);
        }

        if ctx.input.key_pressed(&KeyCode::KeyR) {
            self.reset(ctx);
        }

        if !self.paused {
            self.time += ctx.time.delta();

            let grid_size = (self.meshes.len() as f32).sqrt() as usize;

            for (i, handle) in self.meshes.iter().enumerate() {
                if let Some(mesh) = ctx.scene.get_mesh_mut(*handle) {
                    let x = (i % grid_size) as f32;
                    let z = (i / grid_size) as f32;

                    let center = grid_size as f32 / 2.0;
                    let dx = x - center;
                    let dz = z - center;
                    let dist = (dx * dx + dz * dz).sqrt();

                    let wave = (dist * 0.3 - self.time * self.wave_speed).sin() * 2.0;

                    mesh.position_mut().y = wave;

                    if i % 2 == 0 {
                        *mesh.rotation_z_mut() = self.time * 0.3;
                    }
                }
            }

            // Camera orbit
            let camera = ctx.scene.camera_mut();
            let radius = 150.0 + (grid_size as f32 * 0.1);
            let cam_x = (self.time * 0.2).sin() * radius;
            let cam_z = (self.time * 0.2).cos() * radius;
            let cam_y = 100.0 + (self.time * 0.3).sin() * 30.0;

            camera.set_position([cam_x, cam_y, cam_z]);
            camera.look_at(Vector3::new(0.0, 0.0, 0.0));
        }
    }

    fn render(&mut self, ctx: &RenderContext, draw: &mut Draw) {
        draw.set_layer(Layer::Ui);
        draw.set_color(Color::White);

        let fps = ctx.time.fps();
        let frame_time = ctx.time.delta() * 1000.0;

        draw.debug_text(format!("FPS: {}", fps), 10.0, 10.0);
        draw.debug_text(format!("Frame Time: {:.2}ms", frame_time), 10.0, 30.0);
        draw.debug_text(format!("Instances: {}", self.meshes.len()), 10.0, 50.0);
        draw.debug_text(
            format!(
                "Instance Writes: {}",
                ctx.profiling.render.instance_writes()
            ),
            10.0,
            70.0,
        );

        if self.show_all {
            draw.debug_text("", 10.0, 120.0);
            draw.debug_text("Controls:", 10.0, 140.0);
            draw.debug_text("  [SPACE] - Pause/Resume", 10.0, 160.0);
            draw.debug_text("  [V] - Toggle UI", 10.0, 180.0);
            draw.debug_text("  [+/-] - Add/Remove 10k instances", 10.0, 200.0);
            draw.debug_text("  [R] - Reset to 100k", 10.0, 220.0);
            draw.debug_text("", 10.0, 240.0);
            draw.debug_text("Presets:", 10.0, 260.0);
            draw.debug_text("  [1] - 50k instances", 10.0, 280.0);
            draw.debug_text("  [2] - 100k instances", 10.0, 300.0);
            draw.debug_text("  [3] - 150k instances", 10.0, 320.0);
            draw.debug_text("  [4] - 200k instances", 10.0, 340.0);
            draw.debug_text("  [5] - 250k instances", 10.0, 360.0);
        } else {
            draw.debug_text("[V] - Show Controls", 10.0, 120.0);
        }

        if self.paused {
            draw.debug_text("", 10.0, 400.0);
            draw.debug_text("PAUSED", 10.0, 420.0);
        }
    }
}

impl ExtremeStress {
    fn spawn_grid(&mut self, ctx: &mut Context) {
        let spacing = 0.8;
        let offset = (self.grid_size as f32 * spacing) / 2.0;

        for z in 0..self.grid_size {
            for x in 0..self.grid_size {
                let pos_x = x as f32 * spacing - offset;
                let pos_z = z as f32 * spacing - offset;

                let t = (x + z) as f32 / (self.grid_size * 2) as f32;
                let hue = t * 360.0;
                let color = Color::hsv(hue, 0.6, 0.8);

                let mesh = Mesh::new(
                    Geometry::cube(0.4),
                    Material::new_color(color),
                    Transform3d::default()
                        .with_position([pos_x, 0.0, pos_z])
                        .with_scale([1.0, 1.0, 1.0]),
                );

                self.meshes.push(ctx.scene.add_mesh(mesh));
            }
        }
    }

    fn add_instances(&mut self, ctx: &mut Context, count: usize) {
        let start = self.meshes.len();
        let new_grid_size = ((start + count) as f32).sqrt().ceil() as usize;
        let spacing = 0.8;
        let offset = (new_grid_size as f32 * spacing) / 2.0;

        for i in start..(start + count) {
            let x = i % new_grid_size;
            let z = i / new_grid_size;

            let pos_x = x as f32 * spacing - offset;
            let pos_z = z as f32 * spacing - offset;

            let t = (x + z) as f32 / (new_grid_size * 2) as f32;
            let hue = t * 360.0;
            let color = Color::hsv(hue, 0.6, 0.8);

            let mesh = Mesh::new(
                Geometry::cube(0.4),
                Material::new_color(color),
                Transform3d::default().with_position([pos_x, 0.0, pos_z]),
            );

            self.meshes.push(ctx.scene.add_mesh(mesh));
        }

        self.grid_size = new_grid_size;
    }

    fn remove_instances(&mut self, ctx: &mut Context, count: usize) {
        let to_remove = count.min(self.meshes.len());

        for _ in 0..to_remove {
            if let Some(handle) = self.meshes.pop() {
                ctx.scene.remove_mesh(handle);
            }
        }

        self.grid_size = (self.meshes.len() as f32).sqrt().ceil() as usize;
    }

    fn set_instance_count(&mut self, ctx: &mut Context, target: usize) {
        let current = self.meshes.len();

        if target > current {
            self.add_instances(ctx, target - current);
        } else if target < current {
            self.remove_instances(ctx, current - target);
        }
    }

    fn reset(&mut self, ctx: &mut Context) {
        for handle in self.meshes.drain(..) {
            ctx.scene.remove_mesh(handle);
        }

        self.grid_size = 316;
        self.target_count = 100000;
        self.spawn_grid(ctx);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_title("Instancing Example: EXTREME Stress Test")
                .with_label("main")
                .with_size((1280, 720))
                .with_resizable(false)
                .with_initial_scene(ExtremeStress::default()),
        )
        .build()
        .run();
}
