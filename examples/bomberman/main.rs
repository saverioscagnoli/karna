mod bomberman;
mod consts;
mod level;

use std::time::Duration;

use karna::App;
use karna::KeyCode;
use karna::Scene;
use karna::WindowBuilder;
use logging::info;
use math::Vector2;
use renderer::Color;
use renderer::sprite::AnimatedSprite;
use renderer::sprite::animation::Animation;
use renderer::sprite::animation::Animations;
use renderer::sprite::animation::Frame;
use utils::Lazy;

use crate::bomberman::Bomberman;
use crate::consts::SCREEN_HEIGHT;
use crate::consts::SCREEN_WIDTH;
use crate::level::Level;

pub struct S {
    bomberman: Lazy<Bomberman>,
    level: Lazy<Level>,
}

impl Scene for S {
    fn load(&mut self, ctx: karna::ContextRefMut) {
        let bomberman = ctx.assets.load_png(include_bytes!("images/bomberman.png"));
        let sprite = AnimatedSprite::new(
            bomberman,
            Animations::default()
                .add_animation(
                    "walk-left",
                    Animation::default()
                        .add_frame(Frame::new(0, 0, 16, 25, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 0, 16, 25, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 0, 16, 25, Duration::from_millis(200))),
                )
                .add_animation(
                    "walk-down",
                    Animation::default()
                        .add_frame(Frame::new(1, 30, 14, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 30, 15, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 30, 15, 24, Duration::from_millis(200))),
                )
                .add_animation(
                    "walk-right",
                    Animation::default()
                        .add_frame(Frame::new(0, 58, 16, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 58, 16, 24, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 58, 16, 24, Duration::from_millis(200))),
                )
                .add_animation(
                    "walk-up",
                    Animation::default()
                        .add_frame(Frame::new(0, 86, 15, 23, Duration::from_millis(200)))
                        .add_frame(Frame::new(16, 87, 15, 22, Duration::from_millis(200)))
                        .add_frame(Frame::new(32, 86, 15, 23, Duration::from_millis(200))),
                ),
        );

        self.bomberman
            .set(Bomberman::new(Vector2::new(100.0, 100.0), sprite));
        self.bomberman.load();

        // Create the level after we loaded other assets, because this consumes `ctx`.
        self.level
            .set(Level::new(include_str!("levels/1.lvl"), ctx).expect("invalid level"));
    }

    fn update(&mut self, ctx: karna::ContextRefMut) {
        self.bomberman.update(ctx);
    }

    fn draw(&self, ctx: karna::ContextRef, draw: &mut renderer::Draw) {
        self.level.render(draw);
        self.bomberman.draw(draw);
    }
}

fn main() {
    App::builder()
        .with_window(
            WindowBuilder::default()
                .with_title("Sprite demo")
                .with_size((SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))
                .with_initial_scene(S {
                    bomberman: Lazy::new(),
                    level: Lazy::new(),
                }),
        )
        .build()
        .run()
}
