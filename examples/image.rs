#![allow(unused)]

use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Handle;
use karna::assets::Image;
use karna::logging::Config;
use karna::logging::LevelFilter;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use karna::render::SceneRef;
use logging::info;

struct ImageDemo {
    pcb: Handle<Image>,
}

impl Scene for ImageDemo {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        ctx.window.set_resizable(true);
        let pcb = ctx.assets.load_image("assets/pcb.png");

        Self { pcb }
    }

    fn fixed_update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.image(self.pcb, 200.0, 200.0);

        draw.set_color(Color::rgba(0.0, 0.0, 1.0, 0.9));
        draw.image(self.pcb, 700.0, 200.0);

        draw.set_color(Color::rgba(1.0, 0.0, 0.0, 0.7));
        draw.image(self.pcb, 700.0, 100.0);

        draw.set_color(Color::White);
        draw.debug_text("Invalid image", 300.0, 500.0);
        draw.image(Handle::INVALID, 300.0, 530.0);
    }
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Image Demo")
                .with_size((1280, 720))
                .with_scene::<ImageDemo>(0)
                .with_active_scene(0),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
