use karna::App;
use logging::{Config, LevelFilter};

fn main() {
    karna::logging::init(Config::default().with_min_level(LevelFilter::Debug)).unwrap();

    App::new().run();
}
