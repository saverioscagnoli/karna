use karna::App;
use karna::ContextRef;
use karna::ContextRefMut;
use karna::Handle;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Image;
use karna::input::KeyCode;
use karna::logging;
use karna::math::Vector2;
use karna::render::Color;
use karna::render::Draw;
use log::LevelFilter;

struct S {
    image: Handle<Image>,
    pos: Vector2<f32>,
    prev_pos: Vector2<f32>,
    vel: Vector2<f32>,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextRefMut) {
        let image_bytes = include_bytes!("./assets/tetsuo.png");
        let image = ctx.assets.load_image(image_bytes);

        self.image = image;

        if let Some(monitor) = ctx.monitors.current() {
            ctx.time.set_target_fps(monitor.refresh_rate());
        }

        ctx.time.set_target_fps(120);
    }

    fn fixed_update(&mut self, ctx: ContextRefMut) {
        const VEL: f32 = 250.0;

        // snapshot before mutating this tick
        self.prev_pos = self.pos;

        if ctx.input.key_held(&KeyCode::KeyW) {
            self.vel.y = -VEL;
        }

        if ctx.input.key_held(&KeyCode::KeyA) {
            self.vel.x = -VEL;
        }

        if ctx.input.key_held(&KeyCode::KeyS) {
            self.vel.y = VEL;
        }

        if ctx.input.key_held(&KeyCode::KeyD) {
            self.vel.x = VEL;
        }

        self.pos += self.vel * ctx.time.fixed_delta();
        self.vel *= 0.9;

        if self.vel.length_sq() < 0.01 {
            self.vel.set([0.0, 0.0]);
        }
    }

    fn update(&mut self, ctx: ContextRefMut) {
        let _ = ctx;
    }

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {
        draw.set_color(Color::Cyan);

        for i in 0..10 {
            for j in 0..10 {
                draw.point(40.0 + i as f32 * 10.0, 100.0 + j as f32 * 10.0);
            }
        }

        let alpha = ctx.time.alpha();
        let render_pos = self.prev_pos.lerp(&self.pos, alpha);

        draw.set_color(Color::White);
        draw.image(self.image, render_pos.x, render_pos.y);

        draw.set_color(Color::Cyan);
        draw.circle(300.0, 300.0, 50.0);

        draw.set_color(Color::Magenta);
        draw.line_v([10.0, 50.0], [124.0, 478.0]);

        draw.set_color(Color::White);
        draw.debug_text(
            format!("fps {}\ndt {:.6}s", ctx.time.fps(), ctx.time.delta()),
            10.0,
            10.0,
        );
    }
}

fn main() {
    logging::init(
        logging::Config::default()
            .with_min_level(log::LevelFilter::Debug)
            .with_module_filter("sctk", LevelFilter::Error)
            .with_module_filter("naga", log::LevelFilter::Error)
            .with_module_filter("wgpu", log::LevelFilter::Error),
    )
    .expect("Failed to init logging");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_size((1280, 720))
                .with_scene(
                    "initial",
                    S {
                        pos: Vector2::new(10.0, 10.0),
                        prev_pos: Vector2::new(10.0, 10.0),
                        vel: Vector2::zero(),
                        image: Handle::default(),
                    },
                )
                .with_active_scene("initial"),
        )
        .build()
        .run();
}
