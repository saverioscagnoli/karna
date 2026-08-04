#![allow(unused)]

use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Font;
use karna::assets::Handle;
use karna::render::Color;
use karna::render::Draw;
use karna::render::SceneRef;

struct TextNewlineDemo {
    jbmono: Handle<Font>,
}

impl Scene for TextNewlineDemo {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        let jbmono = ctx.assets.load_font("assets/jbmono.ttf", 21.5);

        Self { jbmono }
    }

    fn update(&mut self, _c: ContextRef, _s: &mut SceneRef) {}

    fn draw(&mut self, _c: DrawContext, draw: &mut Draw) {
        draw.debug_text("first line\nsecond line\n\nblank line above", 20.0, 20.0);
        draw.debug_text("crlf one\r\ncrlf two", 20.0, 190.0);
        draw.debug_text("tab:\tafter\nab\tcd", 20.0, 300.0);

        draw.set_color(Color::Magenta);
        draw.text(
            self.jbmono,
            "first line\nsecond line\n\nblank line above",
            20.0,
            370.0,
        );
        draw.text(self.jbmono, "crlf one\r\ncrlf two", 20.0, 500.0);
        draw.text(self.jbmono, "tab:\tafter\nab\tcd", 20.0, 600.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("nlcheck")
                .with_size((800, 800))
                .with_scene::<TextNewlineDemo>(0)
                .with_active_scene(0),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
