use karna::{App, Scene, WindowBuilder, input::KeyCode};
use renderer::Color;

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

struct StatesDemo2;

impl Scene for StatesDemo2 {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
    }

    fn update(&mut self, ctx: &mut karna::Context) {}

    fn render(&mut self, ctx: &mut karna::Context) {
        let state = ctx.states.get::<SharedState>().unwrap();

        ctx.render.draw_debug_text(
            "Press space on the cyan window to toggle the state",
            [10.0, 10.0],
        );

        ctx.render.draw_debug_text(
            if state.value {
                "state is true"
            } else {
                "state is false"
            },
            [10.0, 30.0],
        );
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
                .with_initial_scene(StatesDemo2),
        )
        .build()
        .run();
}
