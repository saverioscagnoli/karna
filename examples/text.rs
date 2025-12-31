use karna::{App, Scene, WindowBuilder};
use renderer::{Color, Layer, Text, TextHandle};
use utils::{Handle, Label, label};

struct TextDemo {
    text1: TextHandle,
    text2: TextHandle,
}

impl Scene for TextDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
        ctx.assets.load_font(
            label!("jetbrains mono"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf").to_vec(),
            16,
        );

        let mut text = Text::new(label!("jetbrains mono"), "Hello, world!");

        text.set_position([100.0, 100.0, 0.0]);
        text.set_color(Color::Red);

        self.text1 = ctx.render.add_text(text);

        let mut text = Text::new(label!("jetbrains mono"), "This is jetbrains mono");

        text.set_position([100.0, 150.0, 0.0]);
        text.set_color(Color::Green);
        self.text2 = ctx.render.add_text(text);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let text1 = ctx.render.get_text_mut(self.text1);

        *text1.rotation_mut() += 0.01;
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render
            .debug_text("This, instead, is immediate mode text!!", 10.0, 300.0);

        ctx.render.set_font(label!("jetbrains mono"));
        ctx.render
            .draw_text("I'm drawing this in immediate mode :D", 300.0, 10.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Text demo")
                .with_resizable(false)
                .with_initial_scene(TextDemo {
                    text1: TextHandle::dummy(),
                    text2: TextHandle::dummy(),
                }),
        )
        .build()
        .run();
}
