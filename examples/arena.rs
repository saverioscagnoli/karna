//! Runs `examples/arena.lua` — the same `Scene` shape as `demo.rs`, written in
//! Lua instead of Rust.

use karna::prelude::*;

fn main() {
    let _ = init_logging(LogConfig::default().with_min_level(LevelFilter::Debug));

    let arena = SceneId::new_label("arena");

    App::builder()
        .with_window(
            WindowBuilder::new()
                .with_title("arena")
                .with_size(math::Size::new(960u32, 640))
                .with_lua_scene(arena, "examples/arena.lua")
                .with_active_scene(arena),
        )
        .with_asset_root("examples/")
        .build()
        .run();
}
