use karna::App;
use karna::ContextRef;
use karna::ContextRefMut;
use karna::Scene;
use karna::WindowBuilder;
use karna::logging;
use log::info;

struct S;

impl Scene for S {
    fn load(&mut self, ctx: ContextRefMut) {
        info!("Scene loaded")
    }

    fn update(&mut self, ctx: ContextRefMut) {
        info!("dt {}", ctx.time.delta());
    }

    fn draw(&self, ctx: ContextRef) {}
}

fn main() {
    logging::init(logging::Config::default()).expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_scene("initial", S)
                .with_active_scene("initial"),
        )
        .build()
        .run();
}
