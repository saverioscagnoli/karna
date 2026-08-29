#![allow(unused)]

use karna::prelude::*;

const DEMO_SCENE: SceneId = SceneId::new_str("demo");
const DEMO2_SCENE: SceneId = SceneId::new_str("demo2");
const ACCEL: f32 = 250.0;

struct Demo {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    pcb: Handle<Image>,
    cob: Handle<Image>,
}

impl Scene for Demo {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        let pcb = ctx.assets.load_image("assets/pcb.png");
        let cob = ctx.assets.load_image("assets/cobblestone.png");

        Self {
            pos: Vector2::new(10.0, 10.0),
            vel: Vector2::zero(),
            pcb,
            cob,
        }
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
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

        let mouse = ctx.window.mouse_position();
        let inside = mouse.x >= 400.0 && mouse.x < 560.0 && mouse.y >= 300.0 && mouse.y < 460.0;

        if inside {
            ctx.window
                .set_cursor(Cursor::Custom(self.cob, Vector2::zero()));
        } else {
            ctx.window.set_cursor(Cursor::default());
        }

        if ctx.input.key_pressed(Key::F) {
            ctx.window.activate_scene(DEMO2_SCENE);
            ctx.window.deactivate_scene(DEMO_SCENE);
        }
    }

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

struct Demo2;

impl Scene for Demo2 {
    fn load(ctx: &mut LoadContext) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn update(&mut self, ctx: &mut UpdateContext) {
        if ctx.input.key_pressed(Key::F) {
            ctx.window.deactivate_scene(DEMO2_SCENE);
            ctx.window.activate_scene(DEMO_SCENE);
        }
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.set_color(Color::MAGENTA);
        draw.rect(300.0, 400.0, 150.0, 450.0);
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
                .with_scene::<Demo2>(DEMO2_SCENE)
                .with_active_scene(DEMO_SCENE),
        )
        .with_root("examples/")
        .build()
        .run();
}
