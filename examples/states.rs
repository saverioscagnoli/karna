#![allow(unused)]

use karna::{App, Scene, WindowBuilder, input::KeyCode};
use renderer::Color;
use utils::label;

struct StatesDemo1;

impl Scene for StatesDemo1 {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
        ctx.states.insert(label!("boolean"), false);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        if ctx.input.key_pressed(&KeyCode::Space) {
            ctx.scenes.request_change(label!("scene_2"));
        }

        if ctx.input.key_pressed(&KeyCode::KeyB) {
            let mut state = ctx.states.get_mut::<bool>(label!("boolean")).unwrap();

            *state = !*state;
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        let state = ctx.states.get::<bool>(label!("boolean")).unwrap();
        let str = format!("This is Scene 1. Boolean value: {}", state);

        ctx.render.ui.draw_debug_text(str, [10.0, 10.0]);
        ctx.render
            .ui
            .draw_debug_text("Press 'B' to toggle the state!", [10.0, 30.0]);

        ctx.render
            .ui
            .draw_debug_text("Press 'Space' to switch to Scene 2.", [10.0, 50.0]);
    }

    fn on_changed(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Black);
    }
}

struct StatesDemo2;

impl Scene for StatesDemo2 {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Brown);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        if ctx.input.key_pressed(&KeyCode::Space) {
            ctx.scenes.request_change(label!("initial"));
        }
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        let state = ctx.states.get::<bool>(label!("boolean")).unwrap();
        let str = format!("This is Scene 2. Boolean value: {}", state);

        ctx.render.ui.draw_debug_text(str, [10.0, 10.0]);

        ctx.render
            .ui
            .draw_debug_text("Press 'Space' to switch to Scene 1.", [10.0, 30.0]);
    }

    fn on_changed(&mut self, ctx: &mut karna::Context) {
        ctx.render.set_clear_color(Color::Brown);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("states demo")
                .with_resizable(false)
                .with_initial_scene(StatesDemo1)
                .with_scene(label!("scene_2"), StatesDemo2),
        )
        .build()
        .run()
}
