use karna::prelude::*;

const DEMO_SCENE: SceneId = SceneId::new_str("demo");

struct Demo;

impl Scene for Demo {
    fn load(ctx: LoadContext) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);
        Self
    }

    fn fixed_update(&mut self, ctx: UpdateContext) {
        println!("{}", ctx.time.delta());
    }

    fn update(&mut self, ctx: UpdateContext) {}

    fn draw(&mut self, ctx: DrawContext) {}
}

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Trace));

    AppBuilder::default()
        .with_window(
            WindowBuilder::default()
                .with_title("Demo Window")
                .with_size((1280, 720))
                .with_scene::<Demo>(DEMO_SCENE)
                .with_active_scene(DEMO_SCENE),
        )
        .build()
        .run();
}
