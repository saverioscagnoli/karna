use karna::{AppBuilder, Scene, WindowBuilder, label};
use renderer::{Frame, Layer, Sprite, SpriteHandle};
use std::time::Duration;

struct S {
    sprite: SpriteHandle,
}

impl Scene for S {
    fn load(&mut self, ctx: &mut karna::Context) {
        ctx.assets.load_image(
            label!("witch"),
            include_bytes!("assets/witch-idle.png").to_vec(),
        );

        let mut sprite = Sprite::new(
            label!("witch"),
            vec![
                Frame {
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 48,
                    duration: Duration::from_millis(150),
                },
                Frame {
                    x: 0,
                    y: 48,
                    width: 32,
                    height: 48,
                    duration: Duration::from_millis(150),
                },
                Frame {
                    x: 0,
                    y: 96,
                    width: 32,
                    height: 48,
                    duration: Duration::from_millis(150),
                },
                Frame {
                    x: 0,
                    y: 144,
                    width: 32,
                    height: 48,
                    duration: Duration::from_millis(150),
                },
                Frame {
                    x: 0,
                    y: 192,
                    width: 32,
                    height: 48,
                    duration: Duration::from_millis(150),
                },
                Frame {
                    x: 0,
                    y: 240,
                    width: 32,
                    height: 48,
                    duration: Duration::from_millis(150),
                },
            ],
        );

        sprite.set_render_scale([2.0, 2.0]);

        self.sprite = ctx.render.add_sprite(Layer::World, sprite);
    }

    fn update(&mut self, ctx: &mut karna::Context) {
        let sprite = ctx.render.get_sprite_mut(self.sprite);

        sprite.update(ctx.time.delta());
    }

    fn render(&mut self, ctx: &mut karna::Context) {}
}

fn main() {
    AppBuilder::new()
        .with_window(
            WindowBuilder::new()
                .with_label("main")
                .with_title("Sprite demo")
                .with_resizable(false)
                .with_initial_scene(S {
                    sprite: SpriteHandle::dummy(),
                }),
        )
        .build()
        .run();
}
