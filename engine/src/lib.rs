use std::ffi;
use std::ffi::CString;

use sokol::app as sapp;
use sokol::gfx as sg;
use sokol::glue as sglue;

/// Live handle to the OS window. Cached fields are refreshed once per
/// frame from sokol_app; setters call straight through to sokol_app so
/// changes (e.g. title) take effect immediately.
///
/// NOTE: verify the exact `sapp::` function names/signatures against the
/// version of the `sokol` crate you're using — sokol_app's C API is
/// `sapp_set_window_title`, `sapp_request_quit`, `sapp_toggle_fullscreen`,
/// `sapp_is_fullscreen`, etc, and the Rust bindings should mirror these in
/// snake_case, but exact naming can drift between crate versions.
pub struct Window {
    title: String,
    width: i32,
    height: i32,
}

impl Window {
    fn new(title: &str, width: i32, height: i32) -> Self {
        Self {
            title: title.to_string(),
            width,
            height,
        }
    }

    /// Pull current values from sokol_app. Called once per frame since the
    /// window can be resized by the OS/user at any time.
    fn sync(&mut self) {
        self.width = sapp::width();
        self.height = sapp::height();
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// Change the window title at runtime.
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        sapp::set_window_title(title);
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn is_fullscreen(&self) -> bool {
        sapp::is_fullscreen()
    }

    pub fn toggle_fullscreen(&self) {
        sapp::toggle_fullscreen();
    }

    pub fn request_quit(&self) {
        sapp::request_quit();
    }
}

/// Per-frame handle passed into every Scene callback. Add whatever else
/// you need here (input state, elapsed time, asset caches, etc.).
pub struct Context {
    pub window: Window,
}

impl Context {
    fn new(window: Window) -> Self {
        Self { window }
    }

    fn sync(&mut self) {
        self.window.sync();
    }
}

/// Implement this for your game/demo. Mirrors love2d's load/update/draw.
pub trait Scene {
    fn load(&mut self, ctx: &mut Context) {
        let _ = ctx;
    }
    fn update(&mut self, ctx: &mut Context, dt: f32) {
        let _ = (ctx, dt);
    }
    fn draw(&mut self, ctx: &mut Context) {
        let _ = ctx;
    }
    fn cleanup(&mut self, ctx: &mut Context) {
        let _ = ctx;
    }
}

/// Fluent config for the initial window/app parameters.
pub struct WindowBuilder {
    title: String,
    width: i32,
    height: i32,
    sample_count: i32,
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self {
            title: "app".to_string(),
            width: 800,
            height: 600,
            sample_count: 4,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_sample_count(mut self, sample_count: i32) -> Self {
        self.sample_count = sample_count;
        self
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Entry point: `App::builder().with_window(...).with_scene(...).run()`
pub struct AppBuilder {
    window: WindowBuilder,
    scene: Option<Box<dyn Scene>>,
}

impl AppBuilder {
    pub fn with_window(mut self, window: WindowBuilder) -> Self {
        self.window = window;
        self
    }

    pub fn with_scene<S: Scene + 'static>(mut self, scene: S) -> Self {
        self.scene = Some(Box::new(scene));
        self
    }

    pub fn run(self) {
        let scene = self
            .scene
            .expect("AppBuilder::run called without a scene (use .with_scene(...))");
        App {
            window: self.window,
            scene,
        }
        .run();
    }
}

pub struct App {
    window: WindowBuilder,
    scene: Box<dyn Scene>,
}

/// Everything the C callbacks need, boxed once and passed through as
/// sokol_app's void* user_data.
struct AppState {
    scene: Box<dyn Scene>,
    ctx: Context,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder {
            window: WindowBuilder::new(),
            scene: None,
        }
    }

    fn run(self) {
        let window = Window::new(&self.window.title, self.window.width, self.window.height);
        let state = AppState {
            scene: self.scene,
            ctx: Context::new(window),
        };
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
            state.ctx.sync();
            state.scene.load(&mut state.ctx);
        }

        extern "C" fn frame_cb(user_data: *mut ffi::c_void) {
            let state = unsafe { &mut *(user_data as *mut AppState) };
            state.ctx.sync();
            let dt = sapp::frame_duration() as f32;
            state.scene.update(&mut state.ctx, dt);
            state.scene.draw(&mut state.ctx);
        }

        extern "C" fn cleanup_cb(user_data: *mut ffi::c_void) {
            let mut state = unsafe { Box::from_raw(user_data as *mut AppState) };
            state.scene.cleanup(&mut state.ctx);
            sg::shutdown();
        }

        sapp::run(&sapp::Desc {
            init_userdata_cb: Some(init_cb),
            frame_userdata_cb: Some(frame_cb),
            cleanup_userdata_cb: Some(cleanup_cb),
            user_data,
            window_title: CString::new(self.window.title).unwrap().as_ptr(),
            width: self.window.width,
            height: self.window.height,
            sample_count: self.window.sample_count,
            logger: sapp::Logger {
                func: Some(sokol::log::slog_func),
                ..Default::default()
            },
            icon: sapp::IconDesc {
                sokol_default: true,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

// ---------------------------------------------------------------------
// Example usage
// ---------------------------------------------------------------------

pub fn run() {}
