use std::sync::mpsc::Receiver;

use renderer::Renderer;
use winit::event::WindowEvent;

use crate::AppEvent;
use crate::context::WindowContext;
use crate::scene::Scenes;
use crate::window::Window;

pub struct WindowState {
    context: WindowContext,
    events: Receiver<AppEvent>,
    scenes: Scenes,
    active_scenes: Vec<String>,
}

impl WindowState {
    pub fn new(
        window: Window,
        renderer: Renderer,
        scenes: Scenes,
        active_scenes: Vec<String>,
        events: Receiver<AppEvent>,
    ) -> Self {
        let mut state = Self {
            context: WindowContext::new(window, renderer),
            events,
            scenes,
            active_scenes: Vec::new(),
        };

        for label in active_scenes {
            if let Some(scene) = state.scenes.get_mut(&label) {
                scene.load(state.context.as_ref_mut());
            }

            state.active_scenes.push(label);
        }

        state
    }

    pub fn start_loop(mut self) {
        let mut should_close = false;

        while !should_close {
            if let Ok(first) = self.events.recv() {
                for event in std::iter::once(first).chain(self.events.try_iter()) {
                    match event {
                        AppEvent::WindowEvent(WindowEvent::CloseRequested) => {
                            should_close = true;
                            break;
                        }

                        AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
                            self.context.renderer.resize(size.width, size.height);
                        }
                        AppEvent::WindowEvent(_) => {}
                    }
                }
            }

            for label in &self.active_scenes {
                if let Some(scene) = self.scenes.get_mut(label) {
                    scene.update(self.context.as_ref_mut());
                }
            }

            for label in &self.active_scenes {
                if let Some(scene) = self.scenes.get(label) {
                    scene.draw(self.context.as_ref());
                }
            }

            self.context.renderer.present();

            let to_activate: Vec<_> = self.context.scenes.pending_activate.drain(..).collect();
            let to_deactivate: Vec<_> = self.context.scenes.pending_deactivate.drain(..).collect();

            for label in to_activate {
                if !self.active_scenes.contains(&label) {
                    if let Some(scene) = self.scenes.get_mut(&label) {
                        scene.load(self.context.as_ref_mut());
                    }
                    self.active_scenes.push(label);
                }
            }

            for label in to_deactivate {
                self.active_scenes.retain(|s| s != &label);
            }

            self.context.window.request_redraw();
            self.context.time.wait_for_next_frame();
        }
    }
}
