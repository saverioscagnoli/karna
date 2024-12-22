#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("[\x1b[32mINFO\x1b[0m] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("[\x1b[33mWARN\x1b[0m] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        let loc = std::panic::Location::caller();
        println!("[\x1b[31mERROR\x1b[0m] {} at {}", format!($($arg)*), loc);
    };
}
