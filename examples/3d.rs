use std::f32;

use karna::prelude::*;

const GRID: usize = 32; // vertices per side (GRID x GRID)
const PLANE_SIZE: f32 = 10.0; // world-space extent of the plane

struct S {
    mesh: Handle<Mesh>,
    geometry: Handle<Geometry>,

    // base (flat) positions in the XZ plane, y = 0
    base: Vec<Vector3<f32>>,
    // per-vertex manual height you push up/down via imgui
    offsets: Vec<f32>,
    // static index buffer for the grid
    indices: Vec<u32>,
    // scratch vertex buffer we upload each frame
    verts: Vec<Vertex>,

    time: f32,

    // wave params (tweakable in imgui)
    wave_amp: f32,
    wave_freq: f32,
    wave_speed: f32,

    // vertex the imgui "raise" tools act on
    selected: i32,
    raise_step: f32,

    yaw: f32,
    pitch: f32,
    plane_color: [f32; 3],
}

impl S {
    /// Build the flat grid: positions + indices.
    fn build_grid() -> (Vec<Vector3<f32>>, Vec<u32>) {
        let mut base = Vec::with_capacity(GRID * GRID);
        let mut indices = Vec::with_capacity((GRID - 1) * (GRID - 1) * 6);

        let half = PLANE_SIZE * 0.5;
        let step = PLANE_SIZE / (GRID as f32 - 1.0);

        for z in 0..GRID {
            for x in 0..GRID {
                let px = -half + x as f32 * step;
                let pz = -half + z as f32 * step;
                base.push(Vector3::new(px, 0.0, pz));
            }
        }

        // two triangles per quad cell
        for z in 0..GRID - 1 {
            for x in 0..GRID - 1 {
                let i0 = (z * GRID + x) as u32;
                let i1 = (z * GRID + x + 1) as u32;
                let i2 = ((z + 1) * GRID + x) as u32;
                let i3 = ((z + 1) * GRID + x + 1) as u32;

                indices.extend([i0, i2, i1]);
                indices.extend([i1, i2, i3]);
            }
        }

        (base, indices)
    }

    /// Rebuild `self.verts` from base + wave + manual offsets.
    fn rebuild_verts(&mut self) {
        self.verts.clear();
        self.verts.reserve(self.base.len());

        for (idx, p) in self.base.iter().enumerate() {
            // radial wave based on distance from origin
            let dist = (p.x * p.x + p.z * p.z).sqrt();
            let wave = (dist * self.wave_freq - self.time * self.wave_speed).sin() * self.wave_amp;

            let y = wave + self.offsets[idx];

            // color by height for a bit of visual feedback
            let t = (y / (self.wave_amp.max(0.001) + 3.0)) * 0.5 + 0.5;
            let color = Vector4::new(
                self.plane_color[0] * t + 0.1,
                self.plane_color[1] * (1.0 - t) + 0.1,
                self.plane_color[2],
                1.0,
            );

            // uv in [0,1] across the grid
            let u = (p.x / PLANE_SIZE) + 0.5;
            let v = (p.z / PLANE_SIZE) + 0.5;

            self.verts.push(Vertex::new(
                Vector3::new(p.x, y, p.z),
                color,
                Vector2::new(u, v),
            ));
        }
    }
}

impl Scene for S {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        scene.set_camera_projection(Projection::standard_3d(
            ctx.window.size(),
            75.0,
            0.1,
            1000.0,
        ));

        ctx.time.set_target_fps(120);

        let (base, indices) = S::build_grid();
        self.offsets = vec![0.0; base.len()];
        self.base = base;
        self.indices = indices;

        // initial vertex buffer
        self.rebuild_verts();

        self.geometry = scene.add_geometry(Geometry::new(&self.verts, &self.indices));
        let mat = scene.add_material(MaterialDesc::default().color(Color::Cyan));

        let mesh = Mesh {
            geometry: self.geometry,
            material: mat,
            transform: Transform::default(),
        };

        self.mesh = scene.add_mesh(mesh);

        // start the camera up and back, looking at the plane
        let camera = scene.camera_mut();
        camera.position = Vector3::new(0.0, 8.0, -12.0);

        ctx.window.capture_cursor(true);
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let dt = ctx.time.delta();
        self.time += dt;

        // --- animate + upload the plane every frame ---
        self.rebuild_verts();
        {
            let geom = scene.get_geometry_mut(self.geometry);
            geom.update(&self.verts, &self.indices);
        }

        // --- camera look ---
        let move_speed = 8.0;

        if ctx.window.cursor_captured() {
            let sensitivity = 0.5f32.to_radians();
            let mouse_delta = ctx.input.mouse_delta();
            self.yaw += mouse_delta.x * sensitivity;
            self.pitch -= mouse_delta.y * sensitivity;

            let limit = 89.0f32.to_radians();
            self.pitch = self.pitch.clamp(-limit, limit);
        }

        let forward = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let right = forward.cross(&world_up).normalize();
        let up = right.cross(&forward).normalize();

        let camera = scene.camera_mut();
        let step = move_speed * dt;

        if ctx.input.key_pressed(Keycode::Escape) {
            ctx.window.toggle_cursor_capture();
        }

        if ctx.input.key_held(Keycode::KeyW) {
            camera.position = camera.position + forward * step;
        }

        if ctx.input.key_held(Keycode::KeyS) {
            camera.position = camera.position - forward * step;
        }

        if ctx.input.key_held(Keycode::KeyD) {
            camera.position = camera.position + right * step;
        }

        if ctx.input.key_held(Keycode::KeyA) {
            camera.position = camera.position - right * step;
        }

        camera.target = camera.position + forward;
        camera.up = up;
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        draw.set_layer(Layer::Ui);

        draw.debug_text(
            &format!(
                "fps {}\ndt {}\nPress 'Esc' to toggle cursor",
                ctx.time.fps(),
                ctx.time.delta()
            ),
            10.0,
            10.0,
        );

        draw.imgui(|ui| {
            ui.window("Plane").build(|| {
                ui.text(format!("grid {GRID}x{GRID}  verts {}", self.base.len()));

                ui.separator();
                ui.text("wave");
                ui.slider("amplitude", 0.0, 3.0, &mut self.wave_amp);
                ui.slider("frequency", 0.0, 5.0, &mut self.wave_freq);
                ui.slider("speed", 0.0, 8.0, &mut self.wave_speed);

                ui.separator();
                ui.color_picker3("plane color", &mut self.plane_color);

                ui.separator();
                ui.text("raise a vertex");

                let max = self.base.len() as i32 - 1;
                ui.slider("vertex index", 0, max, &mut self.selected);
                ui.slider("raise step", 0.1, 3.0, &mut self.raise_step);

                let sel = self.selected.clamp(0, max) as usize;

                if ui.button("raise +") {
                    self.offsets[sel] += self.raise_step;
                }
                ui.same_line();
                if ui.button("lower -") {
                    self.offsets[sel] -= self.raise_step;
                }
                ui.same_line();
                if ui.button("reset vertex") {
                    self.offsets[sel] = 0.0;
                }

                ui.text(format!("offset[{sel}] = {:.2}", self.offsets[sel]));

                // direct drag on the selected vertex height
                ui.slider("offset##sel", -5.0, 5.0, &mut self.offsets[sel]);

                if ui.button("reset ALL vertices") {
                    for o in self.offsets.iter_mut() {
                        *o = 0.0;
                    }
                }
            });
        });
    }
}

fn main() {
    karna::logging::init(
        karna::logging::Config {
            min_level: karna::logging::LevelFilter::Debug,
            ..Default::default()
        }
        .hide_wgpu(true),
    )
    .expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_scene(
                    "demo",
                    S {
                        mesh: Handle::default(),
                        geometry: Handle::default(),
                        base: Vec::new(),
                        offsets: Vec::new(),
                        indices: Vec::new(),
                        verts: Vec::new(),
                        time: 0.0,
                        wave_amp: 0.8,
                        wave_freq: 1.2,
                        wave_speed: 3.0,
                        selected: 0,
                        raise_step: 0.5,
                        yaw: f32::consts::FRAC_PI_2,
                        pitch: -0.4,
                        plane_color: [0.2, 0.6, 1.0],
                    },
                )
                .with_active_scene("demo"),
        )
        .build()
        .run();
}
