use std::time::Duration;

use karna::{
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
    }
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        self.pos = self.tween.update(ctx.time.delta());

        if self.tween.finished() {
            let size = ctx.window.size();
            let x = rng(0, size.width);
            let y = rng(0, size.height);

            let target = (x, y).into();

            self.tween = Tween::new(
                self.pos,
                target,
                Duration::from_secs(2),
                *pick(&Easing::all()),
            );

            self.target = target;
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

        ctx.render.fill_text(
            format!("Current easing: {}", self.tween.easing().name()),
            (10.0, 10.0),
            Color::WHITE,
        );
        ctx.render.fill_text(
            format!(
                "{:.1}s / {:.1}s",
                self.tween.elapsed(),
                self.tween.duration().as_secs_f32()
            ),
            (10.0, 30.0),
            Color::WHITE,
        );
    }
}

fn main() {
    App::new("Tweeeeeeeeeeeeeeens", (800, 600))
        .unwrap()
        .run(&mut Game {
            pos: Vec2::zero(),
            tween: Tween::new(
                Vec2::zero(),
                (500, 500).into(),
                Duration::from_secs_f32(2.0),
                Easing::InOutBack,
            ),
            target: (500, 500).into(),
        });
}
