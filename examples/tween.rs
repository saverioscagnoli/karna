use karna::{
    input::Key,
    math::{pick, rng, Easing, Size, Tween, Vec2},
    render::Color,
    traits::Scene,
    App, Context,
};
use std::time::Duration;

const EASINGS: &[Easing] = &[
    Easing::Linear,
    Easing::InSine,
    Easing::OutSine,
    Easing::InOutSine,
    Easing::InQuad,
    Easing::OutQuad,
    Easing::InOutQuad,
    Easing::InCubic,
    Easing::OutCubic,
    Easing::InOutCubic,
    Easing::InQuart,
    Easing::OutQuart,
    Easing::InOutQuart,
    Easing::InQuint,
    Easing::OutQuint,
    Easing::InOutQuint,
    Easing::InExpo,
    Easing::OutExpo,
    Easing::InOutExpo,
    Easing::InCirc,
    Easing::OutCirc,
    Easing::InOutCirc,
    Easing::InBack,
    Easing::OutBack,
    Easing::InOutBack,
    Easing::InElastic,
    Easing::OutElastic,
    Easing::InOutElastic,
    Easing::InBounce,
    Easing::OutBounce,
    Easing::InOutBounce,
];

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct Inventory {
    pos: Vec2,
    size: Size,
    tween: Tween<Vec2>,
}

impl Inventory {
    fn new() -> Self {
        let size = Size::new(WIDTH, HEIGHT);
        let inv_size = Size::new(500, 300);

        Self {
            pos: Vec2::zero(),
            size: (500, 300).into(),
            tween: Tween::new(
                Vec2::new(size.fit_center_x(inv_size.width), HEIGHT as f32),
                size.fit_center(inv_size),
                Duration::from_millis(200),
                Easing::CubicBezier(0.47, 1.6, 1.0, 1.0),
            ),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.pos = self.tween.move_by(ctx.time.delta());

        if self.tween.is_finished() {
            self.tween.reverse();
        }

        if ctx.input.key_pressed(Key::E) {
            if !self.tween.is_running() {
                self.tween.start();
            }
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        ctx.render.set_color(Color::WHITE);
        ctx.render.fill_rect(self.pos, self.size);
    }
}

struct Circle {
    pos: Vec2,
    tween: Tween<Vec2>,
}

impl Circle {
    fn new() -> Self {
        Self {
            pos: Vec2::zero(),
            tween: Tween::new_and_start(
                Vec2::zero(),
                Size::new(800, 600).center(),
                Duration::from_millis(1000),
                Easing::InSine,
            ),
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.pos = self.tween.move_by(ctx.time.delta());

        if self.tween.is_finished() {
            let size = ctx.window.size();
            let x = rng(25..=size.width - 25);
            let y = rng(25..=size.height - 25);

            let target = (x, y).into();

            self.tween =
                Tween::new_and_start(self.pos, target, Duration::from_secs(2), *pick(EASINGS));
        }

        if ctx.input.key_pressed(Key::Space) {
            if self.tween.is_paused() {
                self.tween.start();
            } else {
                self.tween.pause();
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::RED);
        ctx.render.fill_aa_circle(self.pos, 25);

        ctx.render.set_color(Color::GREEN);
        ctx.render.fill_aa_circle(self.tween.target(), 5);

        ctx.render.set_color(Color::BLACK);

        ctx.render.fill_text(
            format!("Current easing: {}", self.tween.easing().to_string()),
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

struct FirstScene {
    circle: Circle,
    inventory: Inventory,
}

impl Scene for FirstScene {
    fn load(&mut self, _ctx: &mut Context) {}

    fn update(&mut self, ctx: &mut Context) {
        self.circle.update(ctx);
        self.inventory.update(ctx);
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.render
            .fill_text(format!("FPS: {}", ctx.time.fps()), (10, 10), Color::WHITE);

        ctx.render.fill_text(
            "Press SPACE to pause/resume the circle tween",
            (10, 70),
            Color::WHITE,
        );

        ctx.render
            .fill_text("Press E to show/hide the inventory", (10, 90), Color::WHITE);

        self.circle.draw(ctx);
        self.inventory.draw(ctx);

        ctx.render.set_color(Color::BLACK);
    }
}

fn main() {
    App::window("Tweeeeeeeeeeeeeeens", (800, 600)).run(FirstScene {
        circle: Circle::new(),
        inventory: Inventory::new(),
    });
}
