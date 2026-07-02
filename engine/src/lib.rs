mod context;
pub mod input;
mod scene;
mod time;
mod window;

use std::ffi;
use std::ffi::CString;

use sokol::app as sapp;
use sokol::gfx as sg;
use sokol::glue as sglue;
use utils::FastHashMap;

use crate::context::Context;
pub use crate::context::ContextRef;
pub use crate::context::ContextRefMut;
pub use crate::scene::Scene;
pub use crate::scene::Scenes;
pub use crate::window::Window;

/// Live handle to the OS window. Cached fields are refreshed once per
/// frame from sokol_app; setters call straight through to sokol_app so
/// changes (e.g. title) take effect immediately.

/// Per-frame handle passed into every Scene callback. Add whatever else
/// you need here (input state, elapsed time, asset caches, etc.).
// Context

/// Fluent config for the initial window/app parameters.
pub struct WindowBuilder {
    title: String,
    size: math::Size<u32>,
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_size<S: Into<math::Size<u32>>>(mut self, size: S) -> Self {
        let size: math::Size<u32> = size.into();

        self.size = size;
        self
    }

    pub fn build(self) -> Window {
        Window::new(self.title, self.size)
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: "incredible window".to_owned(),
            size: math::Size::new(1280, 720),
        }
    }
}

pub struct AppBuilder {
    window: WindowBuilder,
    scenes: Scenes,
}

impl AppBuilder {
    pub fn with_window(mut self, window: WindowBuilder) -> Self {
        self.window = window;
        self
    }

    pub fn with_scene<S: Scene + 'static>(mut self, label: String, scene: S) -> Self {
        self.scenes.insert(label, Box::new(scene));
        self
    }

    pub fn build(self) -> App {
        App {
            scenes: self.scenes,
            queued_window: self.window,
        }
    }
}

pub struct App {
    queued_window: WindowBuilder,
    scenes: Scenes,
}

/// Everything the C callbacks need, boxed once and passed through as
/// sokol_app's void* user_data.

struct AppState {
    scenes: Scenes,
    ctx: Context,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder {
            scenes: FastHashMap::default(),
            window: WindowBuilder::new(),
        }
    }

    pub fn run(self) {
        let window = Window::new(self.queued_window.title, self.queued_window.size);

        let state = AppState {
            ctx: Context::new(window),
            scenes: self.scenes,
        };

        let title = state.ctx.window.title().to_string();
        let win_size = state.ctx.window.size();

        let user_data = Box::into_raw(Box::new(state)) as *mut ffi::c_void;

        extern "C" fn init_cb(user_data: *mut ffi::c_void) {
            let state = unsafe { &mut *(user_data as *mut AppState) };

            sg::setup(&sg::Desc {
                environment: sglue::environment(),
                logger: sg::Logger {
                    func: Some(sokol::log::slog_func),
                    ..Default::default()
                },
                ..Default::default()
            });

            state.ctx.init_gpu(); // build Renderer AFTER setup

            state
                .scenes
                .get_mut("initial")
                .unwrap()
                .load(state.ctx.as_mut());
        }

        extern "C" fn frame_cb(user_data: *mut ffi::c_void) {
            let state = unsafe { &mut *(user_data as *mut AppState) };
            let view = state.ctx.window.size();

            while let Some(tick_start) = state.ctx.time.next_tick() {
                if let Some(scene) = state.scenes.get_mut("initial") {
                    scene.fixed_update(state.ctx.as_mut());
                }

                state.ctx.time.do_tick(tick_start);
            }

            if let Some(scene) = state.scenes.get_mut("initial") {
                scene.update(state.ctx.as_mut());

                let (ctx, mut draw) = state.ctx.split();

                scene.draw(ctx, &mut draw);
            }

            state.ctx.render_mut().present(view);
            state.ctx.input.flush();
            state.ctx.time.wait_for_next_frame();
        }

        extern "C" fn cleanup_cb(user_data: *mut ffi::c_void) {
            let state = unsafe { &mut *(user_data as *mut AppState) };

            for scene in state.scenes.values_mut() {
                scene.cleanup(state.ctx.as_mut());
            }

            sg::shutdown();
        }

        extern "C" fn event_cb(event: *const sapp::Event, user_data: *mut ffi::c_void) {
            let state = unsafe { &mut *(user_data as *mut AppState) };
            let event = unsafe { &*event };

            match event._type {
                sapp::EventType::Resized => {
                    let new_size = math::Size::new(sapp::width() as u32, sapp::height() as u32);
                    state.ctx.window.size = new_size;
                    state.ctx.render_mut().resize(new_size);
                }
                sapp::EventType::KeyDown => {
                    let k = event.key_code;

                    if !event.key_repeat {
                        state.ctx.input.pressed_keys.insert(k);
                    }

                    state.ctx.input.held_keys.insert(k);
                }

                sapp::EventType::KeyUp => {
                    let k = event.key_code;

                    state.ctx.input.held_keys.remove(&k);
                    state.ctx.input.released_keys.insert(k);
                }

                sapp::EventType::MouseMove => {
                    state.ctx.input.mouse_position.x = event.mouse_x;
                    state.ctx.input.mouse_position.y = event.mouse_y;
                    state.ctx.input.mouse_delta.x = event.mouse_dx;
                    state.ctx.input.mouse_delta.y = event.mouse_dy;
                }

                sapp::EventType::MouseDown => {
                    state
                        .ctx
                        .input
                        .pressed_mouse_buttons
                        .insert(event.mouse_button);

                    state
                        .ctx
                        .input
                        .held_mouse_buttons
                        .insert(event.mouse_button);
                }

                sapp::EventType::MouseUp => {
                    state
                        .ctx
                        .input
                        .held_mouse_buttons
                        .remove(&event.mouse_button);
                }
                _ => {}
            }
        }

        let title = CString::new(title).unwrap();

        sapp::run(&sapp::Desc {
            init_userdata_cb: Some(init_cb),
            frame_userdata_cb: Some(frame_cb),
            cleanup_userdata_cb: Some(cleanup_cb),
            event_userdata_cb: Some(event_cb),
            user_data,
            window_title: title.as_ptr(),
            width: win_size.width as i32,
            height: win_size.height as i32,
            logger: sapp::Logger {
                func: Some(sokol::log::slog_func),
                ..Default::default()
            },
            icon: sapp::IconDesc {
                sokol_default: true,
                ..Default::default()
            },
            swap_interval: 0,
            ..Default::default()
        });
    }
}
