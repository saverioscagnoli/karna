#![allow(unused)]

use karna::prelude::*;

const LINE_SCENE: SceneId = SceneId::new_str("line");

struct LineDemo;

impl Scene for LineDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);
        Self
    }

    fn update(&mut self, ctx: &mut UpdateContext) {}

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        let m = ctx.window.mouse_position();
        let size = ctx.window.size().cast::<f32>();

        let text = format!("mouse: ({}, {})\ndt: {}", m.x, m.y, ctx.time.delta());

        draw.print(text, 10.0, 10.0);

        draw.set_color(Color::CYAN);
        draw.line(0.0, m.y, size.w(), m.y);
        draw.line(m.x, 0.0, m.x, size.h());

        draw.set_color(Color::MAGENTA);
        draw.circle_v(m, 5.0);
    }
}

fn main() {
    _ = traccia::init(
        traccia::Config::default()
            .with_min_level(traccia::LevelFilter::Debug)
            .with_module_filter("cosmic_text", traccia::LevelFilter::Warn)
            .with_target(SdlTarget::default()),
    );

    App::builder()
        .with_root("karna/examples/")
        .with_window(
            WindowBuilder::default()
                .with_title("lines demo")
                .with_size((1280, 720))
                .with_scene::<LineDemo>(LINE_SCENE)
                .with_active_scene(LINE_SCENE),
        )
        .build()
        .run();
}
