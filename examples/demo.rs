#![allow(unused)]

use karna::App;
use karna::ContextRef;
use karna::Scene;
use karna::WindowBuilder;
use karna::logging::Config;
use karna::logging::LevelFilter;
use karna::render::Draw;
use karna::render::SceneRef;

struct Demo {}

impl Scene for Demo {
    fn load(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        ctx.window.set_resizable(true);
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        println!("fps {} dt {}", ctx.time.fps(), ctx.time.delta());
    }

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {}
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo")
                .with_scene(0, Demo {}, true),
        )
        .build()
        .run();
}
