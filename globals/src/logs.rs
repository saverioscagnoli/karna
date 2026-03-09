use std::sync::{Arc, LazyLock};

use logging::LogLevel;
use parking_lot::RwLock;

static LOGS: LazyLock<Arc<RwLock<Vec<(LogLevel, String)>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));

#[inline]
pub fn get() -> &'static Arc<RwLock<Vec<(LogLevel, String)>>> {
    &LOGS
}
