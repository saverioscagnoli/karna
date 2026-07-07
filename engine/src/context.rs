use assets::AssetServer;
use assets::AssetServerGuard;
use imgui::ActiveImgui;
use imgui::SharedImgui;
use logging::info;
use renderer::Draw;
use renderer::Renderer;
use winit::event::DeviceEvent;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::keyboard::PhysicalKey;

use crate::AppEvent;
use crate::input::Input;
use crate::monitors::Monitor;
use crate::monitors::Monitors;
use crate::scene::SceneManager;
use crate::storage::GenericStorage;
use crate::time::Time;
use crate::window::Window;

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub storage: GenericStorage,
    pub assets: AssetServer,
    pub scenes: SceneManager,
    pub renderer: Renderer,
    pub imgui: SharedImgui,
    pub monitors: Monitors,
    pub show_console: bool,
}

impl WindowContext {
    pub fn new(
        window: Window,
        assets: AssetServer,
        renderer: Renderer,
        monitors: Vec<Monitor>,
        imgui: SharedImgui,
    ) -> Self {
        // Arc::clone
        let winit_window = window.inner.clone();

        Self {
            window,
            time: Time::new(),
            input: Input::new(),
            storage: GenericStorage::new(),
            assets,
            scenes: SceneManager::new(),
            renderer,
            monitors: Monitors::new(winit_window, monitors),
            imgui,
            show_console: false,
        }
    }

    pub fn as_ref_mut<'ctx>(&'ctx mut self) -> ContextRefMut<'ctx> {
        ContextRefMut {
            window: &self.window,
            time: &mut self.time,
            input: &mut self.input,
            storage: &mut self.storage,
            assets: &self.assets,
            scenes: &mut self.scenes,
            render: &mut self.renderer,
            monitors: &self.monitors,
        }
    }

    pub fn split<'ctx>(&'ctx mut self) -> (ContextRef<'ctx>, Draw<'ctx>) {
        let ctx = ContextRef {
            window: &self.window,
            time: &self.time,
            input: &self.input,
            storage: &self.storage,
            assets: self.assets._guard(),
            scenes: &self.scenes,
            monitors: &self.monitors,
        };

        let imgui = ActiveImgui::new(&self.imgui, self.window.id());
        let draw = Draw::_new(&mut self.renderer, self.assets._guard(), imgui);

        (ctx, draw)
    }

    pub fn update_imgui_time(&mut self) {
        let mut imgui = ActiveImgui::new(&self.imgui, self.window.id());
        let io = imgui.io_mut();

        io.delta_time = self.time.delta();
    }

    pub fn handle_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
                info!("Setting window size to {}x{}", size.width, size.height);
                self.renderer._resize(size.width, size.height);
            }

            AppEvent::QueryMonitors(monitors) => {
                self.monitors.monitors = monitors.clone();
            }

            AppEvent::WindowEvent(event) => match event {
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => match key_event.physical_key {
                    PhysicalKey::Code(c) => {
                        if key_event.state.is_pressed() {
                            if !key_event.repeat {
                                self.input.pressed_keys.insert(c);
                            }

                            self.input.held_keys.insert(c);
                        } else {
                            self.input.held_keys.remove(&c);
                            self.input.released_keys.insert(c);
                        }
                    }

                    _ => {}
                },

                WindowEvent::CursorMoved { position, .. } => {
                    self.input
                        .mouse_position
                        .set([position.x as f32, position.y as f32]);
                }

                WindowEvent::MouseInput { button, state, .. } => {
                    if state.is_pressed() {
                        self.input.pressed_mouse_buttons.insert(*button);
                        self.input.held_mouse_buttons.insert(*button);
                    } else {
                        self.input.held_mouse_buttons.remove(button);
                    }
                }

                WindowEvent::MouseWheel { delta, .. } => match *delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.input.wheel_delta.set([x, y]);
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        self.input.wheel_delta.set([pos.x as f32, pos.y as f32]);
                    }
                },

                _ => {}
            },

            AppEvent::DeviceEvent(event) => match event.as_ref() {
                DeviceEvent::MouseMotion { delta } => {
                    self.input.mouse_delta.set([delta.0 as f32, delta.1 as f32]);
                }

                _ => {}
            },
        }
    }

    pub fn handle_event_for_imgui(&mut self, event: &AppEvent) {
        let mut imgui = ActiveImgui::new(&self.imgui, self.window.id());
        let io = imgui.io_mut();

        match event {
            AppEvent::WindowEvent(WindowEvent::Resized(size)) => {
                io.display_size = [size.width as f32, size.height as f32];
            }

            AppEvent::WindowEvent(event) => match event {
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => match key_event.physical_key {
                    PhysicalKey::Code(c) => {
                        if key_event.state.is_pressed()
                            && let Some(text) = key_event.text.as_deref()
                        {
                            for ch in text.chars() {
                                io.add_input_character(ch);
                            }
                        }

                        if let Some(ik) = imgui::winit_keycode_to_imgui(c) {
                            io.add_key_event(ik, key_event.state.is_pressed());
                        }
                    }

                    _ => {}
                },

                WindowEvent::ModifiersChanged(m) => {
                    let state = m.state();

                    io.add_key_event(imgui::Key::ModShift, state.shift_key());
                    io.add_key_event(imgui::Key::ModCtrl, state.control_key());
                    io.add_key_event(imgui::Key::ModAlt, state.alt_key());
                    io.add_key_event(imgui::Key::ModSuper, state.super_key());
                }

                WindowEvent::CursorMoved { position, .. } => {
                    io.mouse_pos = [position.x as f32, position.y as f32];
                }

                WindowEvent::MouseInput { button, state, .. } => {
                    if let Some(mb) = imgui::winit_mousebutton_to_imgui(*button) {
                        io.add_mouse_button_event(mb, state.is_pressed());
                    }
                }

                WindowEvent::MouseWheel { delta, .. } => match *delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        io.mouse_wheel = y;
                        io.mouse_wheel_h = x;
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        let pos = [pos.x as f32, pos.y as f32];

                        io.mouse_wheel = pos[0];
                        io.mouse_wheel_h = pos[1];
                    }
                },

                _ => {}
            },

            AppEvent::DeviceEvent(event) => match event.as_ref() {
                DeviceEvent::MouseMotion { delta } => {
                    io.mouse_delta = [delta.0 as f32, delta.1 as f32];
                }

                _ => {}
            },

            _ => {}
        }
    }
}

pub struct ContextRefMut<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx mut Time,
    pub input: &'ctx mut Input,
    pub storage: &'ctx mut GenericStorage,
    pub assets: &'ctx AssetServer,
    pub scenes: &'ctx mut SceneManager,
    pub render: &'ctx mut Renderer,
    pub monitors: &'ctx Monitors,
}

pub struct ContextRef<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx Time,
    pub input: &'ctx Input,
    pub storage: &'ctx GenericStorage,
    pub assets: AssetServerGuard<'ctx>,
    pub scenes: &'ctx SceneManager,
    pub monitors: &'ctx Monitors,
}
