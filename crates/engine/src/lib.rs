//! The karna engine: windowing, events, input, rendering and assets.
//!
//! See ARCHITECTURE.md at the repository root for how the crates fit together
//! and which boundaries are enforced by tests.

pub mod events;
mod scene;
mod window;

pub use crate::events::Finger;
pub use crate::events::GamepadEvent;
pub use crate::events::KeyEvent;
pub use crate::events::Keycode;
pub use crate::events::Lifecycle;
pub use crate::events::Modifiers;
pub use crate::events::MouseButton;
pub use crate::events::MouseEvent;
pub use crate::events::SDLEvent;
pub use crate::events::SDLWindowEvent;
pub use crate::events::Scancode;
pub use crate::events::TouchEvent;
pub use crate::events::WindowId;
