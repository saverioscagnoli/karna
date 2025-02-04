mod error;

pub use error::*;

pub fn date() -> String {
    let now = chrono::Local::now();
    now.format("%d/%m/%Y %H:%M:%S").to_string()
}

// These macros are a simple way to log messages to the standard output and error.
// They are more of a pretty print than a real logger.

/// Logs an info message to the standard output.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("[\x1b[32mINFO\x1b[0m] [\x1b[36m{}\x1b[0m] {}", $crate::date(), format_args!($($arg)*));
    };
}

/// Logs a warning message to the standard error.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("[\x1b[33mWARN\x1b[0m] [\x1b[36m{}\x1b[0m] {}", $crate::date(), format_args!($($arg)*));
    };
}

/// Logs an error message to the standard error.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[\x1b[31mERROR\x1b[0m] [\x1b[36m{}\x1b[0m] {}", $crate::date(), format_args!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run with `cargo test -- --nocapture` to see the output
    #[test]
    fn test_log() {
        info!("Very informative message");
        info!("This is a number: {}", 42);
        warn!("This is your last warning!");
        error!("Something went wrong!");
    }
}
