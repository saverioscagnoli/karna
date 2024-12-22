use std::ffi::CStr;

use crate::{info, input::Input, Audio, Renderer, Time, Window};
use sdl2::{
    video::{self, GLContext},
    VideoSubsystem,
};

pub(crate) struct SDL {
    ctx: sdl2::Sdl,
    video: sdl2::VideoSubsystem,
    pub(crate) event_pump: sdl2::EventPump,
}

impl SDL {
    pub(crate) fn init() -> Self {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let event_pump = sdl.event_pump().unwrap();

        Self {
            ctx: sdl,
            video,
            event_pump,
        }
    }
}

pub struct Context {
    pub(crate) sdl: SDL,
    pub(crate) running: bool,
    pub window: Window,
    pub render: Renderer,
    pub input: Input,
    pub time: Time,
    pub audio: Audio,

    _gl_context: GLContext,
    _gl: (),
}

impl Context {
    pub(crate) fn new(title: &str, width: u32, height: u32) -> Self {
        let sdl = SDL::init();

        let (window, _gl_context, _gl) = Context::init_opengl(&sdl.video, title, width, height);

        let (width, height) = window.size();

        let window = Window::new(window);
        let render = Renderer::new(width, height);
        let input = Input::new(&sdl.ctx);
        let time = Time::new();
        let audio = Audio::new();

        info!("Karna v{}", env!("CARGO_PKG_VERSION"));

        unsafe {
            let version = gl::GetString(gl::VERSION);
            let renderer = gl::GetString(gl::RENDERER);

            if !version.is_null() {
                let version = CStr::from_ptr(version as *const _).to_str().unwrap();
                info!("Using OpenGL v{}", version);
            }

            if !renderer.is_null() {
                let renderer = CStr::from_ptr(renderer as *const _).to_str().unwrap();
                info!("Device: {}", renderer);
            }
        }

        Self {
            sdl,
            running: false,
            window,
            render,
            input,
            time,
            audio,
            _gl_context,
            _gl,
        }
    }

    pub(crate) fn start(&mut self) {
        self.running = true;
    }

    pub fn init_opengl(
        video: &VideoSubsystem,
        title: &str,
        width: u32,
        height: u32,
    ) -> (video::Window, GLContext, ()) {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let window = video.window(title, width, height).opengl().build().unwrap();

        let _gl_context = window.gl_create_context().unwrap();
        let _gl = gl::load_with(|s| video.gl_get_proc_address(s) as *const _);

        (window, _gl_context, _gl)
    }
}
