use karna::prelude::*;

struct Demo;

impl Scene for Demo {
    fn load(ctx: LoadContext, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn update(&mut self, ctx: UpdateContext, scene: &mut SceneRef) {}

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {}
}

fn main() {
    init_logging(Config::default().with_min_level(LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo")
                .with_scene::<Demo>(SceneId::new_label("demo"))
                .with_active_scene(SceneId::new_label("demo")),
        )
        .build()
        .run();
}
