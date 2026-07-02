use std::u32;

use karna::App;
use karna::ContextRef;
use karna::ContextRefMut;
use karna::Scene;
use karna::WindowBuilder;
use karna::input::Keycode;
use sokol::gfx as sg;
use sokol::glue as sglue;

struct ClearApp {
    pass_action: sg::PassAction,
    elapsed: f32,
}

impl Scene for ClearApp {
    fn load(&mut self, ctx: ContextRefMut) {
        ctx.time.set_target_fps(200);
        self.pass_action.colors[0] = sg::ColorAttachmentAction {
            load_action: sg::LoadAction::Clear,
            clear_value: sg::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            ..Default::default()
        };
        let backend = sg::query_backend();

        println!(
            "Using backend: {:?}, window: {}x{}",
            backend,
            ctx.window.width(),
            ctx.window.height()
        );
    }

    fn fixed_update(&mut self, ctx: ContextRefMut) {
        println!("fps {}", ctx.time.fps())
    }

    fn update(&mut self, ctx: ContextRefMut) {
        self.elapsed += ctx.time.delta();

        let g = self.pass_action.colors[0].clear_value.g + 0.01;
        self.pass_action.colors[0].clear_value.g = if g > 1.0 { 0.0 } else { g };

        // Example of mutating the window at runtime.
        if self.elapsed > 2.0 {
            self.elapsed = 0.0;
            ctx.window.set_title("still clearing...");
        }
    }

    fn draw(&mut self, _ctx: ContextRef) {
        sg::begin_pass(&sg::Pass {
            action: self.pass_action,
            swapchain: sglue::swapchain(),
            ..Default::default()
        });
        sg::end_pass();
        sg::commit();
    }
}

pub fn main() {
    App::builder()
        .with_window(WindowBuilder::new().with_title("clear.rs"))
        .with_scene(
            "initial".to_string(),
            ClearApp {
                pass_action: sg::PassAction::new(),
                elapsed: 0.0,
            },
        )
        .build()
        .run();
}
