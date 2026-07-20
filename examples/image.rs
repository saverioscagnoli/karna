#![allow(unused)]

use karna::App;
use karna::Context;
use karna::Scene;
use karna::SceneView;
use karna::WindowBuilder;
use karna::assets::Handle;
use karna::assets::Image;
use karna::logging::Config;
use karna::render::Draw;

const IMAGE_SCENE: usize = 0;

struct ImageDemo {
    pcb: Handle<Image>,
}

impl Scene for ImageDemo {
    fn load(&mut self, mut ctx: &mut Context, scene: &mut SceneView) {
        ctx.assets.write_scope(|w| {
            self.pcb = w.load_image(include_bytes!("assets/pcb.png"));
        })
    }

    fn fixed_update(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn update(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {}
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Demo window")
                .with_size((1280, 720))
                .with_scene(
                    IMAGE_SCENE,
                    ImageDemo {
                        pcb: Handle::INVALID,
                    },
                )
                .with_active_scene(IMAGE_SCENE),
        )
        .build()
        .run();
}
