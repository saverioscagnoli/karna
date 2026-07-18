use karna::App;
use karna::Context;
use karna::Scene;
use karna::WindowBuilder;
use karna::logging::Config;
use karna::render::Draw;

const DEMO_SCENE: usize = 0;

struct Demo;

impl Scene for Demo {
    fn load(&mut self, ctx: Context) {}

    fn fixed_update(&mut self, ctx: Context) {}

    fn update(&mut self, ctx: Context) {}

    fn draw(&mut self, ctx: Context, draw: &mut Draw) {}
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Demo window")
                .with_size((1280, 720))
                .with_scene(DEMO_SCENE, Demo)
                .with_active_scene(DEMO_SCENE),
        )
        .build()
        .run();
}
