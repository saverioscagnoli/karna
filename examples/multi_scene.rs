#![allow(unused)]

use std::any::Any;

use karna::prelude::*;

struct S;

impl Scene for S {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {}

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        if ctx.input.key_pressed(Keycode::Digit2) {
            // When transitioning between exactly 2 scenes,
            // you have to call deactivate first, otherwise they will stack and
            // may not output the result you want.
            ctx.scenes.deactivate("scene-1");
            ctx.scenes.activate_with(
                "scene-2",
                "Hello! This is some data from scene 1!".to_string(),
            );
        }
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        draw.debug_text("This is scene 1", 10.0, 10.0);
        draw.debug_text("Press '2' to go into scene 2!", 10.0, 30.0);
    }
}

#[derive(Default)]
struct A {
    user_data: String,
}

impl Scene for A {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {}

    fn loaded_with(&mut self, ctx: ContextMut, scene: &mut SceneHandle, user_data: Box<dyn Any>) {
        self.user_data = user_data.downcast_ref::<String>().unwrap().clone();
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        if ctx.input.key_pressed(Keycode::Digit1) {
            ctx.scenes.deactivate("scene-2");
            ctx.scenes.activate("scene-1");
        }

        if ctx.input.key_pressed(Keycode::Digit3) {
            ctx.scenes.deactivate("scene-2");
            ctx.scenes.register("scene-3", Box::new(C));
            ctx.scenes.activate("scene-3");
        }
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        draw.debug_text("This is scene 2", 10.0, 10.0);
        draw.debug_text(
            &format!("user data was passed: {}", self.user_data),
            10.0,
            30.0,
        );
        draw.debug_text("Press '1' to go back to scene 1!", 10.0, 50.0);
        draw.debug_text(
            "Press '3' to go to scene '3'! (Registered programmatically)",
            10.0,
            70.0,
        );
    }
}

struct C;

impl Scene for C {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {}

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        if ctx.input.key_pressed(Keycode::Digit1) {
            ctx.scenes.deactivate("scene-3");
            ctx.scenes.activate("scene-1");
        }

        if ctx.input.key_pressed(Keycode::Digit2) {
            ctx.scenes.deactivate("scene-3");
            ctx.scenes.activate("scene-2");
        }
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        draw.debug_text("This is scene 3! (Registered programmatically)", 10.0, 10.0);
        draw.debug_text("Press '1' to go back to scene 1!", 10.0, 30.0);
        draw.debug_text("Press '2' to go back to scene 2!", 10.0, 50.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_title("multi scene demo")
                .with_scene("scene-1", S)
                .with_scene("scene-2", A::default())
                .with_active_scene("scene-1"),
        )
        .build()
        .run();
}
