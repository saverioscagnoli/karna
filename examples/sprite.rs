use std::time::Duration;

use karna::App;
use karna::Scene;
use karna::WindowBuilder;
use renderer::sprite::AnimatedSprite;
use renderer::sprite::Sprite;
use renderer::sprite::animation::Animation;
use renderer::sprite::animation::Animations;
use renderer::sprite::animation::Frame;

pub struct S {
    sprite: AnimatedSprite,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        let bomberman = ctx.assets.load_png(include_bytes!("images/bomberman.png"));

        self.sprite = AnimatedSprite::new(
            bomberman,
            Animations::default().add_animation(
                "walk-left",
                Animation::default()
                    .add_frame(Frame::new(0, 0, 16, 24, Duration::from_millis(100)))
                    .add_frame(Frame::new(16, 0, 16, 24, Duration::from_millis(100)))
                    .add_frame(Frame::new(32, 0, 16, 24, Duration::from_millis(100))),
            ),
        );

        self.sprite.animator.play("walk-left", true);
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {
        self.sprite
            .update(Duration::from_secs_f32(ctx.time.delta()));
    }

    fn draw(&self, ctx: karna::ContextRef, draw: &mut renderer::Draw) {
        self.sprite.draw(draw, 100.0, 100.0);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::default()
                .with_title("Sprite demo")
                .with_size((1280, 720))
                .with_initial_scene(S {
                    sprite: AnimatedSprite::default(),
                }),
        )
        .build()
        .run()
}
