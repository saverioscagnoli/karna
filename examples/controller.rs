use std::time::Duration;

use karna::{
    input::Button,
    math::Vec2,
    render::Color,
    traits::{Draw, Load, Update},
    App, Context,
};

struct Game;

impl Load for Game {
    fn load(&mut self, _ctx: &mut Context) {}
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        let lt = ctx.input.left_trigger();
        let rt = ctx.input.right_trigger();

        ctx.input.rumble(lt, rt, Duration::from_millis(100));
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.set_color(Color::GREEN);

        if ctx.input.button_down(Button::A) {
            ctx.render.fill_circle((600, 450), 10);
        } else {
            ctx.render.draw_circle((600, 450), 10);
        }

        ctx.render.set_color(Color::RED);

        if ctx.input.button_down(Button::B) {
            ctx.render.fill_circle((620, 430), 10);
        } else {
            ctx.render.draw_circle((620, 430), 10);
        }

        ctx.render.set_color(Color::BLUE);

        if ctx.input.button_down(Button::X) {
            ctx.render.fill_circle((580, 430), 10);
        } else {
            ctx.render.draw_circle((580, 430), 10);
        }

        // Orange
        ctx.render.set_color(Color::RGB(255, 165, 0));

        if ctx.input.button_down(Button::Y) {
            ctx.render.fill_circle((600, 410), 10);
        } else {
            ctx.render.draw_circle((600, 410), 10);
        }

        ctx.render.set_color(Color::WHITE);

        if ctx.input.button_down(Button::DPadDown) {
            ctx.render.fill_rect((300, 430), (20, 30));
        } else {
            ctx.render.draw_rect((300, 430), (20, 30));
        }

        if ctx.input.button_down(Button::DPadUp) {
            ctx.render.fill_rect((300, 380), (20, 30));
        } else {
            ctx.render.draw_rect((300, 380), (20, 30));
        }

        if ctx.input.button_down(Button::DPadLeft) {
            ctx.render.fill_rect((270, 410), (30, 20));
        } else {
            ctx.render.draw_rect((270, 410), (30, 20));
        }

        if ctx.input.button_down(Button::DPadRight) {
            ctx.render.fill_rect((320, 410), (30, 20));
        } else {
            ctx.render.draw_rect((320, 410), (30, 20));
        }

        let ls = ctx.input.left_stick();

        let ls_center = Vec2::new(200, 200);
        let actual_ls = ls_center + ls * 50;

        ctx.render.draw_line(ls_center, actual_ls);
        ctx.render.fill_aa_circle(actual_ls, 5);

        let rs = ctx.input.right_stick();

        let rs_center = Vec2::new(600, 200);

        let actual_rs = rs_center + rs * 50;

        ctx.render.set_color(Color::WHITE);
        ctx.render.draw_line(rs_center, actual_rs);
        ctx.render.fill_aa_circle(actual_rs, 5);

        ctx.render.fill_text(
            format!("Left stick: ({:.2}, {:.2})", ls.x, ls.y),
            (10, 10),
            Color::WHITE,
        );

        ctx.render.fill_text(
            format!("Right stick: ({:.2}, {:.2})", rs.x, rs.y),
            (10, 35),
            Color::WHITE,
        );

        ctx.render.fill_text(
            format!("Left trigger: {}", ctx.input.left_trigger()),
            (10, 60),
            Color::CYAN,
        );

        ctx.render.fill_text(
            format!("Right trigger: {}", ctx.input.right_trigger()),
            (10, 85),
            Color::CYAN,
        );

        ctx.render.set_color(Color::BLACK);
    }
}

fn main() {
    App::new("controller testing", (800, 600))
        .unwrap()
        .run(&mut Game);
}
