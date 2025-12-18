pub mod input;

mod monitors;
mod time;
mod window;

use crate::context::input::Input;
use assets::AssetManager;
use renderer::Renderer;
use std::sync::Arc;
use winit::{event::WindowEvent, keyboard::PhysicalKey};

// Re-exports
pub use crate::context::time::Time;
pub use monitors::{Monitor, Monitors};
pub use window::Window;
pub(crate) use window::WinitWindow;

pub struct Context {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub render: Renderer,
    pub monitors: Monitors,
    pub assets: Arc<AssetManager>,
}

impl Context {
    pub(crate) fn new(window: Window, assets: Arc<AssetManager>) -> Self {
        let render = Renderer::new(Arc::clone(window.inner()), Arc::clone(&assets));
        let monitors = Monitors::new(Arc::clone(window.inner()));

        Self {
            window,
            time: Time::default(),
            input: Input::default(),
            render,
            monitors,
            assets,
        }
    }

    pub(crate) fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.render.resize(size.width, size.height);
            }

            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(code) => {
                    if event.state.is_pressed() {
                        if !event.repeat {
                            self.input.pressed_keys.insert(code);
                        }

                        self.input.held_keys.insert(code);
                    } else {
                        self.input.held_keys.remove(&code);
                    }
                }

                PhysicalKey::Unidentified(_) => {}
            },

            WindowEvent::CursorMoved { position, .. } => {
                self.input.mouse_position.x = position.x as f32;
                self.input.mouse_position.y = position.y as f32;
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() {
                    if !self.input.pressed_mouse.contains(&button) {
                        self.input.pressed_mouse.insert(button);
                    }

                    self.input.held_mouse.insert(button);
                } else {
                    self.input.held_mouse.remove(&button);
                }
            }

            _ => {}
        }
    }
}
