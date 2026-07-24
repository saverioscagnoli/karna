use karna::App;
use karna::logging::Config;
use karna::logging::LevelFilter;

fn main() {
    karna::init_logging(Config::default().with_min_level(LevelFilter::Debug));
    App::new().run();
}
