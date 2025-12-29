use karna::{AppBuilder, Scene, WindowBuilder};

struct StatsDemo;

impl Scene for StatsDemo {
    fn load(&mut self, ctx: &mut karna::Context) {}

    fn update(&mut self, ctx: &mut karna::Context) {}

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
                .with_initial_scene(StatsDemo),
        )
        .build()
        .run();
}
