#![allow(unused)]

use karna::{
    App, Scene, WindowBuilder, label,
    render::{Color, Text},
};

struct SimpleText {
    hello_text: Option<Text>,
}

impl SimpleText {
    fn new() -> Self {
        Self { hello_text: None }
    }
}

impl Scene for SimpleText {
    fn load(&mut self, ctx: &mut karna::Context) {
        // Set background color
        ctx.render.set_clear_color(Color::rgb(0.1, 0.1, 0.15));

        // Step 1: Load the font first!
        ctx.render.load_font(
            label!("jetbrains"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf").to_vec(),
            48, // font size in pixels
        );

        // Step 2: Create text after font is loaded
        let mut text = Text::new(label!("jetbrains"), "Hello, Karna!");
        text.set_color(Color::White);
        text.set_position([100.0, 100.0]);

        self.hello_text = Some(text);
    }

    fn update(&mut self, _ctx: &mut karna::Context) {
        // Nothing to update
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        // Render the text
        if let Some(text) = &self.hello_text {
            text.draw(&mut ctx.render);
        }
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("simple_text")
                .with_title("Simple Text Example")
                .with_size((800, 600))
                .with_resizable(false)
                .with_initial_scene(SimpleText::new()),
        )
        .build()
        .run();
}
