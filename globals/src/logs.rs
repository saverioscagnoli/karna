use std::sync::{Arc, LazyLock, RwLock};

static LOGS: LazyLock<Arc<RwLock<Vec<String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));

#[inline]
pub fn get() -> &'static Arc<RwLock<Vec<String>>> {
    &LOGS
}
