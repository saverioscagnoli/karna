use std::time::Duration;

use karna::{
    context::Context,
    core::App,
    flags::LoopFlag,
    input::Key,
    math::{Easing, Tween, Vec2},
    traits::{Load, Render, Update},
};
use sdl2::pixels::Color;

struct Game {
    pos: Vec2,
    tween: Tween<f32>,
    started: bool,
    r: f32,
}

impl Load for Game {
    fn load(&mut self, ctx: &mut Context) {
        let size = ctx.window.size();

        self.pos = (size.width as f32 / 2.0, size.height as f32 / 2.0).into();
        ctx.render
            .load_font("default", "examples/assets/font.ttf", 20);
        ctx.render.set_font("default");
    }
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.time.delta();
        let acc = 250.0;

        if ctx.input.key_down(Key::W) {
            self.pos.y -= acc * dt;
        }

        if ctx.input.key_down(Key::S) {
            self.pos.y += acc * dt;
        }

        if ctx.input.key_down(Key::A) {
            self.pos.x -= acc * dt;
        }

        if ctx.input.key_down(Key::D) {
            self.pos.x += acc * dt;
        }

        if ctx.input.key_pressed(Key::SPACE) {
            self.started = !self.started;
        }

        if self.started {
            self.r = self.tween.update(dt);

            if self.tween.is_over() {
                self.started = false;
                self.tween = self.tween.reversed();
            }
        }
    }
}

impl Render for Game {
    fn render(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::WHITE);

        ctx.render.fill_circle(self.pos, self.r);

        ctx.render.fill_text(ctx.time.fps(), (10, 10), Color::WHITE);

        ctx.render.set_color(Color::BLACK);
    }
}

fn main() {
    App::new()
        .unwrap()
        .flags(&[LoopFlag::Accelerated])
        .window("Move with WASD!", (800, 600))
        .run(&mut Game {
            pos: Vec2::zero(),
            tween: Tween::new(
                0.0,
                200.0,
                Duration::from_secs_f32(1.5),
                Easing::QuadraticInOut,
            ),
            started: false,
            r: 0.0,
        });
}
