use karna::App;
use karna::ContextRef;
use karna::ContextRefMut;
use karna::Handle;
use karna::Scene;
use karna::WindowBuilder;
use karna::assets::Font;
use karna::logging;
use karna::render::Draw;
use log::LevelFilter;
use renderer::Color;

struct S {
    jbmono: Handle<Font>,
}

impl Scene for S {
    fn load(&mut self, ctx: ContextRefMut) {
        self.jbmono = ctx
            .assets
            .load_font(include_bytes!("assets/jbmono.ttf"), 18);
    }

    fn update(&mut self, ctx: ContextRefMut) {
        let _ = ctx;
    }

    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw) {
        let _ = ctx;

        draw.debug_text(
            "Hello world! This is debug text!\nAnd this is a fantastic new line\nThis is a \t tabulation?",
            10.0,
            10.0,
        );

        draw.text(
            self.jbmono,
            "This is a custom font.\nUse ctx.assets.load_font to do it!",
            200.0,
            200.0,
        );

        draw.set_color(Color::Cyan);
        draw.text(self.jbmono, "Endearing cyan font...", 200.0, 400.0);
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
                        jbmono: Handle::default(),
                    },
                )
                .with_active_scene("initial"),
        )
        .build()
        .run();
}
