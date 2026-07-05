use std::sync::mpsc::Receiver;

use assets::AssetServer;
use logging::info;
use renderer::Renderer;
use winit::event::DeviceEvent;
use winit::event::MouseButton;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::keyboard::PhysicalKey;

use crate::AppEvent;
use crate::context::WindowContext;
use crate::monitors::Monitor;
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
        events: Receiver<AppEvent>,
        window: Window,
        assets: AssetServer,
        renderer: Renderer,
        scenes: Scenes,
        active_scenes: Vec<String>,
        monitors: Vec<Monitor>,
    ) -> Self {
        let mut state = Self {
            context: WindowContext::new(window, assets, renderer, monitors),
            events,
            scenes,
            active_scenes: Vec::new(),
        };

        for label in active_scenes {
            if let Some(scene) = state.scenes.get_mut(&label) {
                scene.load(state.context.as_ref_mut());
                info!("Scene '{}' loaded", label);
            }

            state.active_scenes.push(label);
        }

        state
    }

    pub fn start_loop(mut self) {
        let mut should_close = false;

        while !should_close {
            for event in self.events.try_iter() {
                match event {
                    AppEvent::WindowEvent(WindowEvent::CloseRequested) => {
                        should_close = true;
                        break;
                    }

                    AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
                        info!("Setting window size to {}x{}", size.width, size.height);
                        self.context.renderer._resize(size.width, size.height);
                    }

                    AppEvent::QueryMonitors(monitors) => {
                        self.context.monitors.monitors = monitors;
                    }

                    AppEvent::WindowEvent(event) => match event {
                        WindowEvent::KeyboardInput {
                            event: key_event, ..
                        } => match key_event.physical_key {
                            PhysicalKey::Code(c) => {
                                if key_event.state.is_pressed() {
                                    if !key_event.repeat {
                                        self.context.input.pressed_keys.insert(c);
                                    }

                                    self.context.input.held_keys.insert(c);
                                } else {
                                    self.context.input.held_keys.remove(&c);
                                    self.context.input.released_keys.insert(c);
                                }
                            }

                            _ => {}
                        },

                        WindowEvent::CursorMoved { position, .. } => {
                            self.context
                                .input
                                .mouse_position
                                .set([position.x as f32, position.y as f32]);
                        }

                        WindowEvent::MouseInput { button, state, .. } => {
                            if state.is_pressed() {
                                self.context.input.pressed_mouse_buttons.insert(button);
                                self.context.input.held_mouse_buttons.insert(button);
                            } else {
                                self.context.input.held_mouse_buttons.remove(&button);
                            }
                        }

                        WindowEvent::MouseWheel { delta, .. } => {
                            self.context.input.wheel_delta = match delta {
                                MouseScrollDelta::LineDelta(_x, y) => y,
                                MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                            }
                        }

                        _ => {}
                    },

                    AppEvent::DeviceEvent(event) => match event.as_ref() {
                        DeviceEvent::MouseMotion { delta } => {
                            self.context
                                .input
                                .mouse_delta
                                .set([delta.0 as f32, delta.1 as f32]);
                        }

                        _ => {}
                    },
                }
            }

            for label in &self.active_scenes {
                if let Some(scene) = self.scenes.get_mut(label) {
                    scene.update(self.context.as_ref_mut());
                }
            }

            while let Some(tick_start) = self.context.time.next_tick() {
                self.context.time.do_tick(tick_start);

                for label in &self.active_scenes {
                    if let Some(scene) = self.scenes.get_mut(label) {
                        scene.fixed_update(self.context.as_ref_mut());
                    }
                }
            }

            for label in &self.active_scenes {
                let (ctx, mut draw) = self.context.split();

                if let Some(scene) = self.scenes.get_mut(label) {
                    scene.draw(ctx, &mut draw);
                }
            }

            let m = self.context.input.mouse_position();
            let io = self.context.renderer.imgui().io_mut();

            io.delta_time = self.context.time.delta();
            io.display_size = [
                self.context.window.width() as f32,
                self.context.window.height() as f32,
            ];
            io.mouse_pos = [m.x, m.y];

            io.mouse_down = [
                self.context.input.mouse_held(&MouseButton::Left),
                self.context.input.mouse_held(&MouseButton::Right),
                self.context.input.mouse_held(&MouseButton::Middle),
                false,
                false,
            ];

            let ui = self.context.renderer.imgui().new_frame();

            ui.window("Debug")
                .size([300.0, 200.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text("Hello from imgui!");
                    if ui.button("Click me") {
                        println!("clicked");
                    }
                });

            ui.show_demo_window(&mut true);

            self.context
                .renderer
                ._present(&self.context.assets._guard());

            let to_activate: Vec<_> = self.context.scenes.pending_activate.drain(..).collect();
            let to_deactivate: Vec<_> = self.context.scenes.pending_deactivate.drain(..).collect();

            for label in to_activate {
                if !self.active_scenes.contains(&label) {
                    if let Some(scene) = self.scenes.get_mut(&label) {
                        scene.load(self.context.as_ref_mut());
                        info!("Scene '{}' loaded", label);
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
