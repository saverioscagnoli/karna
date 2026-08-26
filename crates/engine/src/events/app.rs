//! Events the engine raises for itself.
//!
//! Scenes hold borrowed handles, not the window, so a request to change the
//! title or resize cannot be applied where it is made. It is dispatched
//! through [`super::queue`] and drained by the main loop, which does own the
//! window.

pub struct AppEvent {}
