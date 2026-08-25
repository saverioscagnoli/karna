#![allow(unused)]

use std::thread;
use std::time::Duration;

use karna::prelude::*;
use utils::Handle;

struct ImageDemo {
    pcb: Handle<Image>,
}

impl Scene for ImageDemo {
    fn load(ctx: LoadContext, scene: &mut SceneView) -> Self
    where
        Self: Sized,
    {
        let pcb = ctx.assets.load_image("assets/pcb.png");
        let cobb = ctx.assets.load_image("assets/cobblestone.png");

        ctx.window.set_custom_cursor(cobb, [0, 0]);

        Self { pcb }
    }

    fn update(&mut self, ctx: UpdateContext, scene: &mut SceneView) {}

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.image(self.pcb, 100.0, 100.0);

        draw.set_color(Color::RED);
        draw.image(self.pcb, 500.0, 200.0);
    }
}

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("image demo")
                .with_scene::<ImageDemo>(SceneId::new_label("image-demo"))
                .with_active_scene(SceneId::new_label("image-demo")),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
