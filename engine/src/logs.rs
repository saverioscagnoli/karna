use logging::Config;

pub fn init_logging(conf: Config) {
    logging::init(conf).expect("Failed to set logger")
}
