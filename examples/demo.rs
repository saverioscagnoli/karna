#![allow(unused)]
use std::f32;
use std::f32::consts::PI;
use std::f32::consts::TAU;

use karna::prelude::*;

struct Demo {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    image: Handle<Image>,
    t: f32,
}

impl Demo {
    fn new() -> Self {
        Self {
            pos: Vector2::new(80.0, 80.0),
            vel: Vector2::zero(),
            image: Handle::default(),
            t: 0.0,
        }
    }
}

impl Scene for Demo {
    fn load(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        ctx.assets.write_scope(|a| {
            self.image = a.load_image(include_bytes!("assets/pcb.png"));
        });

        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        let dt = ctx.time.delta();
        self.t += dt;

        let accel = 5000.0;
        self.vel.y += ctx.input.key_axis([Keycode::KeyW, Keycode::KeyS]) * accel * dt;
        self.vel.x += ctx.input.key_axis([Keycode::KeyA, Keycode::KeyD]) * accel * dt;
        self.vel *= 0.85f32.powf(60.0).powf(dt);
        self.pos += self.vel * dt;
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        let t = self.t;

        draw.set_clear_color(Color::rgb(0.08, 0.09, 0.11));

        draw.set_color(Color::White);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);

        draw.set_color(Color::Cyan);
        draw.push_state();
        draw.translate(240.0, 105.0);
        draw.rotate(t);
        draw.rect(-25.0, -25.0, 50.0, 50.0);

        draw.push_state();
        draw.translate(60.0, 0.0);
        draw.rotate(-3.0 * t);
        draw.set_color(Color::Magenta);
        draw.rect(-10.0, -10.0, 20.0, 20.0);
        draw.pop_state();
        draw.pop_state();

        draw.set_color(Color::Orange);
        draw.push_state();
        draw.translate(380.0, 105.0);
        draw.scale(1.5 + (t * 2.0).sin() * 0.5, 1.0);
        draw.rotate(t * 0.5);
        draw.rect(-25.0, -25.0, 50.0, 50.0);
        draw.pop_state();

        draw.debug_text("transforms (WASD moves white)", 80.0, 40.0);

        let sy = 260.0;

        draw.set_color(Color::Red);
        draw.circle(120.0, sy, 40.0);

        draw.set_color(Color::Cyan);
        draw.circle_outline(230.0, sy, 40.0, 3.0 + ((t * 3.0).sin() + 1.0) * 4.0);

        let r_max = 25.0; // half of height 50
        let r = ((t.sin() + 1.0) * 0.5) * r_max;
        draw.set_color(Color::Orange);
        draw.rounded_rect(300.0, sy - 25.0, 120.0, 50.0, r);

        draw.set_color(Color::Magenta);
        draw.rounded_rect_outline(450.0, sy - 25.0, 120.0, 50.0, 12.0, 3.0);

        draw.set_color(Color::White);
        draw.rect_outline(600.0, sy - 25.0, 80.0, 50.0, 2.0);

        draw.debug_text(
            "sdf shapes: fill / outline / animated radius",
            80.0,
            sy - 70.0,
        );

        let ly = 400.0;

        for i in 0..5 {
            draw.set_line_width(1.0 + i as f32 * 3.0);
            draw.set_color(Color::White);

            let y = ly + i as f32 * 22.0;

            draw.line(80.0, y, 260.0, y);
        }

        draw.set_line_width(4.0);
        draw.set_color(Color::Cyan);

        let mut prev = Vector2::new(300.0, ly + 40.0);

        for i in 1..=40 {
            let x = 300.0 + i as f32 * 8.0;
            let y = ly + 40.0 + ((x * 0.03) + t * 3.0).sin() * 30.0;
            let next = Vector2::new(x, y);

            draw.line_v(prev, next);
            prev = next;
        }

        draw.set_line_width(6.0);
        draw.set_color(Color::Orange);

        let (cx, cy) = (720.0, ly + 45.0);

        draw.line(cx, cy, cx + t.cos() * 55.0, cy + t.sin() * 55.0);
        draw.set_color(Color::White);
        draw.circle_outline(cx, cy, 60.0, 2.0);

        for i in 0..10 {
            for j in 0..5 {
                let phase = (t * 4.0 - (i + j) as f32 * 0.4).sin() * 0.5 + 0.5;

                draw.set_color(Color::rgba(0.3, 0.9, 1.0, 0.25 + phase * 0.75));
                draw.point(830.0 + i as f32 * 14.0, ly + 20.0 + j as f32 * 14.0);
            }
        }

        draw.set_line_width(1.0);
        draw.debug_text(
            "capsules: width ramp / polyline / caps / points",
            80.0,
            ly - 30.0,
        );

        let (ox, oy) = (150.0, 590.0);

        draw.set_color(Color::rgba(1.0, 0.2, 0.2, 0.6));
        draw.circle(ox, oy, 45.0); // drawn first -> bottom

        draw.set_color(Color::rgba(0.2, 1.0, 0.2, 0.6));
        draw.rect(ox - 20.0, oy - 20.0, 80.0, 60.0); // on top of circle

        draw.set_line_width(10.0);
        draw.set_color(Color::rgba(0.4, 0.4, 1.0, 0.6));
        draw.line(ox - 60.0, oy + 30.0, ox + 90.0, oy - 40.0); // topmost
        draw.set_line_width(1.0);

        draw.debug_text("order test: circle < rect < line", ox - 70.0, oy - 80.0);

        draw.push_state();
        draw.translate(1000.0, 130.0);
        draw.rotate((t * 0.7).sin() * 0.3);
        draw.set_color(Color::White);
        draw.image_v(self.image, Vector2::new(-64.0, -64.0));
        draw.pop_state();

        draw.set_color(Color::rgba(1.0, 0.4, 0.4, 0.5));
        draw.image(self.image, 950.0, 280.0);

        draw.set_color(Color::White);
        draw.debug_text("images: rotated / tinted", 950.0, 40.0);

        let (bx, by, bw, bh) = (480.0, 560.0, 220.0, 60.0);
        let pulse = (t * 2.0).sin() * 0.5 + 0.5;

        draw.set_color(Color::rgba(0.15, 0.17, 0.22, 0.9));
        draw.rounded_rect(bx, by, bw, bh, 14.0);
        draw.set_color(Color::rgba(0.3 + pulse * 0.6, 0.9, 1.0, 1.0));
        draw.rounded_rect_outline(bx, by, bw, bh, 14.0, 2.0 + pulse * 2.0);
        draw.set_color(Color::White);
        draw.debug_text("Mamma Mia!", bx + 30.0, by + 22.0);

        draw.set_layer(Layer::Ui);
        draw.set_color(Color::rgba(0.0, 0.0, 0.0, 0.5));
        draw.rounded_rect(6.0, 6.0, 150.0, 44.0, 8.0);
        draw.set_color(Color::White);
        draw.debug_text(
            &format!("dt {:.7}\nfps {}", ctx.time.delta(), ctx.time.fps()),
            14.0,
            12.0,
        );
    }
}

fn main() {
    karna::logging::init(
        karna::logging::Config {
            min_level: karna::logging::LevelFilter::Debug,
            ..Default::default()
        }
        .hide_wgpu(true),
    )
    .expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("demo window")
                .with_size(Size::new(1280, 720))
                .with_scene("demo", Demo::new())
                .with_active_scene("demo"),
        )
        .build()
        .run();
}
