use std::time::Duration;

use karna::{
    input::Key,
    math::{pick, rng, Easing, Tween, Vec2},
    render::Color,
    traits::{Draw, Load, Update},
    App, Context,
};

struct Game {
    pos: Vec2,
    tween: Tween<Vec2>,
    target: Vec2,
}

impl Load for Game {
    fn load(&mut self, ctx: &mut Context) {
        ctx.window.set_resizable(true);
        ctx.time.set_target_fps(144);
    }
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        self.pos = self.tween.update(ctx.time.delta());

        if self.tween.finished() {
            let size = ctx.window.size();
            let x = rng(25..=size.width - 25);
            let y = rng(25..=size.height - 25);

            let target = (x, y).into();

            self.tween = Tween::new(
                self.pos,
                target,
                Duration::from_secs(2),
                *pick(&Easing::all()),
            );

            self.tween.start();

            self.target = target;
        }

        if ctx.input.key_pressed(Key::Space) {
            if self.tween.paused() {
                self.tween.start();
            } else {
                self.tween.pause();
            }
        }
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::RED);
        ctx.render.fill_aa_circle(self.pos, 25);

        ctx.render.set_color(Color::GREEN);
        ctx.render.fill_aa_circle(self.target, 5);

        ctx.render.set_color(Color::BLACK);

        ctx.render
            .fill_text(format!("FPS: {}", ctx.time.fps()), (10, 10), Color::WHITE);

        ctx.render.fill_text(
            format!("Current easing: {}", self.tween.easing().name()),
            (10, 30),
            Color::WHITE,
        );

        ctx.render.fill_text(
            format!(
                "{:.1}s / {:.1}s",
                self.tween.elapsed(),
                self.tween.duration().as_secs_f32()
            ),
            (10, 50),
            Color::WHITE,
        );
    }
}

fn main() {
    let mut initial_tween = Tween::new(
        Vec2::zero(),
        (500, 500).into(),
        Duration::from_secs_f32(2.0),
        Easing::CubicBezier(0.17, 0.67, 0.83, 0.67),
    );

    initial_tween.start();

    App::new("Tweeeeeeeeeeeeeeens", (800, 600))
        .unwrap()
        .run(&mut Game {
            pos: Vec2::zero(),
            tween: initial_tween,
            target: (500, 500).into(),
        });
}
