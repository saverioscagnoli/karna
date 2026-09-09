#![no_std]

use karna::App;
use karna::log::SdlTarget;
use traccia::info;

fn main() {
    _ = traccia::init(
        traccia::Config::default()
            .with_min_level(traccia::LevelFilter::Debug)
            .with_target(SdlTarget::default()),
    );

    App::new().run();

    info!("bye");
}
