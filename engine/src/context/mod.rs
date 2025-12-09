mod audio;
mod monitors;
mod time;
mod window;

// Export input as a separate module
pub mod input;

use crate::context::{audio::Audio, input::Input};
use renderer::{GPU, Renderer};
use std::sync::Arc;
use winit::{event::WindowEvent, keyboard::PhysicalKey};

// Re-exports
pub use monitors::{Monitor, Monitors};
pub use time::Time;
pub use window::Window;

pub struct Context {
    pub gpu: Arc<GPU>,
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub render: Renderer,
    pub audio: Audio,
    pub monitors: Monitors,
}

impl Context {
    pub fn new(gpu: Arc<GPU>, window: Window, recommended_fps: u32) -> Self {
        let render = Renderer::new(Arc::clone(&gpu), Arc::clone(window.inner()));

        render.init();

        let monitors = Monitors::new(Arc::clone(window.inner()));

        Self {
            gpu,
            window,
            time: Time::new(recommended_fps),
            input: Input::new(),
            render,
            audio: Audio::new(),
            monitors,
        }
    }

    #[inline]
    pub(crate) fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.render.resize(size.into());
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
