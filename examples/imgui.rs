use karna::{input::KeyCode, render::imgui::Condition, App, Context, Scene};
use math::rng;

pub struct ImguiDemo {
    checked: bool,
    slider_value: i32,
    values: Vec<f32>,
    open: bool,
}

impl Scene for ImguiDemo {
    fn load(&mut self, _ctx: &mut Context) {}

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.key_pressed(KeyCode::Space) {
            self.open = !self.open;
        }
    }

    fn render(&mut self, ctx: &mut Context) {
        ctx.render.imgui.render_frame(|ui| {
            if self.open {
                ui.window("Frame Info")
                    .position([10.0, 10.0], Condition::FirstUseEver)
                    .size([300.0, 200.0], Condition::FirstUseEver)
                    .opened(&mut self.open)
                    .build(|| {
                        ui.text(format!("fps: {}", ctx.time.fps()));
                        ui.text(format!("delta time: {}", ctx.time.delta()));

                        if ui.button("Click me!") {
                            println!("Button clicked!");
                        }

                        ui.separator();
                        ui.checkbox("cool", &mut self.checked);
                        ui.slider("slider", 0, 100, &mut self.slider_value);
                        ui.plot_histogram("Frame Times", &self.values).build();
                    });
            }

            ui.show_demo_window(&mut true);
        });
    }
}

fn main() {
    App::new()
        .with_size((1280, 720))
        .with_scene(
            "default",
            ImguiDemo {
                checked: false,
                slider_value: 0,
                values: (0..20).map(|_| rng(0.0..16.0)).collect(),
                open: true,
            },
        )
        .run()
        .expect("Failed to run app");
}
