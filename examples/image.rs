#![allow(unused)]

use karna::App;
use karna::ContextMut;
use karna::Handle;
use karna::Scene;
use karna::SceneHandle;
use karna::WindowBuilder;
use karna::assets::Image;
use karna::input::Keycode;
use karna::math::Size;
use karna::render::Draw;
use math::Vector2;

struct S {
    pos: Vector2<f32>,
    image: Handle<Image>,
    duck: Handle<Image>,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        ctx.assets.write_scope(|a| {
            let bytes = include_bytes!("assets/tetsuo.png");
            self.image = a.load_image(bytes);

            let duck_bytes = include_bytes!("assets/duck.png");
            self.duck = a.load_image(duck_bytes);
        });

        ctx.window.set_icon(self.duck);
        ctx.window.set_custom_cursor(self.duck, [0, 0]);
    }

    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {
        if ctx.input.key_held(Keycode::KeyW) {
            self.pos.y -= 10.0;
        }

        if ctx.input.key_held(Keycode::KeyA) {
            self.pos.x -= 10.0;
        }

        if ctx.input.key_held(Keycode::KeyS) {
            self.pos.y += 10.0;
        }

        if ctx.input.key_held(Keycode::KeyD) {
            self.pos.x += 10.0;
        }
    }

    fn draw(&mut self, _ctx: karna::ContextRef, draw: &mut Draw) {
        draw.image_v(self.image, self.pos);
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
                    "image",
                    S {
                        pos: Vector2::zero(),
                        image: Handle::default(),
                        duck: Handle::default(),
                    },
                )
                .with_active_scene("image"),
        )
        .build()
        .run();
}
