use karna::{AppBuilder, Context, Draw, RenderContext, Scene, WindowBuilder, render::Color};
use math::rng;

#[derive(Default)]
struct ImmediateDemo {
    lines: Vec<(f32, f32, f32, f32)>,
}

impl Scene for ImmediateDemo {
    fn load(&mut self, _ctx: &mut Context) {
        let lines: Vec<(f32, f32, f32, f32)> = (0..10)
            .map(|_| {
                (
                    rng(0.0..800.0),
                    rng(0.0..600.0),
                    rng(0.0..800.0),
                    rng(0.0..600.0),
                )
            })
            .collect();

        self.lines = lines;
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, _ctx: &RenderContext, draw: &mut Draw) {
        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.point(40.0 + i as f32 * 10.0, 100.0 + j as f32 * 10.0);
            }
        }

        draw.set_color(Color::Orange);
        draw.line(500.0, 100.0, 100.0, 400.0);

        draw.set_color(Color::Pink);
        draw.lines(&self.lines);

        draw.set_color(Color::Red);
        draw.rect(100.0, 300.0, 50.0, 50.0);

        draw.set_color(Color::Magenta);
        draw.circle(300.0, 100.0, 50.0);

        draw.set_color(Color::White);
        draw.debug_text("Text!!", 10.0, 10.0);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Immediate mode demo")
                .with_resizable(false)
                .with_initial_scene(ImmediateDemo::default()),
        )
        .build()
        .run();
}
