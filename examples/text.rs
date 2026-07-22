#![allow(unused)]

use karna::App;
use karna::Context;
use karna::Scene;
use karna::SceneView;
use karna::WindowBuilder;
use karna::assets::Font;
use karna::assets::Handle;
use karna::input::Keycode;
use karna::logging::Config;
use karna::math::Vector2;
use karna::math::Vector3;
use karna::render::Color;
use karna::render::Draw;
use karna::render::Layer;

struct TextDemo {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    jbmono: Handle<Font>,
}

impl Scene for TextDemo {
    fn load(&mut self, ctx: &mut Context, scene: &mut SceneView) {
        ctx.assets.write_scope(|w| {
            self.jbmono = w.load_font(include_bytes!("assets/jbmono.ttf"), 16.0);
        });
    }

    fn update(&mut self, ctx: &mut Context, scene: &mut SceneView) {
        let dt = ctx.time.delta();
        let max_speed = 400.0;
        let responsiveness = 10.0;

        let mut dx = 0.0;
        let mut dy = 0.0;

        if ctx.input.key_held(Keycode::W) {
            dy -= 1.0;
        }

        if ctx.input.key_held(Keycode::S) {
            dy += 1.0;
        }

        if ctx.input.key_held(Keycode::A) {
            dx -= 1.0;
        }

        if ctx.input.key_held(Keycode::D) {
            dx += 1.0;
        }

        let len = ((dx * dx + dy * dy) as f32).sqrt();

        if len > 0.0 {
            dx /= len;
            dy /= len;
        }

        let target_x = dx * max_speed;
        let target_y = dy * max_speed;
        let t = 1.0 - (-responsiveness * dt).exp();

        self.vel.x += (target_x - self.vel.x) * t;
        self.vel.y += (target_y - self.vel.y) * t;
        self.pos += self.vel * dt;

        let follow_speed = 5.0;

        let rect_w = 50.0;
        let rect_h = 50.0;
        let rect_center = self.pos + Vector2::new(rect_w * 0.5, rect_h * 0.5);

        let view = ctx.window.size().as_f32();
        let camera = scene.camera_mut();

        let target = rect_center - Vector2::new(view.width * 0.5, view.height * 0.5);
        let target = Vector3::new(
            rect_center.x - view.width * 0.5,
            rect_center.y - view.height * 0.5,
            camera.position.z,
        );

        let t = 1.0 - (-follow_speed * dt).exp();

        camera.position = camera.position.lerp(&target, t);
    }

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {
        draw.set_color(Color::Green);

        for i in 0..20 {
            let n = i as f32;

            draw.rect(n * 200.0, 0.0, 20.0, 20.0);
            draw.rect(0.0, n * 200.0, 20.0, 20.0);
        }

        draw.set_color(Color::Blue);
        draw.rect(400.0, 400.0, 80.0, 80.0);
        draw.rect(900.0, 300.0, 80.0, 80.0);
        draw.rect(200.0, 700.0, 80.0, 80.0);
        draw.rect(1100.0, 900.0, 80.0, 80.0);

        draw.set_color(Color::Yellow);
        draw.rect(0.0, 0.0, 30.0, 30.0);

        draw.set_color(Color::Red);
        draw.rect(self.pos.x, self.pos.y, 50.0, 50.0);

        draw.set_layer(Layer::Ui);
        draw.set_color(Color::White);
        draw.debug_text("Some debug text", 10.0, 10.0);
        draw.debug_text(format!("dt {:.7}", ctx.time.delta()), 10.0, 30.0);
        draw.debug_text(format!("fps {} (100 samples)", ctx.time.fps()), 10.0, 50.0);

        draw.set_color(Color::Cyan);
        draw.debug_text("tinted text\nwith two lines", 100.0, 160.0);

        draw.set_color(Color::White);

        draw.text(
            self.jbmono,
            "This is another font!\nSome symbols. !@#$%^&*()[]{}",
            300.0,
            10.0,
        );

        draw.set_color(Color::Magenta);
        draw.debug_text(
            "This is the same font imgui uses I think?\nàèìòù",
            720.0,
            300.0,
        );
    }
}

struct Atlas {
    active_page: usize,
}

impl Scene for Atlas {
    fn load(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn update(&mut self, ctx: &mut Context, scene: &mut SceneView) {}

    fn draw(&mut self, ctx: &mut Context, draw: &mut Draw) {
        let handles = ctx
            .assets
            .atlas_page_handles()
            .enumerate()
            .collect::<Vec<_>>();

        for (i, handle) in handles {
            if i == self.active_page {
                draw.image(handle, 0.0, 0.0);
            }
        }
    }
}

fn main() {
    karna::init_logging(Config::default().with_min_level(karna::logging::LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("text rendering")
                .with_size((1280, 720))
                .with_resizable(true)
                .with_scene(
                    0,
                    TextDemo {
                        pos: Vector2::new(10.0, 10.0),
                        vel: Vector2::zero(),
                        jbmono: Handle::INVALID,
                    },
                )
                .with_active_scene(0),
        )
        .with_window(
            WindowBuilder::new()
                .with_title("texture atlas")
                .with_size((1024, 1024))
                .with_scene(0, Atlas { active_page: 0 })
                .with_active_scene(0),
        )
        .build()
        .run();
}
