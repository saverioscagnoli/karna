use karna::prelude::*;

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    let arena = SceneId::new_label("lua");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("arena")
                .with_size(math::Size::new(960, 640))
                .with_lua_scene(arena, "examples/lua/main.lua")
                .with_active_scene(arena),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
