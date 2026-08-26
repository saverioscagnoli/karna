//! Events, from two sources.
//!
//! [`sdl`] holds events originating outside the process — input, window
//! changes, lifecycle — expressed in the engine's own vocabulary rather than
//! SDL's. [`poll`] is the translation layer and the only module allowed to
//! name an SDL type. [`app`] holds events the engine raises for itself, and
//! [`queue`] is the channel carrying them back to the main loop.
//!
//! The split matters because the two have opposite directions: SDL events flow
//! inward from the platform, app events flow outward from scenes to the loop
//! that owns the window.

pub mod app;
pub mod poll;
pub mod queue;
pub mod sdl;

pub use app::AppEvent;
pub use poll::poll;
pub use sdl::Finger;
pub use sdl::GamepadEvent;
pub use sdl::KeyEvent;
pub use sdl::Keycode;
pub use sdl::Lifecycle;
pub use sdl::Modifiers;
pub use sdl::MouseButton;
pub use sdl::MouseEvent;
pub use sdl::SDLEvent;
pub use sdl::SDLWindowEvent;
pub use sdl::Scancode;
pub use sdl::TouchEvent;
pub use sdl::WindowId;
