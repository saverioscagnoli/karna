use logging::Config;

pub fn init_logging(conf: Config) {
    logging::init(
        conf.with_module_filter("sctk", logging::LevelFilter::Error)
            .with_module_filter("naga", logging::LevelFilter::Error)
            .with_module_filter("wgpu", logging::LevelFilter::Error),
    )
    .expect("Failed to set logger")
}
