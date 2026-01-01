use logging::{LogLevel, ctx, fatal, info, trace};

fn main() {
    logging::init_with_level(LogLevel::Trace);

    let _guard = ctx!("request_id", "abc123");
    info!("Starting");

    {
        let _guard = ctx!("user", "bob");
        info!("Processing"); // has both request_id and user
        fatal!("WOW!");
    }

    info!("Done"); // only request_id
    trace!("Wowie zowie")
}
