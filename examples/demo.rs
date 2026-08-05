#![allow(unused)]

use karna::prelude::*;

struct Demo;

impl Scene for Demo {
    fn load(ctx: LoadContext, scene: &mut SceneView) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn fixed_update(&mut self, ctx: UpdateContext, scene: &mut SceneView) {}

    fn update(&mut self, ctx: UpdateContext, scene: &mut SceneView) {}

    fn draw(&mut self, ctx: DrawContext, scene: &mut SceneView, draw: &mut Draw) {}
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
