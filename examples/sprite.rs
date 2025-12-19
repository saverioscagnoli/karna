use std::time::Duration;

use karna::{AppBuilder, Scene, WindowBuilder, label};
use renderer::{Frame, Sprite};

struct S {
    sprite: Sprite,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.assets.load_image(
            label!("witch"),
            include_bytes!("assets/witch-idle.png").to_vec(),
        );
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        self.sprite.update(ctx.time.delta());
    }

    fn render(&mut self, ctx: &mut karna::Context) {
        ctx.render.draw_mesh(&self.sprite);
    }
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Sprite demo")
                .with_resizable(false)
                .with_initial_scene(S {
                    sprite: Sprite::new(
                        label!("witch"),
                        vec![
                            Frame {
                                x: 0.0,
                                y: 0.0,
                                width: 32.0,
                                height: 48.0,
                                duration: Duration::from_millis(150),
                            },
                            Frame {
                                x: 0.0,
                                y: 48.0,
                                width: 32.0,
                                height: 48.0,
                                duration: Duration::from_millis(150),
                            },
                            Frame {
                                x: 0.0,
                                y: 96.0,
                                width: 32.0,
                                height: 48.0,
                                duration: Duration::from_millis(150),
                            },
                            Frame {
                                x: 0.0,
                                y: 144.0,
                                width: 32.0,
                                height: 48.0,
                                duration: Duration::from_millis(150),
                            },
                            Frame {
                                x: 0.0,
                                y: 192.0,
                                width: 32.0,
                                height: 48.0,
                                duration: Duration::from_millis(150),
                            },
                            Frame {
                                x: 0.0,
                                y: 240.0,
                                width: 32.0,
                                height: 48.0,
                                duration: Duration::from_millis(150),
                            },
                        ],
                    )
                    .with_render_scale([2.0, 2.0]),
                }),
        )
        .build()
        .run();
}
