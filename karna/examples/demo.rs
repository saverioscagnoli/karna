#![allow(unused)]

use karna::prelude::*;

const DEMO_SCENE: SceneId = SceneId::new_str("DEMO");

struct DemoScene {
    pcb: Handle<Image>,
}

impl Scene for DemoScene {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        Self {
            pcb: ctx.assets.load_image("assets/pcb2.png"),
        }
    }

    fn fixed_update(&mut self, ctx: &mut UpdateContext) {}

    fn update(&mut self, ctx: &mut UpdateContext) {}

    fn draw(&mut self, ctx: &mut DrawContext, draw: &mut Draw) {
        draw.set_color(Color::RED).rect(10.0, 10.0, 100.0, 50.0);
        draw.set_color(Color::CYAN).circle(300.0, 200.0, 40.0);
        draw.set_color(Color::YELLOW).line(20.0, 300.0, 400.0, 350.0, 3.0);
        draw.set_color(Color::WHITE).image(self.pcb, 500.0, 100.0);

        draw.on_layer(Layer::UI)
            .set_color(Color::WHITE)
            .rect_lines(0.0, 0.0, 1280.0, 32.0, 2.0);
    }
}

fn main() {
    _ = traccia::init(
        traccia::Config::default()
            .with_min_level(traccia::LevelFilter::Debug)
            .with_target(SdlTarget::default()),
    );

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Demo window")
                .with_size((1280, 720))
                .with_scene::<DemoScene>(DEMO_SCENE)
                .with_active_scene(DEMO_SCENE),
        )
        .with_root("karna/examples/")
        .build()
        .run();
}
