use karna::{App, Scene, WindowBuilder, render::Text};
use renderer::{Color, Layer};
use utils::{Handle, label};

struct TextDemo {
    text1: Handle<Text>,
}

impl Scene for TextDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);

        let mut text = Text::new(label!("debug"), "Hello, world!");

        text.set_position([100.0, 100.0, 0.0]);

        self.text1 = ctx.render.add_text(Layer::World, text);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let text1 = ctx.render.get_text_mut(self.text1);

        *text1.rotation_mut() += 0.01;
    }

    fn render(&mut self, ctx: &mut karna::Context) {}
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Text demo")
                .with_resizable(false)
                .with_initial_scene(TextDemo {
                    text1: Handle::dummy(),
                }),
        )
        .build()
        .run();
}
