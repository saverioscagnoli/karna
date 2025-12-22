#![allow(unused)]

use karna::{App, Scene, WindowBuilder, input::KeyCode};
use renderer::{Color, Text};
use utils::label;

struct SharedState {
    value: bool,
}

struct StatesDemo;

impl Scene for StatesDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.states.insert(SharedState { value: false });
        ctx.render.set_clear_color(Color::Cyan);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        if ctx.input.key_pressed(&KeyCode::Space)
            && let Some(ref mut state) = ctx.states.get_mut::<SharedState>()
        {
            state.value = !state.value;
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {}
}

struct StatesDemo2 {
    text: Text,
    color: Color,
}

impl Scene for StatesDemo2 {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let t = ctx.time.elapsed().as_secs_f32();

        self.color.set_red((t * 2.0).sin() * 0.5 + 0.5);
        self.color.set_green(((t * 2.0) + 2.0).sin() * 0.5 + 0.5);
        self.color.set_blue(((t * 2.0) + 4.0).sin() * 0.5 + 0.5);
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        let state = ctx.states.get::<SharedState>().unwrap();

        ctx.render.draw_debug_text(
            "Press space on the cyan window to toggle the state",
            [10.0, 10.0],
        );

        if state.value {
            self.text.set_color(self.color);
            self.text.set_content("state is true");
        } else if self.text.content() != "state is false" {
            self.text.set_color(Color::White);
            self.text.set_content("state is false");
        };

        ctx.render.draw_text(&mut self.text);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("states demo")
                .with_resizable(false)
                .with_initial_scene(StatesDemo),
        )
        .with_window(
            WindowBuilder::new()
                .with_label("secondart")
                .with_title("states demo secondary window")
                .with_resizable(false)
                .with_initial_scene(StatesDemo2 {
                    text: Text::new(label!("debug")).with_position([10.0, 30.0]),
                    color: Color::White,
                }),
        )
        .build()
        .run();
}
