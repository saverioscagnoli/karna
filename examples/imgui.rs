use karna::{
    input::KeyCode,
    math::rng,
    render::{imgui::Condition, Rect},
    App, Context, Scene,
};

pub struct ImguiDemo {
    checked: bool,
    slider_value: i32,
    values: Vec<f32>,
    open: bool,
    clear_color: [f32; 4],
    rect: Rect,
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
        let mut new_clear_color = None;
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

                        if ui.color_picker4("Clear color", &mut self.clear_color) {
                            new_clear_color.replace(self.clear_color);
                        }
                    });
            }

            ui.show_demo_window(&mut true);
        });

        if let Some(color) = new_clear_color {
            ctx.render.set_clear_color(color.into());
        }

        self.rect.render(&mut ctx.render);
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
                clear_color: [0.1, 0.1, 0.1, 1.0],
                
                rect: Rect::default().with_position([10, 10]).with_size(50.0),
            },
        )
        .run()
        .expect("Failed to run app");
}
