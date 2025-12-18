#![allow(unused)]
use karna::{App, Scene, WindowBuilder, label, render::Text};

struct TextDemo {
    text: Text,
}

impl Scene for TextDemo {
    fn load(&mut self, ctx: &mut karna::Context) {}

    fn update(&mut self, ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_text(&self.text);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("text demo")
                .with_resizable(false)
                .with_initial_scene(TextDemo {
                    text: Text::new(label!("debug")).with_content("Hello world!"),
                }),
        )
        .build()
        .run();
}
