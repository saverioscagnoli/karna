use assets::Font;
use karna::App;
use karna::Scene;
use karna::WindowBuilder;
use karna::math::Size;
use karna::render::Color;
use karna::render::Draw;
use utils::Handle;

struct S {
    jbmono: Handle<Font>,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        self.jbmono = ctx.assets.load_font(include_bytes!("fonts/jbmono.ttf"), 16);
    }

    fn update(&mut self, _ctx: karna::ContextRefMut) {}

    fn draw(&self, _ctx: karna::ContextRef, draw: &mut Draw) {
        draw.debug_text("Hello world!!\nThis is my text", 50.0, 50.0);

        draw.set_color(Color::Cyan);
        draw.text(self.jbmono, "I love this font!", 300.0, 100.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size(Size::new(1280, 720))
                .with_initial_scene(S {
                    jbmono: Handle::default(),
                }),
        )
        .build()
        .run();
}
