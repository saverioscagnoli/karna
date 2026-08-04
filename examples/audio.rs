#![allow(unused)]

use std::alloc::handle_alloc_error;
use std::u32;

use karna::App;
use karna::ContextRef;
use karna::DrawContext;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Audio;
use karna::assets::Handle;
use karna::gpu::PresentMode;
use karna::input::Key;
use karna::logging::Config;
use karna::logging::LevelFilter;
use karna::render::Color;
use karna::render::Draw;
use karna::render::SceneRef;

struct AudioDemo {
    mammamia: Handle<Audio>,
    vol: f32,
}

impl Scene for AudioDemo {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized,
    {
        let mammamia = ctx.assets.load_audio("assets/mamma-mia.mp3");

        Self { mammamia, vol: 0.5 }
    }

    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {
        if ctx.input.key_pressed(Key::Space) {
            let audio = ctx.assets.get_audio(self.mammamia);
            ctx.audio.play(audio);
        }

        if ctx.input.key_pressed(Key::Up) {
            self.vol += 0.1;
            ctx.audio.set_master_volume(self.vol);
        }

        if ctx.input.key_pressed(Key::Down) {
            self.vol -= 0.1;
            ctx.audio.set_master_volume(self.vol);
        }
    }

    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw) {
        draw.debug_text(
            format!(
                "Press 'Space' to play the sound.\nArrowUp for increasing volume\nArrowDown for decreasing volume\nVolume: {:.1}",
                self.vol,
        ),
        10.0,
        10.0
        );
    }
}

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("Audio Demo")
                .with_size((1280, 720))
                .with_scene::<AudioDemo>(0)
                .with_active_scene(0),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
