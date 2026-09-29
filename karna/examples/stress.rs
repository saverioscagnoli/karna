use std::time::Instant;

use karna::prelude::*;

const STRESS: SceneId = SceneId::new_str("stress");

struct Config {
    meshes: usize,
    moving: usize,
    spawns: usize,
    animate: bool,
    transparent: usize,
    seconds: f32,
    close: bool,
}

impl Config {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let arg = |i: usize, default: f32| {
            args.get(i)
                .and_then(|a| a.parse::<f32>().ok())
                .unwrap_or(default)
        };

        Self {
            meshes: arg(0, 50_000.0) as usize,
            moving: arg(1, 5.0) as usize,
            spawns: arg(2, 20.0) as usize,
            animate: arg(3, 1.0) != 0.0,
            transparent: arg(4, 1.0) as usize,
            seconds: arg(5, 0.0),
            close: arg(6, 0.0) != 0.0,
        }
    }
}

struct Stats {
    last: Instant,
    window_start: Instant,
    started: Instant,
    frames: u32,
    total_frames: u64,
    sum_ms: f64,
    max_ms: f64,
    fps: f64,
    avg_ms: f64,
    worst_ms: f64,
}

impl Stats {
    fn new() -> Self {
        let now = Instant::now();

        Self {
            last: now,
            window_start: now,
            started: now,
            frames: 0,
            total_frames: 0,
            sum_ms: 0.0,
            max_ms: 0.0,
            fps: 0.0,
            avg_ms: 0.0,
            worst_ms: 0.0,
        }
    }

    fn tick(&mut self) -> bool {
        let now = Instant::now();
        let ms = (now - self.last).as_secs_f64() * 1000.0;
        self.last = now;

        self.frames += 1;
        self.total_frames += 1;
        self.sum_ms += ms;
        self.max_ms = self.max_ms.max(ms);

        let window = (now - self.window_start).as_secs_f64();

        if window < 1.0 {
            return false;
        }

        self.fps = self.frames as f64 / window;
        self.avg_ms = self.sum_ms / self.frames as f64;
        self.worst_ms = self.max_ms;

        self.frames = 0;
        self.sum_ms = 0.0;
        self.max_ms = 0.0;
        self.window_start = now;
        true
    }
}

struct Stress {
    config: Config,
    stats: Stats,
    geometries: Vec<Handle<Geometry>>,
    materials: Vec<Handle<Material>>,
    glass: Handle<Material>,
    pulse: Handle<Material>,
    grid: Vec<Handle<Mesh>>,
    bullets: Vec<(Handle<Mesh>, f32)>,
    spawned: u64,
    time: f32,
    orbit: bool,
    angle: f32,
}

impl Stress {
    fn spacing() -> f32 {
        1.6
    }

    fn side(&self) -> usize {
        (self.config.meshes.max(1) as f32).sqrt().ceil() as usize
    }

    fn grid_mesh(&self, i: usize) -> Mesh {
        let side = self.side();
        let (x, z) = (i % side, i / side);
        let half = side as f32 * Self::spacing() * 0.5;

        let geometry = self.geometries[i % self.geometries.len()];
        let material = if self.config.transparent > 0 && i % 100 < self.config.transparent {
            self.glass
        } else if i % 17 == 0 {
            self.pulse
        } else {
            self.materials[(i / 3) % self.materials.len()]
        };

        let mut mesh = Mesh::new(geometry, material);
        mesh.set_position(vec3!(
            x as f32 * Self::spacing() - half,
            0.0,
            z as f32 * Self::spacing() - half
        ));
        mesh.set_rotation(Quaternion::from_rotation_y(i as f32 * 0.37));
        mesh
    }

    fn fill(&mut self, scene: &mut SceneHandle<'_>) {
        while self.grid.len() < self.config.meshes {
            let mesh = self.grid_mesh(self.grid.len());
            self.grid.push(scene.spawn(mesh));
        }

        while self.grid.len() > self.config.meshes {
            if let Some(h) = self.grid.pop() {
                scene.despawn(h);
            }
        }
    }

    fn update_camera(&self, scene: &mut SceneHandle<'_>) {
        let camera = scene.camera_mut(Layer::WORLD);

        if self.config.close {
            let eye = vec3!(self.angle.cos() * 20.0, 6.0, self.angle.sin() * 20.0);
            let look = vec3!(
                eye.x + (self.angle * 1.7).cos() * 10.0,
                2.0,
                eye.z + (self.angle * 1.7).sin() * 10.0
            );
            camera.set_position(eye);
            camera.set_target(look);
            return;
        }

        let reach = self.side() as f32 * Self::spacing() * 0.6 + 10.0;
        camera.set_position(vec3!(
            self.angle.cos() * reach,
            reach * 0.6,
            self.angle.sin() * reach
        ));
        camera.set_target(vec3!(0.0, 0.0, 0.0));
    }
}

impl Scene for Stress {
    fn load(ctx: &mut LoadContext) -> Self {
        let config = Config::from_args();

        ctx.window.set_present_mode(PresentMode::Immediate);
        ctx.time.set_target_fps(10_000);

        let geometries = vec![
            ctx.assets.add_geometry(Geometry::cube(1.0)),
            ctx.assets.add_geometry(Geometry::sphere(0.6, 16, 10)),
            ctx.assets.add_geometry(Geometry::cylinder(0.5, 1.0, 16)),
            ctx.assets
                .add_geometry(Geometry::cuboid(vec3!(0.4, 1.4, 0.4))),
        ];

        let palette = [
            Color::RED,
            Color::CYAN,
            Color::YELLOW,
            Color::MAGENTA,
            Color::GREEN,
            Color::hex(0xff8800),
            Color::hex(0x8844ff),
            Color::WHITE,
        ];

        let materials = palette
            .iter()
            .map(|c| ctx.assets.add_material(Material::new().with_color(*c)))
            .collect();

        let glass = ctx.assets.add_material(
            Material::new()
                .with_color(Color::rgba(0.4, 0.7, 1.0, 0.4))
                .with_blend(Blend::Alpha),
        );
        let pulse = ctx.assets.add_material(Material::new());

        let mut camera = Camera::new(Projection::Perspective {
            fov: 60f32.to_radians(),
            aspect_ratio: ctx.window.aspect_ratio(),
            near: 0.5,
            far: 2000.0,
        });
        camera.set_position(vec3!(0.0, 50.0, -100.0));
        camera.set_target(vec3!(0.0, 0.0, 0.0));
        ctx.scene.set_camera(Layer::WORLD, camera);

        let mut stress = Self {
            config,
            stats: Stats::new(),
            geometries,
            materials,
            glass,
            pulse,
            grid: Vec::new(),
            bullets: Vec::new(),
            spawned: 0,
            time: 0.0,
            orbit: true,
            angle: 0.8,
        };

        let started = Instant::now();
        stress.fill(&mut ctx.scene);
        println!(
            "[stress] spawned {} meshes in {:.1} ms",
            stress.grid.len(),
            started.elapsed().as_secs_f64() * 1000.0
        );

        stress.update_camera(&mut ctx.scene);
        stress
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();
        self.time += dt;
        let t = self.time;

        if ctx.input.key_pressed(Key::Up) {
            self.config.moving = (self.config.moving + 5).min(100);
        }
        if ctx.input.key_pressed(Key::Down) {
            self.config.moving = self.config.moving.saturating_sub(5);
        }
        if ctx.input.key_pressed(Key::Right) {
            self.config.spawns += 10;
        }
        if ctx.input.key_pressed(Key::Left) {
            self.config.spawns = self.config.spawns.saturating_sub(10);
        }
        if ctx.input.key_pressed(Key::M) {
            self.config.animate = !self.config.animate;
        }
        if ctx.input.key_pressed(Key::Space) {
            self.orbit = !self.orbit;
        }
        if ctx.input.key_pressed(Key::C) {
            self.config.close = !self.config.close;
            self.update_camera(&mut ctx.scene);
        }
        if ctx.input.key_pressed(Key::Equals) {
            self.config.meshes += 10_000;
        }
        if ctx.input.key_pressed(Key::Minus) {
            self.config.meshes = self.config.meshes.saturating_sub(10_000);
        }

        self.fill(&mut ctx.scene);

        if self.orbit {
            self.angle += dt * 0.1;
            self.update_camera(&mut ctx.scene);
        }

        let moving = self.grid.len() * self.config.moving / 100;

        for (i, h) in self.grid.iter().take(moving).enumerate() {
            ctx.scene.mesh_mut(*h).position_mut().y = (t * 2.0 + i as f32 * 0.05).sin() * 0.8;
        }

        if self.config.animate {
            let pulse = ctx.assets.material_mut(self.pulse);
            pulse.color = Color::rgb(
                (t * 2.0).sin() * 0.5 + 0.5,
                (t * 2.0 + 2.094).sin() * 0.5 + 0.5,
                (t * 2.0 + 4.189).sin() * 0.5 + 0.5,
            );
        }

        let reach = self.side() as f32 * Self::spacing() * 0.5;

        for i in 0..self.config.spawns {
            let a = t * 1.7 + i as f32 * 0.618;
            let r = reach * ((i as f32 * 0.37 + t).sin() * 0.5 + 0.5);
            let mut bullet = Mesh::new(self.geometries[1], self.materials[2]);
            bullet.set_position(vec3!(a.cos() * r, 2.0, a.sin() * r));
            bullet.set_scale(vec3!(0.4, 0.4, 0.4));
            self.bullets.push((ctx.scene.spawn(bullet), t));
            self.spawned += 1;
        }

        let scene = &mut ctx.scene;
        self.bullets.retain(|(h, born)| {
            if t - born > 1.0 {
                scene.despawn(*h);
                false
            } else {
                true
            }
        });

        for (h, born) in &self.bullets {
            ctx.scene.mesh_mut(*h).position_mut().y = 2.0 + (t - born) * 6.0;
        }

        if self.stats.tick() {
            println!(
                "[stress] fps {:>6.0}  avg {:>6.2} ms  worst {:>6.2} ms  meshes {:>6}  moving {:>3}%  spawns/frame {:>3}  materials {}",
                self.stats.fps,
                self.stats.avg_ms,
                self.stats.worst_ms,
                ctx.scene.meshes().len(),
                self.config.moving,
                self.config.spawns,
                if self.config.animate {
                    "animated"
                } else {
                    "static"
                },
            );
        }

        if self.config.seconds > 0.0 && self.time >= self.config.seconds {
            let elapsed = self.stats.started.elapsed().as_secs_f64();
            println!(
                "[stress] done: {} frames in {:.1} s, average {:.0} fps ({:.2} ms), {} spawned",
                self.stats.total_frames,
                elapsed,
                self.stats.total_frames as f64 / elapsed,
                elapsed * 1000.0 / self.stats.total_frames as f64,
                self.spawned,
            );
            std::process::exit(0);
        }
    }

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        draw.set_layer(Layer::UI);
        draw.set_color(Color::WHITE);
        draw.print(
            format!(
                "fps {:.0}  avg {:.2} ms  worst {:.2} ms\nmeshes {}  moving {}%  spawns/frame {}  materials {}",
                self.stats.fps,
                self.stats.avg_ms,
                self.stats.worst_ms,
                ctx.scene.meshes().len(),
                self.config.moving,
                self.config.spawns,
                if self.config.animate { "animated" } else { "static" },
            ),
            10.0,
            10.0,
        );
        draw.print(
            "up/down moving   left/right spawns   +/- 10k meshes   m materials   space orbit   c close camera",
            10.0,
            50.0,
        );
    }
}

fn main() {
    AppBuilder::default()
        .with_window(
            WindowBuilder::new()
                .with_title("Stress")
                .with_size((1280, 720))
                .with_scene::<Stress>(STRESS)
                .with_active_scene(STRESS),
        )
        .with_root("karna/examples/")
        .build()
        .run();
}
