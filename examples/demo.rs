use engine::SceneHandle;
use engine::input::Keycode;
use karna::App;
use karna::ContextMut;
use karna::ContextRef;
use karna::Scene;
use karna::WindowBuilder;
use karna::render::Color;
use karna::render::Draw;
use math::Vector2;

struct S {
    prev_pos: Vector2<f32>,
    vel: Vector2<f32>,
    pos: Vector2<f32>,
    r: f32,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        ctx.time.set_target_fps(120);
    }

    fn update(&mut self, _ctx: ContextMut, _scene: &mut SceneHandle) {}

    fn fixed_update(&mut self, ctx: ContextMut, _scene: &mut SceneHandle) {
        const ACCEL: f32 = 5000.0;
        const MAX_SPEED: f32 = 900.0;
        const FRICTION: f32 = 0.85; // fraction of velocity retained per 1/60s tick

        let dt = ctx.time.fixed_delta();
        self.prev_pos = self.pos;

        let input = Vector2::new(
            ctx.input.key_axis([Keycode::KeyA, Keycode::KeyD]),
            ctx.input.key_axis([Keycode::KeyW, Keycode::KeyS]),
        );

        // Normalize so diagonal movement isn't faster than axis movement.
        let input = if input.length_sq() > 1.0 {
            input.normalize()
        } else {
            input
        };

        self.vel += input * ACCEL * dt;

        // Frame-rate independent friction, applied whether or not there's input,
        // so releasing keys decelerates the same way regardless of fps.
        self.vel *= FRICTION.powf(60.0 * dt);

        // Explicit speed cap instead of an implicit one derived from the friction constant.
        if self.vel.length_sq() > MAX_SPEED * MAX_SPEED {
            self.vel = self.vel.normalize() * MAX_SPEED;
        }

        self.pos += self.vel * dt;
        self.r += dt;
    }

    fn draw(&self, ctx: ContextRef, draw: &mut Draw) {
        let render_pos = self.prev_pos.lerp(&self.pos, ctx.time.alpha());

        draw.set_color(Color::Red);

        draw.push_state();
        draw.translate(render_pos.x, render_pos.y);
        draw.rotate(self.r);
        draw.rect(-25.0, -25.0, 50.0, 50.0);
        draw.pop_state();
    }
}

struct A {
    t: f32,
}

impl Scene for A {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {}

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        self.t += ctx.time.delta();
    }

    fn draw(&self, ctx: ContextRef, draw: &mut Draw) {
        let r = (self.t.sin() * 0.5) + 0.5;
        let g = (self.t.cos() * 0.5) + 0.5;
        let b = (self.t.tan() * 0.5) + 0.5;

        draw.set_clear_color(Color::rgb(r, g, b));
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
                .with_title("demo")
                .with_size((1280, 720))
                .with_scene(
                    "demo",
                    S {
                        prev_pos: Vector2::zero(),
                        vel: Vector2::zero(),
                        pos: Vector2::zero(),
                        r: 0.0,
                    },
                )
                .with_active_scene("demo"),
        )
        //  .with_window(
        //      WindowBuilder::new()
        //          .with_title("demo2")
        //          .with_size((800, 600))
        //          .with_scene("demo-2", A { t: 0.0 })
        //          .with_active_scene("demo-2"),
        //  )
        .build()
        .run();
}
