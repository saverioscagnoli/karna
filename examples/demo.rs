use karna::App;
use karna::ContextRef;
use karna::ContextRefMut;
use karna::Scene;
use karna::WindowBuilder;
use karna::logging;
use karna::render::Color;
use karna::render::Draw;
use log::info;

struct S {
    x: f32,
    y: f32,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextRefMut) {
        info!("Scene loaded")
    }

    fn update(&mut self, ctx: ContextRefMut) {
        info!("dt {}", ctx.time.delta());

        self.x += 1.0;
        self.y += 1.0
    }

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {
        draw.set_color(Color::Cyan);
        draw.rect(self.x, self.y, 50.0, 50.0);
    }
}

fn main() {
    logging::init(logging::Config::default()).expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_scene("initial", S { x: 0.0, y: 0.0 })
                .with_active_scene("initial"),
        )
        .build()
        .run();
}
