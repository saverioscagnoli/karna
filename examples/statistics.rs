use karna::{AppBuilder, Scene, WindowBuilder, input::KeyCode};
use renderer::{Color, Geometry, Material, Mesh, MeshHandle, Transform};
use traccia::info;

struct StatsDemo {
    rect: MeshHandle,
    circle: MeshHandle,
    logs_toggle: bool,
    c: usize,
}

impl Scene for StatsDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        let rect = Mesh::new(
            Geometry::rect(50.0, 50.0),
            Material::new_color(Color::Cyan),
            Transform::default().with_position([400.0, 400.0, 0.0]),
        );

        self.rect = ctx.render.add_mesh(rect);

        let circle = Mesh::new(
            Geometry::circle(50.0, 32),
            Material::new_color(Color::Magenta),
            Transform::default().with_position([100.0, 500.0, 0.0]),
        );

        self.circle = ctx.render.add_mesh(circle);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let vel = 250.0;
        let rect = ctx.render.get_mesh_mut(self.rect);

        if ctx.input.key_held(&KeyCode::KeyW) {
            *rect.position_y_mut() -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            *rect.position_y_mut() += vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            *rect.position_x_mut() -= vel * ctx.time.delta();
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            *rect.position_x_mut() += vel * ctx.time.delta();
        }

        if ctx.input.key_pressed(&KeyCode::KeyL) {
            self.logs_toggle = !self.logs_toggle;
        }

        if ctx.input.key_pressed(&KeyCode::Space) {
            self.c += 1;
            info!("This is test log number {}", self.c);
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        let mut y = 10.0;

        ctx.render
            .debug_text(format!("FPS: {:.2}", ctx.profiling.time.fps()), 10.0, y);

        y += 20.0;

        ctx.render
            .debug_text(format!("DT: {:.6}", ctx.profiling.time.delta()), 10.0, y);
        y += 20.0;

        ctx.render.debug_text(
            format!("Draw Calls: {}", ctx.profiling.render.draw_calls()),
            10.0,
            y,
        );
        y += 20.0;

        ctx.render.debug_text(
            format!("Vertices: {}", ctx.profiling.render.vertices()),
            10.0,
            y,
        );
        y += 20.0;

        ctx.render.debug_text(
            format!("Indices: {}", ctx.profiling.render.indices()),
            10.0,
            y,
        );
        y += 20.0;

        ctx.render.debug_text(
            format!("Triangles: {}", ctx.profiling.render.triangles()),
            10.0,
            y,
        );

        y += 20.0;

        ctx.render.debug_text(
            format!(
                "Allocated: {:.2} MB",
                ctx.profiling.mem.current() as f32 / 1024.0 / 1024.0
            ),
            10.0,
            y,
        );

        y += 20.0;

        ctx.render.debug_text(
            format!(
                "Peak mem: {:.2} MB",
                ctx.profiling.mem.peak() as f32 / 1024.0 / 1024.0
            ),
            10.0,
            y,
        );

        y += 20.0;

        ctx.render
            .debug_text(format!("CPU: {}", ctx.info.cpu_model()), 10.0, y);

        y += 20.0;

        ctx.render
            .debug_text(format!("CPU cores: {}", ctx.info.cpu_cores()), 10.0, y);

        y += 20.0;

        ctx.render.debug_text(
            format!(
                "MEM: {:.2} GB",
                ctx.info.mem_total() as f32 / 1024.0 / 1024.0 / 1024.0
            ),
            10.0,
            y,
        );

        y += 20.0;

        ctx.render
            .debug_text(format!("GPU: {}", ctx.info.gpu_model()), 10.0, y);

        y += 20.0;

        ctx.render.debug_text(
            format!("Resolution: {}x{}", ctx.window.width(), ctx.window.height()),
            10.0,
            y,
        );

        y += 20.0;

        ctx.render.debug_text(
            format!(
                "Mesh instance buffer writes: {}",
                ctx.profiling.render.instance_writes()
            ),
            10.0,
            y,
        );

        y += 20.0;

        ctx.render.debug_text(
            format!(
                "Geometry buffers: {} ({} kb)",
                ctx.profiling.render.geometry_buffers(),
                ctx.profiling.render.geometry_buffers_size() as f32 / 1024.0
            ),
            10.0,
            y,
        );

        y += 20.0;

        ctx.render.debug_text("Press L to see logs!", 10.0, y);

        if self.logs_toggle {
            ctx.render.debug_logs(400.0);
        }
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Statistics demo")
                .with_size((1920, 1080))
                .with_resizable(false)
                .with_initial_scene(StatsDemo {
                    rect: MeshHandle::dummy(),
                    circle: MeshHandle::dummy(),
                    logs_toggle: false,
                    c: 0,
                }),
        )
        .build()
        .run();
}
