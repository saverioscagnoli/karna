use core::f32::consts::TAU;

use karna::prelude::*;

const TRANSFORM_SCENE: SceneId = SceneId::new_str("transform");
const IDLE: Label = Label::new_str("idle");
const RUN: Label = Label::new_str("run");

const SCALE: f32 = 4.0;
const SPEED: f32 = 300.0;
const ORBITERS: usize = 6;

struct TransformDemo {
    witch: Sprite,
    orbiters: Vec<Sprite>,
    pos: math::Vector2<f32>,
    facing: f32,
    spin: f32,
    t: f32,
}

impl Scene for TransformDemo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        ctx.time.set_target_fps(120);

        let idle = ctx.assets.load_image("assets/witch-idle.png");
        let run = ctx.assets.load_image("assets/witch-run.png");

        let sheet = ctx.assets.add_spritesheet(
            Spritesheet::new()
                .with_strip(IDLE, idle, 6, 8.0)
                .with_strip(RUN, run, 8, 12.0),
        );

        let orbit_sheet = ctx.assets.add_spritesheet(Spritesheet::strip(idle, 6, 8.0));

        let orbiters = (0..ORBITERS)
            .map(|i| {
                let mut s = Sprite::new(orbit_sheet);
                s.update(i as f32 * 0.13);
                s
            })
            .collect();

        let size = ctx.window.size().cast::<f32>();

        Self {
            witch: Sprite::new(sheet),
            orbiters,
            pos: math::vec2!(size.w() / 2.0, size.h() / 2.0),
            facing: 1.0,
            spin: 0.0,
            t: 0.0,
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        let dt = ctx.time.delta();
        self.t += dt;

        let mut dir = math::vec2!(0.0f32, 0.0);

        if ctx.input.key_down(Key::A) {
            dir.x -= 1.0;
        }
        if ctx.input.key_down(Key::D) {
            dir.x += 1.0;
        }
        if ctx.input.key_down(Key::W) {
            dir.y -= 1.0;
        }
        if ctx.input.key_down(Key::S) {
            dir.y += 1.0;
        }

        if dir.x != 0.0 {
            self.facing = dir.x.signum();
        }

        if ctx.input.key_down(Key::Q) {
            self.spin -= TAU * 0.5 * dt;
        }
        if ctx.input.key_down(Key::E) {
            self.spin += TAU * 0.5 * dt;
        }
        if ctx.input.key_pressed(Key::R) {
            self.spin = 0.0;
        }

        self.pos += dir * SPEED * dt;

        if dir.x != 0.0 || dir.y != 0.0 {
            self.witch.play(RUN);
        } else {
            self.witch.play(IDLE);
        }

        self.witch.update(dt);

        for s in &mut self.orbiters {
            s.update(dt);
        }
    }

    fn draw(&mut self, _ctx: &mut DrawContext, draw: &mut Draw) {
        let (w, h) = (32.0 * SCALE, 48.0 * SCALE);

        draw.scoped(|d| {
            d.translate(self.pos.x, self.pos.y);

            d.scoped(|d| {
                d.rotate(self.spin);
                d.scale(self.facing, 1.0);
                d.translate(-w / 2.0, -h / 2.0);
                d.sprite_sized(&self.witch, 0.0, 0.0, w, h);
            });

            d.set_color(Color::rgba(1.0, 1.0, 1.0, 0.25));
            d.set_thickness(2.0);
            d.circle_outline(0.0, 0.0, 220.0);

            d.rotate(self.t * 0.6);

            for (i, s) in self.orbiters.iter().enumerate() {
                let angle = i as f32 * TAU / ORBITERS as f32;
                let pulse = 1.0 + 0.3 * (self.t * 3.0 + angle).sin();

                d.scoped(|d| {
                    d.rotate(angle);
                    d.translate(220.0, 0.0);
                    d.rotate(-self.t * 2.0);
                    d.scale(pulse * 1.5, pulse * 1.5);
                    d.set_color(Color::WHITE);
                    d.sprite(s, -16.0, -24.0);
                });
            }
        });

        draw.print("WASD move, A/D flips, Q/E spin, R reset", 10.0, 10.0);
        draw.print(format!("spin: {:.2} rad", self.spin), 10.0, 30.0);
    }
}

fn main() {
    _ = traccia::init(
        traccia::Config::default()
            .with_min_level(traccia::LevelFilter::Debug)
            .with_module_filter("cosmic_text", traccia::LevelFilter::Warn)
            .with_target(SdlTarget::default()),
    );

    App::builder()
        .with_root("karna/examples/")
        .with_window(
            WindowBuilder::default()
                .with_title("transform demo")
                .with_size((1280, 720))
                .with_scene::<TransformDemo>(TRANSFORM_SCENE)
                .with_active_scene(TRANSFORM_SCENE),
        )
        .build()
        .run();
}
