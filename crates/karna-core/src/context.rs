use karna_log::{info, KarnaError};
use karna_math::size::Size;
use sdl3::{
    video::{GLContext, GLProfile, WindowBuilder},
    Sdl,
};

use crate::{time::Time, window::Window};

#[derive(Debug, Clone)]
pub enum Flags {
    Centered,
    Positioned(i32, i32),
    Fullscreen,
    Hidden,
    Borderless,
    Resizable,
    Minimized,
    Maximized,
    InputGrabbed,
}

impl Default for Flags {
    fn default() -> Self {
        Flags::Centered
    }
}

pub struct Context {
    /// Private stuff, such as the SDL context and the OpenGL context.
    pub(crate) sdl: Sdl,
    pub(crate) should_close: bool,

    /// Must keep this alive, otherwise the OpenGL context will be destroyed.
    _gl_context: GLContext,

    /// The handle to the main window,
    /// So that it can be interacted with.
    pub window: Window,

    /// The time interface to handle all time-related operations.
    /// This includes delta time, elapsed time, fps, ticks, etc.
    pub time: Time,
}

impl Context {
    pub(crate) fn init(title: String, size: Size<u32>, window_flags: Option<&[Flags]>) -> Self {
        let sdl = sdl3::init()
            .map_err(|e| KarnaError::Sdl("Initialization".to_string(), e.to_string()))
            .unwrap();

        let video = sdl
            .video()
            .map_err(|e| KarnaError::Sdl("Video initialization".to_string(), e.to_string()))
            .unwrap();

        let gl_attr = video.gl_attr();

        gl_attr.set_context_version(4, 6);
        gl_attr.set_context_profile(GLProfile::Core);

        let mut builder = video.window(title.as_ref(), size.width, size.height);

        Context::apply_flags(window_flags.unwrap_or(&[]), &mut builder);

        let sdl_window = builder
            .opengl()
            .build()
            .map_err(|e| KarnaError::Sdl("Window creation".to_string(), e.to_string()))
            .unwrap();

        let _gl_context = sdl_window
            .gl_create_context()
            .map_err(|e| KarnaError::Sdl("OpenGL context creation".to_string(), e.to_string()))
            .unwrap();

        gl::load_with(|name| video.gl_get_proc_address(name).unwrap() as _);

        // TODO: Dynamic version
        info!("Using OpenGL v{}.{}", 4, 6);

        let window = Window::new(sdl_window);
        let time = Time::new();

        Self {
            sdl,
            should_close: false,
            window,
            time,
            _gl_context,
        }
    }

    fn apply_flags(window_flags: &[Flags], builder: &mut WindowBuilder) {
        for flag in window_flags {
            match flag {
                Flags::Centered => builder.position_centered(),
                Flags::Positioned(x, y) => builder.position(*x, *y),
                Flags::Fullscreen => builder.fullscreen(),
                Flags::Hidden => builder.hidden(),
                Flags::Borderless => builder.borderless(),
                Flags::Resizable => builder.resizable(),
                Flags::Minimized => builder.minimized(),
                Flags::Maximized => builder.maximized(),
                Flags::InputGrabbed => builder.input_grabbed(),
            };
        }
    }
}
