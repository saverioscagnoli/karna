use karna::{AppBuilder, Scene, WindowBuilder};

struct StatsDemo;

impl Scene for StatsDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render
            .debug_text(format!("FPS: {:.2}", ctx.profiling.time.fps()), 10.0, 10.0);

        ctx.render
            .debug_text(format!("DT: {:.4}", ctx.profiling.time.delta()), 10.0, 30.0);

        ctx.render.debug_text(
            format!("Draw Calls: {}", ctx.profiling.render.draw_calls()),
            10.0,
            70.0,
        );

        ctx.render.debug_text(
            format!("Vertices: {}", ctx.profiling.render.vertices()),
            10.0,
            90.0,
        );

        ctx.render.debug_text(
            format!("Indices: {}", ctx.profiling.render.vertices()),
            10.0,
            110.0,
        );

        ctx.render.debug_text(
            format!(
                "Allocated: {:.2} MB",
                ctx.profiling.mem.current() as f32 / 1024.0 / 1024.0
            ),
            10.0,
            130.0,
        );
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Statistics demo")
                .with_resizable(false)
                .with_initial_scene(StatsDemo),
        )
        .build()
        .run();
}
