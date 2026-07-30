#![allow(unused)]

use std::collections::HashMap;

use engine::Key::H;
use engine::Rasterize;
use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Font;
use karna::assets::Handle;
use karna::assets::Image;
use karna::logging::Config;
use karna::logging::LevelFilter;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use karna::render::SceneRef;
use logging::info;

struct S {
    pcb: Handle<Image>,
    jbmono: Handle<Font>,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        self.pcb = ctx.assets.load_image("assets/pcb.png");
        self.jbmono = ctx.assets.load_font("assets/jbmono.ttf", 21.0);

        ctx.window.set_resizable(true);
    }

    fn fixed_update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.image(self.pcb, 200.0, 200.0);

        draw.set_color(Color::rgba(0.0, 0.0, 1.0, 0.9));
        draw.image(self.pcb, 700.0, 200.0);

        draw.set_color(Color::rgba(1.0, 0.0, 0.0, 0.7));
        draw.image(self.pcb, 700.0, 100.0);

        draw.set_color(Color::Cyan);
        draw.text(
            self.jbmono,
            format!("Hello world!\ndelta time: {}", ctx.time.delta()),
            10.0,
            10.0,
        );
    }
}

struct AtlasDemo;

impl Scene for AtlasDemo {
    fn load(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        ctx.window.set_resizable(true);
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.atlas_page(0, 0.0, 0.0);
    }
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Texture Atlas Demo")
                .with_size((1280, 720))
                .with_scene(
                    0,
                    S {
                        pcb: Handle::INVALID,
                        jbmono: Handle::INVALID,
                    },
                    true,
                ),
        )
        .with_window(
            WindowBuilder::new()
                .with_title("Texture Atlas")
                .with_size((1024, 1024))
                .with_scene(1, AtlasDemo, true),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
