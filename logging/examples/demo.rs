use logging::{LogLevel, debug, error, fatal, info, trace, warn};

fn main() {
    logging::init_with_level(LogLevel::Trace);

    trace!("This is a trace message");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");
    fatal!("This is a fatal message");
}
