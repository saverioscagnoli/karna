use std::ffi;

use sokol::app as sapp;
use sokol::gfx as sg;
use sokol::glue as sglue;

pub trait Application {
    fn init(&mut self);
    fn frame(&mut self);
    fn cleanup(&mut self) {}
}

pub struct Window<A: Application> {
    app: A,
    title: &'static ffi::CStr,
    width: i32,
    height: i32,
    sample_count: i32,
}

impl<A: Application> Window<A> {
    pub fn new(app: A, title: &'static ffi::CStr, width: i32, height: i32) -> Self {
        Self {
            app,
            title,
            width,
            height,
            sample_count: 4,
        }
    }

    pub fn run(self) {
        let user_data = Box::into_raw(Box::new(self.app)) as *mut ffi::c_void;

        extern "C" fn init_cb<A: Application>(user_data: *mut ffi::c_void) {
            let app = unsafe { &mut *(user_data as *mut A) };
            sg::setup(&sg::Desc {
                environment: sglue::environment(),
                logger: sg::Logger {
                    func: Some(sokol::log::slog_func),
                    ..Default::default()
                },
                ..Default::default()
            });
            app.init();
        }

        extern "C" fn frame_cb<A: Application>(user_data: *mut ffi::c_void) {
            let app = unsafe { &mut *(user_data as *mut A) };
            app.frame();
        }

        extern "C" fn cleanup_cb<A: Application>(user_data: *mut ffi::c_void) {
            let mut app = unsafe { Box::from_raw(user_data as *mut A) };
            app.cleanup();
            sg::shutdown();
        }

        sapp::run(&sapp::Desc {
            init_userdata_cb: Some(init_cb::<A>),
            frame_userdata_cb: Some(frame_cb::<A>),
            cleanup_userdata_cb: Some(cleanup_cb::<A>),
            user_data,
            window_title: self.title.as_ptr(),
            width: self.width,
            height: self.height,
            sample_count: self.sample_count,
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

struct ClearApp {
    pass_action: sg::PassAction,
}

impl Application for ClearApp {
    fn init(&mut self) {
        self.pass_action.colors[0] = sg::ColorAttachmentAction {
            load_action: sg::LoadAction::Clear,
            clear_value: sg::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            ..Default::default()
        };

        let backend = sg::query_backend();
        println!("Using backend: {:?}", backend);
    }

    fn frame(&mut self) {
        let g = self.pass_action.colors[0].clear_value.g + 0.01;
        self.pass_action.colors[0].clear_value.g = if g > 1.0 { 0.0 } else { g };

        sg::begin_pass(&sg::Pass {
            action: self.pass_action,
            swapchain: sglue::swapchain(),
            ..Default::default()
        });
        sg::end_pass();
        sg::commit();
    }
}

pub fn run() {
    let app = ClearApp {
        pass_action: sg::PassAction::new(),
    };
    Window::new(app, c"clear.rs", 800, 600).run();
}
