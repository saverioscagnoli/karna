use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static CONTEXT: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

pub fn insert(key: impl Into<String>, value: impl Into<String>) {
    CONTEXT.with(|ctx| ctx.borrow_mut().insert(key.into(), value.into()));
}

pub fn remove(key: &str) -> Option<String> {
    CONTEXT.with(|ctx| ctx.borrow_mut().remove(key))
}

pub fn snapshot() -> HashMap<String, String> {
    CONTEXT.with(|ctx| ctx.borrow().clone())
}

pub fn clear() {
    CONTEXT.with(|ctx| ctx.borrow_mut().clear());
}

/// RAII guard that removes the key on drop.
pub struct ContextGuard {
    key: String,
}

impl ContextGuard {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        insert(key.clone(), value);
        Self { key }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        remove(&self.key);
    }
}
