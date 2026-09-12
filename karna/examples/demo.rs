#![allow(unused)]

use karna::prelude::*;

const DEMO_SCENE: SceneId = SceneId::new_str("DEMO");

struct DemoScene;

impl Scene for DemoScene {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn update(&mut self, ctx: &mut UpdateContext) {}

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {}
}

fn main() {
    _ = traccia::init(
        traccia::Config::default()
            .with_min_level(traccia::LevelFilter::Debug)
            .with_target(SdlTarget::default()),
    );

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Demo window")
                .with_size((1280, 720))
                .with_scene::<DemoScene>(DEMO_SCENE)
                .with_active_scene(DEMO_SCENE),
        )
        .build()
        .run();
}
