#![allow(unused)]

use karna::prelude::*;

const DEMO_SCENE: SceneId = SceneId::new_str("demo");
const ACCEL: f32 = 250.0;

struct Demo {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    pcb: Handle<Image>,
}

impl Scene for Demo {
    fn load(ctx: LoadContext) -> Self
    where
        Self: Sized,
    {
        let pcb = ctx.assets.load_image("assets/pcb.png");

        Self {
            pos: Vector2::new(10.0, 10.0),
            vel: Vector2::zero(),
            pcb,
        }
    }

    fn fixed_update(&mut self, ctx: UpdateContext) {
        let dt = ctx.time.delta();

        if ctx.input.key_down(Key::W) {
            self.vel.y = -ACCEL;
        }

        if ctx.input.key_down(Key::A) {
            self.vel.x = -ACCEL;
        }

        if ctx.input.key_down(Key::S) {
            self.vel.y = ACCEL;
        }

        if ctx.input.key_down(Key::D) {
            self.vel.x = ACCEL;
        }

        self.pos += self.vel * dt;
        self.vel *= 0.9;

        if self.vel.length() < 0.25 {
            self.vel.set([0.0, 0.0]);
        }
    }

    fn update(&mut self, ctx: UpdateContext) {}

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.set_color(Color::RED);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);

        draw.set_color(Color::CYAN);
        draw.rect(400.0, 300.0, 160.0, 160.0);

        draw.set_color(Color::WHITE);
        draw.image(self.pcb, 700.0, 120.0);
        draw.image_sized(self.pcb, 700.0, 420.0, 128.0, 128.0);
    }
}

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    AppBuilder::default()
        .with_window(
            WindowBuilder::default()
                .with_title("Demo Window")
                .with_size((1280, 720))
                .with_scene::<Demo>(DEMO_SCENE)
                .with_active_scene(DEMO_SCENE),
        )
        .with_root("examples/")
        .build()
        .run();
}
