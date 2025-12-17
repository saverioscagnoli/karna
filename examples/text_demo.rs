#![allow(unused)]

use karna::{
    App, Scene, WindowBuilder,
    input::KeyCode,
    label,
    render::{Color, Text},
};

struct TextDemo {
    title: Option<Text>,
    instructions: Option<Text>,
    counter: Option<Text>,
    count: i32,
}

impl TextDemo {
    fn new() -> Self {
        Self {
            title: None,
            instructions: None,
            counter: None,
            count: 0,
        }
    }
}

impl Scene for TextDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::rgb(0.1, 0.15, 0.2));

        // Load the font FIRST
        ctx.render.load_font(
            label!("jetbrains"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf").to_vec(),
            48,
        );

        // NOW create title text
        let mut title = Text::new(label!("jetbrains"), "Karna Text Rendering");
        title.set_color(Color::White);
        title.set_position([50.0, 80.0]);
        self.title = Some(title);

        // Create instructions
        let mut instructions = Text::new(label!("jetbrains"), "Press SPACE to increment counter");
        instructions.set_color(Color::rgb(0.7, 0.7, 0.7));
        instructions.set_position([50.0, 150.0]);
        self.instructions = Some(instructions);

        // Create counter
        let mut counter = Text::new(label!("jetbrains"), "Count: 0");
        counter.set_color(Color::rgb(0.3, 0.8, 0.3));
        counter.set_position([50.0, 220.0]);
        self.counter = Some(counter);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        if ctx.input.key_pressed(&KeyCode::Space) {
            self.count += 1;

            if let Some(counter) = &mut self.counter {
                counter.set_content(format!("Count: {}", self.count));
                // Change to random color on increment
                counter.set_color(Color::random());
            }
        }

        if ctx.input.key_pressed(&KeyCode::KeyR) {
            self.count = 0;

            if let Some(counter) = &mut self.counter {
                counter.set_content("Count: 0");
                counter.set_color(Color::rgb(0.3, 0.8, 0.3));
            }
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        if let Some(title) = &self.title {
            title.draw(&mut ctx.render);
        }

        if let Some(instructions) = &self.instructions {
            instructions.draw(&mut ctx.render);
        }

        if let Some(counter) = &self.counter {
            counter.draw(&mut ctx.render);
        }
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("text_demo")
                .with_title("Karna Text Demo")
                .with_size((800, 600))
                .with_resizable(false)
                .with_initial_scene(TextDemo::new()),
        )
        .build()
        .run();
}
