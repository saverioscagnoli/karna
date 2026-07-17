#![allow(unused)]
use std::f32;

use karna::prelude::*;

const GRID: usize = 48; // vertices per side (GRID x GRID)
const PLANE_SIZE: f32 = 14.0; // world-space extent of the plane

#[derive(Clone, Copy, PartialEq)]
enum BrushMode {
    Raise,
    Smooth,
    Flatten,
}

/// One wave component. Two of these are summed: a radial ripple from the
/// origin and a directional swell.
struct Wave {
    amp: f32,
    freq: f32,
    speed: f32,
}

struct S {
    mesh: Handle<Mesh>,
    geometry: Handle<Geometry>,

    // base (flat) positions in the XZ plane, y = 0
    base: Vec<Vector3<f32>>,
    // per-vertex sculpted height
    offsets: Vec<f32>,
    // static index buffer for the grid
    indices: Vec<u32>,
    // scratch: final heights this frame (needed for normals), then verts
    heights: Vec<f32>,
    verts: Vec<Vertex>,

    time: f32,
    time_scale: f32,
    paused: bool,

    ripple: Wave, // radial, from origin
    swell: Wave,  // directional, along +x

    // sculpting brush (applied at the crosshair ray / ground intersection)
    brush_mode: BrushMode,
    brush_radius: f32,
    brush_strength: f32,
    brush_pos: Option<Vector2<f32>>, // xz of crosshair hit this frame

    height_gradient: bool,
    plane_color: [f32; 3],

    yaw: f32,
    pitch: f32,
}

impl S {
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

    fn wave_height(&self, x: f32, z: f32) -> f32 {
        let dist = (x * x + z * z).sqrt();
        let ripple =
            (dist * self.ripple.freq - self.time * self.ripple.speed).sin() * self.ripple.amp;
        let swell = (x * self.swell.freq + self.time * self.swell.speed).sin() * self.swell.amp;
        ripple + swell
    }

    fn palette(&self, y: f32) -> Vector3<f32> {
        let deep = Vector3::new(0.10, 0.25, 0.55);
        let mid = Vector3::new(0.15, 0.55, 0.30);
        let peak = Vector3::new(0.85, 0.85, 0.90);

        // Normalize y into [0,1] over the plausible height range.
        let span = (self.ripple.amp + self.swell.amp + 2.5).max(0.5);
        let t = ((y / span) * 0.5 + 0.5).clamp(0.0, 1.0);

        if t < 0.5 {
            let k = t * 2.0;
            deep + (mid - deep) * k
        } else {
            let k = (t - 0.5) * 2.0;
            mid + (peak - mid) * k
        }
    }

    fn rebuild_verts(&mut self) {
        let n = self.base.len();

        self.heights.clear();
        self.heights.reserve(n);

        for (idx, p) in self.base.iter().enumerate() {
            self.heights
                .push(self.wave_height(p.x, p.z) + self.offsets[idx]);
        }

        let step = PLANE_SIZE / (GRID as f32 - 1.0);
        let light = Vector3::new(0.45, 1.0, 0.3).normalize();

        self.verts.clear();
        self.verts.reserve(n);

        for (idx, p) in self.base.iter().enumerate() {
            let x = idx % GRID;
            let z = idx / GRID;
            let y = self.heights[idx];

            let hl = self.heights[z * GRID + x.saturating_sub(1)];
            let hr = self.heights[z * GRID + (x + 1).min(GRID - 1)];
            let hd = self.heights[z.saturating_sub(1) * GRID + x];
            let hu = self.heights[(z + 1).min(GRID - 1) * GRID + x];

            let normal = Vector3::new(hl - hr, 2.0 * step, hd - hu).normalize();
            let shade = 0.25 + 0.75 * normal.dot(&light).max(0.0);

            let base_rgb = if self.height_gradient {
                self.palette(y)
            } else {
                Vector3::new(
                    self.plane_color[0],
                    self.plane_color[1],
                    self.plane_color[2],
                )
            };

            let mut color = Vector4::new(
                base_rgb.x * shade,
                base_rgb.y * shade,
                base_rgb.z * shade,
                1.0,
            );

            if let Some(b) = self.brush_pos {
                let d = ((p.x - b.x).powi(2) + (p.z - b.y).powi(2)).sqrt();
                if d < self.brush_radius {
                    let k = 1.0 - d / self.brush_radius;
                    color.x = (color.x + 0.6 * k).min(1.0);
                    color.y = (color.y + 0.25 * k).min(1.0);
                }
            }

            let u = (p.x / PLANE_SIZE) + 0.5;
            let v = (p.z / PLANE_SIZE) + 0.5;

            self.verts.push(Vertex::new(
                Vector3::new(p.x, y, p.z),
                color,
                Vector2::new(u, v),
            ));
        }
    }

    fn crosshair_ground_hit(cam_pos: Vector3<f32>, forward: Vector3<f32>) -> Option<Vector2<f32>> {
        if forward.y.abs() < 1e-4 {
            return None;
        }

        let t = -cam_pos.y / forward.y;

        if t <= 0.0 {
            return None;
        }

        let hit = cam_pos + forward * t;
        let half = PLANE_SIZE * 0.5;

        if hit.x.abs() > half || hit.z.abs() > half {
            return None;
        }

        Some(Vector2::new(hit.x, hit.z))
    }

    fn sculpt(&mut self, center: Vector2<f32>, dt: f32, invert: bool) {
        let step = PLANE_SIZE / (GRID as f32 - 1.0);
        let half = PLANE_SIZE * 0.5;
        let cx = (((center.x + half) / step).round() as usize).min(GRID - 1);
        let cz = (((center.y + half) / step).round() as usize).min(GRID - 1);
        let flatten_to = self.offsets[cz * GRID + cx];

        for (idx, p) in self.base.iter().enumerate() {
            let d = ((p.x - center.x).powi(2) + (p.z - center.y).powi(2)).sqrt();
            if d >= self.brush_radius {
                continue;
            }

            let k = 0.5 + 0.5 * (d / self.brush_radius * f32::consts::PI).cos();

            match self.brush_mode {
                BrushMode::Raise => {
                    let sign = if invert { -1.0 } else { 1.0 };
                    self.offsets[idx] += sign * self.brush_strength * k * dt;
                }
                BrushMode::Smooth => {
                    let x = idx % GRID;
                    let z = idx / GRID;
                    let avg = (self.offsets[z * GRID + x.saturating_sub(1)]
                        + self.offsets[z * GRID + (x + 1).min(GRID - 1)]
                        + self.offsets[z.saturating_sub(1) * GRID + x]
                        + self.offsets[(z + 1).min(GRID - 1) * GRID + x])
                        * 0.25;
                    let rate = (self.brush_strength * 2.0 * k * dt).min(1.0);
                    self.offsets[idx] += (avg - self.offsets[idx]) * rate;
                }
                BrushMode::Flatten => {
                    let rate = (self.brush_strength * 2.0 * k * dt).min(1.0);
                    self.offsets[idx] += (flatten_to - self.offsets[idx]) * rate;
                }
            }
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

        self.rebuild_verts();

        self.geometry = scene.add_geometry(Geometry::new(&self.verts, &self.indices));
        let mat = scene.add_material(MaterialDesc::default().color(Color::White));
        let mesh = Mesh::new(self.geometry, mat);
        self.mesh = scene.add_mesh(mesh);

        let camera = scene.camera_mut();
        camera.position = Vector3::new(0.0, 9.0, -13.0);

        ctx.window.capture_cursor(true);
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let dt = ctx.time.delta();
        if !self.paused {
            self.time += dt * self.time_scale;
        }

        if ctx.input.key_pressed(Keycode::Escape) {
            ctx.window.toggle_cursor_capture();
        }

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

        let move_speed = 8.0;
        {
            let camera = scene.camera_mut();
            let step = move_speed * dt;

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
            if ctx.input.key_held(Keycode::Space) {
                camera.position = camera.position + world_up * step;
            }
            if ctx.input.key_held(Keycode::KeyC) {
                camera.position = camera.position - world_up * step;
            }

            camera.target = camera.position + forward;
            camera.up = up;
        }

        if ctx.input.key_held(Keycode::KeyQ) {
            self.brush_radius = (self.brush_radius - 4.0 * dt).max(0.3);
        }

        if ctx.input.key_held(Keycode::KeyE) {
            self.brush_radius = (self.brush_radius + 4.0 * dt).min(PLANE_SIZE);
        }

        self.brush_pos = None;
        if ctx.window.cursor_captured() {
            let cam_pos = scene.camera().position;
            self.brush_pos = S::crosshair_ground_hit(cam_pos, forward);

            if let Some(center) = self.brush_pos {
                let lmb = ctx.input.mouse_held(MouseButton::Left);
                let rmb = ctx.input.mouse_held(MouseButton::Right);

                if lmb || rmb {
                    self.sculpt(center, dt, rmb && !lmb);
                }
            }
        }

        self.rebuild_verts();
        {
            let geom = scene.get_geometry_mut(self.geometry);
            geom.update(&self.verts, &self.indices);
        }
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        draw.set_layer(Layer::Ui);

        if ctx.window.cursor_captured() {
            let view = draw.viewport().as_f32();
            let (cx, cy) = (view.width * 0.5, view.height * 0.5);
            draw.set_color(Color::rgba(1.0, 1.0, 1.0, 0.8));
            draw.circle_outline(cx, cy, 5.0, 1.5);
            draw.point(cx, cy);
        }

        draw.set_color(Color::White);
        draw.debug_text(
            &format!(
                "fps {}\n[Esc] cursor  [WASD/Space/C] fly  [Q/E] brush size\n[LMB] sculpt  [RMB] inverse",
                ctx.time.fps(),
            ),
            10.0,
            10.0,
        );

        draw.imgui(|ui| {
            ui.window("Terrain").build(|| {
                ui.text(format!("grid {GRID}x{GRID}  verts {}", self.base.len()));

                if ui.collapsing_header("waves", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                    ui.checkbox("paused", &mut self.paused);
                    ui.slider("time scale", 0.0, 3.0, &mut self.time_scale);

                    ui.text("radial ripple");
                    ui.slider("amp##r", 0.0, 3.0, &mut self.ripple.amp);
                    ui.slider("freq##r", 0.0, 5.0, &mut self.ripple.freq);
                    ui.slider("speed##r", 0.0, 8.0, &mut self.ripple.speed);

                    ui.text("directional swell");
                    ui.slider("amp##s", 0.0, 3.0, &mut self.swell.amp);
                    ui.slider("freq##s", 0.0, 5.0, &mut self.swell.freq);
                    ui.slider("speed##s", 0.0, 8.0, &mut self.swell.speed);

                    if ui.button("calm") {
                        self.ripple = Wave {
                            amp: 0.3,
                            freq: 1.0,
                            speed: 1.2,
                        };
                        self.swell = Wave {
                            amp: 0.15,
                            freq: 0.6,
                            speed: 0.8,
                        };
                    }
                    ui.same_line();
                    if ui.button("stormy") {
                        self.ripple = Wave {
                            amp: 1.2,
                            freq: 1.8,
                            speed: 4.5,
                        };
                        self.swell = Wave {
                            amp: 0.9,
                            freq: 1.1,
                            speed: 3.0,
                        };
                    }
                    ui.same_line();
                    if ui.button("still (sculpt canvas)") {
                        self.ripple.amp = 0.0;
                        self.swell.amp = 0.0;
                    }
                }

                if ui.collapsing_header("brush", imgui::TreeNodeFlags::DEFAULT_OPEN) {
                    if ui.radio_button("raise / lower", self.brush_mode == BrushMode::Raise) {
                        self.brush_mode = BrushMode::Raise;
                    }

                    if ui.radio_button("smooth", self.brush_mode == BrushMode::Smooth) {
                        self.brush_mode = BrushMode::Smooth;
                    }

                    if ui.radio_button("flatten", self.brush_mode == BrushMode::Flatten) {
                        self.brush_mode = BrushMode::Flatten;
                    }

                    ui.slider("radius", 0.3, PLANE_SIZE, &mut self.brush_radius);
                    ui.slider("strength", 0.2, 8.0, &mut self.brush_strength);

                    if ui.button("reset ALL sculpting") {
                        for o in self.offsets.iter_mut() {
                            *o = 0.0;
                        }
                    }
                }

                if ui.collapsing_header("color", imgui::TreeNodeFlags::empty()) {
                    ui.checkbox("height gradient", &mut self.height_gradient);
                    if !self.height_gradient {
                        ui.color_picker3("plane color", &mut self.plane_color);
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
                        heights: Vec::new(),
                        verts: Vec::new(),
                        time: 0.0,
                        time_scale: 1.0,
                        paused: false,
                        ripple: Wave {
                            amp: 0.8,
                            freq: 1.2,
                            speed: 3.0,
                        },
                        swell: Wave {
                            amp: 0.4,
                            freq: 0.7,
                            speed: 1.5,
                        },
                        brush_mode: BrushMode::Raise,
                        brush_radius: 2.0,
                        brush_strength: 3.0,
                        brush_pos: None,
                        height_gradient: true,
                        plane_color: [0.2, 0.6, 1.0],
                        yaw: f32::consts::FRAC_PI_2,
                        pitch: -0.4,
                    },
                )
                .with_active_scene("demo"),
        )
        .build()
        .run();
}
