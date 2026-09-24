use karna::prelude::*;

const MAIN: SceneId = SceneId::new_str("main");

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
                .with_title("script demo")
                .with_size((1280, 720))
                .with_js_script(MAIN, "assets/scripts/main.js")
                .with_active_scene(MAIN),
        )
        .build()
        .run();
}
