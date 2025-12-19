#![allow(unused)]
use karna::{
    App, Scene, WindowBuilder, label,
    render::{self, Color, Text},
};
use math::Vector2;
use std::time::{Duration, Instant};

struct TextDemo {
    text: Text,
    other_text: Text,
    other_other_text: Text,
    last_update: Instant,
}

impl Scene for TextDemo {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.time.set_target_fps(120);
        ctx.assets.load_font(
            label!("jetbrains mono"),
            include_bytes!("assets/JetBrainsMono-Regular.ttf").to_vec(),
            18,
        );

        self.text.set_position(Vector2::new(10.0, 10.0));
        self.other_text.set_position(Vector2::new(200.0, 200.0));
        self.other_text
            .set_content("This is jetbrains mono\nàèìòùáéíóú?! ê");

        self.other_other_text.set_content("WOOOOOOOO");
        self.other_other_text.set_position([350.0, 150.0]);
        self.other_other_text.set_color(Color::Magenta);

        ctx.render.set_clear_color(Color::Black);

        // First update so that it's not empty
        self.text.set_content(format!(
            "fps: {}\ndt: {}\nframe time: {:?}\ntick time: {:?}",
            ctx.time.fps(),
            ctx.time.delta(),
            ctx.time.frame(),
            ctx.time.tick()
        ));
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let t = ctx.time.elapsed().as_secs_f32();

        self.text.set_color_r((t * 2.0).sin() * 0.5 + 0.5);
        self.text.set_color_g(((t * 2.0) + 2.0).sin() * 0.5 + 0.5);
        self.text.set_color_b(((t * 2.0) + 4.0).sin() * 0.5 + 0.5);

        if self.last_update.elapsed() >= Duration::from_millis(50) {
            self.text.set_content(format!(
                "fps: {}\ndt: {}\nframe time: {:?}\ntick time: {:?}",
                ctx.time.fps(),
                ctx.time.delta(),
                ctx.time.frame(),
                ctx.time.tick()
            ));

            self.last_update = Instant::now();
        }

        let pendulum_scale = (t * 2.0).sin() * 2.0 + 2.5;
        self.other_text.set_scale(Vector2::splat(pendulum_scale));
        self.other_other_text.set_rotation(-t * 2.0);
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_text(&mut self.text);
        ctx.render.draw_text(&mut self.other_text);
        ctx.render.draw_text(&mut self.other_other_text);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("text demo")
                .with_resizable(false)
                .with_initial_scene(TextDemo {
                    text: Text::new(label!("debug")),
                    other_text: Text::new(label!("jetbrains mono")),
                    other_other_text: Text::new(label!("jetbrains mono")),
                    last_update: Instant::now(),
                }),
        )
        .build()
        .run();
}
