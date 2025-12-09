use std::time::Duration;

use karna::{AppBuilder, Scene, WindowBuilder, label};
use renderer::{Frame, Sprite};

struct S {
    sprite: Sprite,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut karna::Context) {
        self.sprite.set_scale([64.0, 96.0]);
        ctx.render
            .load_texture(label!("witch"), include_bytes!("assets/witch-idle.png"));
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
        .with_window(WindowBuilder::new().with_initial_scene(Box::new(S {
            sprite: Sprite::new(
                label!("witch"),
                vec![
                    Frame {
                        x: 0,
                        y: 0,
                        width: 32,
                        height: 48,
                    },
                    Frame {
                        x: 0,
                        y: 48,
                        width: 32,
                        height: 48,
                    },
                    Frame {
                        x: 0,
                        y: 96,
                        width: 32,
                        height: 48,
                    },
                    Frame {
                        x: 0,
                        y: 144,
                        width: 32,
                        height: 48,
                    },
                    Frame {
                        x: 0,
                        y: 192,
                        width: 32,
                        height: 48,
                    },
                    Frame {
                        x: 0,
                        y: 240,
                        width: 32,
                        height: 48,
                    },
                ],
                Duration::from_millis(150),
            ),
        })))
        .build()
        .run();
}
