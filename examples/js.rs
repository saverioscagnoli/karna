use karna::prelude::*;

const DEMO: SceneId = SceneId::new_str("js-demo");

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("karna - javascript")
                .with_size(Size::new(960, 640))
                .with_js_scene(DEMO, "examples/js/main.js")
                .with_active_scene(DEMO),
        )
        .with_root("examples/")
        .build()
        .run();
}
