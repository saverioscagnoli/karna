use karna::{
    render::Color,
    traits::{Draw, Load, Update},
    App, Context,
};
use std::{char, vec};

struct Game {
    a: f32,
    b: f32,
    chars_to_draw: Vec<(char, i32, i32)>,
    color: Color,
}

const CHARS: [&str; 13] = [
    " ", ".", ",", "-", "~", ":", ";", "=", "!", "*", "#", "$", "@",
];

impl Load for Game {
    fn load(&mut self, _ctx: &mut Context) {}
}

impl Update for Game {
    fn update(&mut self, ctx: &mut Context) {
        self.chars_to_draw.clear();

        let dt = ctx.time.delta();

        self.a += 1.0 * dt;
        self.b += 0.5 * dt;

        let r = ((self.a * 2.0).sin() * 127.0 + 128.0).clamp(0.0, 255.0) as u8;
        let g = ((self.b * 2.0).sin() * 127.0 + 128.0).clamp(0.0, 255.0) as u8;
        let b = ((self.a * 2.0).cos() * 127.0 + 128.0).clamp(0.0, 255.0) as u8;

        self.color = Color::RGB(r, g, b);

        let x_sep = 10.0;
        let y_sep = 20.0;

        let size = ctx.window.size();

        let rows = size.height / y_sep as u32;
        let cols = size.width / x_sep as u32;
        let screen_size = rows * cols;

        let x_offset = cols / 2;
        let y_offset = rows / 2;

        let mut z = vec![0.0; screen_size as usize];
        let mut b = vec![" "; screen_size as usize];

        for j in (0..628).step_by(10) {
            for i in (0..628).step_by(1) {
                let c = (i as f32).sin();
                let d = (j as f32).cos();
                let e = self.a.sin();
                let f = (j as f32).sin();
                let g = self.a.cos();
                let h = d + 2.0;
                let dd = 1.0 / (c * h * e + f * g + 5.0);
                let l = (i as f32).cos();
                let m = self.b.cos();
                let n = self.b.sin();
                let t = c * h * g - f * e;
                let x = ((x_offset as f32) + 40.0 * dd * (l * h * m - t * n)) as usize;
                let y = ((y_offset as f32) + 20.0 * dd * (l * h * n + t * m)) as usize;
                let o = (x as f32 + cols as f32 * y as f32) as usize;
                let nn = 8.0 * ((f * e - c * d * g) * m - c * d * e - f * g - l * d * n);

                if rows > y as u32 && y > 0 && x > 0 && cols > x as u32 && dd > z[o] {
                    z[o] = dd;
                    b[o] = &CHARS[(nn as f64).max(0.0).min(11.0) as usize];
                }
            }
        }

        let mut x_start = 0.0;
        let mut y_start = 0.0;

        for i in 0..b.len() {
            if i % (cols as usize) == 0 {
                if i != 0 {
                    y_start += y_sep;
                }
                x_start = 0.0;
            }
            self.chars_to_draw
                .push((b[i].chars().next().unwrap(), x_start as i32, y_start as i32));
            x_start += x_sep;
        }
    }

    fn fixed_update(&mut self, _ctx: &mut Context) {}
}

impl Draw for Game {
    fn draw(&mut self, ctx: &mut Context) {
        ctx.render.fill_text(ctx.time.fps(), (10, 10), Color::WHITE);

        for (c, x, y) in &self.chars_to_draw {
            ctx.render.fill_text(c, (*x, *y), self.color);
        }

        ctx.render.set_color(Color::BLACK);
    }
}

fn main() {
    App::new("donut", (1280, 720)).unwrap().run(&mut Game {
        a: 0.0,
        b: 0.0,
        chars_to_draw: vec![],
        color: Color::WHITE,
    });
}
