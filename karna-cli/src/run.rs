use std::path::PathBuf;

use karna::App;
use karna::SceneId;
use karna::WindowBuilder;
use karna::WindowBuilderExt;
use nostd::log::SdlTarget;

const MAIN: SceneId = SceneId::new_str("main");

pub fn exec(path: &PathBuf) -> Result<(), String> {
    _ = traccia::init(
        traccia::Config::default()
            .with_min_level(traccia::LevelFilter::Info)
            .with_target(SdlTarget::default()),
    );

    let root = path.to_str().ok_or_else(|| "path must be valid UTF-8")?;

    App::builder()
        .with_root(root)
        .with_window(
            WindowBuilder::default()
                .with_size((1280, 720))
                .with_js_entry(root, MAIN, "main.js")
                .with_active_scene(MAIN),
        )
        .build()
        .run();

    Ok(())
}
