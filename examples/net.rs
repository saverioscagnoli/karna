#![allow(unused)]

use std::f32;

use karna::prelude::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct TextPacket {
    text: String,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct PosPacket {
    pos: Vector2<f32>,
}

impl Packet for TextPacket {
    const ID: u16 = 1;
}

impl Packet for PosPacket {
    const ID: u16 = 2;
}

struct S {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    image: Handle<Image>,
    mammamia: Handle<Audio>,
    buffer: String,

    pos2: Vector2<f32>,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        ctx.net.connect("0.0.0.0:5000");

        ctx.assets.write_scope(|a| {
            self.image = a.load_image(include_bytes!("assets/pcb.png"));
            self.mammamia = a.load_audio(include_bytes!("assets/mm.mp3"));
        });

        ctx.time.set_target_fps(120);
    }

    fn on_text_input(&mut self, ctx: ContextMut, scene: &mut SceneHandle, text: &str) {
        self.buffer.push_str(text);
    }

    fn fixed_update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {}

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        let accel = 5000.0;
        let dt = ctx.time.delta();

        self.vel.y += ctx.input.key_axis([Keycode::KeyW, Keycode::KeyS]) * accel * dt;
        self.vel.x += ctx.input.key_axis([Keycode::KeyA, Keycode::KeyD]) * accel * dt;

        self.vel *= 0.85f32.powf(60.0).powf(dt);
        self.pos += self.vel * dt;

        for msg in ctx.net.messages::<TextPacket>() {
            println!("Receiveddd {:?}", msg);
        }

        for msg in ctx.net.messages::<PosPacket>() {
            self.pos2 = msg.pos;
        }

        if ctx.input.key_pressed(Keycode::Space) {
            ctx.mixer.play(self.mammamia);
        }

        ctx.net.send(&PosPacket { pos: self.pos });
    }

    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw) {
        draw.set_color(Color::White);

        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);

        draw.set_color(Color::Cyan);

        draw.push_state();
        draw.translate(self.pos.x + 100.0, self.pos.y);
        draw.rotate(f32::consts::PI / 4.0);
        draw.rect(0.0, 0.0, 50.0, 50.0);
        draw.pop_state();

        draw.set_color(Color::Magenta);

        draw.push_state();
        draw.translate(self.pos.x, self.pos.y + 100.0);
        draw.scale(2.0, 1.0);
        draw.rect(0.0, 0.0, 50.0, 50.0);
        draw.pop_state();

        draw.set_color(Color::Magenta);

        draw.line_v([300.0, 100.0], self.pos + Vector2::new(25.0, 25.0));

        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.point(i as f32 * 10.0 + 200.0, j as f32 * 10.0 + 200.0);
            }
        }

        draw.set_color(Color::Magenta);
        draw.rect_v(self.pos2, (50.0, 50.0));

        draw.image(self.image, 800.0, 300.0);

        draw.set_color(Color::White);
        draw.debug_text(
            &format!("dt {:.6}\nfps {}", ctx.time.delta(), ctx.time.fps()),
            10.0,
            10.0,
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
                .with_size(Size::new(1280, 720))
                .with_scene(
                    "demo",
                    S {
                        pos: Vector2::new(50.0, 50.0),
                        vel: Vector2::zero(),
                        mammamia: Handle::default(),
                        image: Handle::default(),
                        buffer: String::new(),
                        pos2: Vector2::new(50.0, 50.0),
                    },
                )
                .with_active_scene("demo"),
        )
        .build()
        .run();
}
