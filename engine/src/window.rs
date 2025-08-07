use err::EngineError;
use sdl2::video::{GLContext, Window as SdlWindow};

pub struct Window {
    inner: SdlWindow,
    _gl_context: GLContext,
}

impl Window {
    pub(crate) fn new<T: AsRef<str>>(
        video: &sdl2::VideoSubsystem,
        title: T,
        width: u32,
        height: u32,
    ) -> Result<Self, EngineError> {
        let gl_attr = video.gl_attr();

        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3); // OpenGL 3.3
        gl_attr.set_double_buffer(true);
        gl_attr.set_depth_size(24);

        let inner = video
            .window(title.as_ref(), width, height)
            .position_centered()
            .opengl()
            .build()?;

        let _gl_context = inner
            .gl_create_context()
            .map_err(|e| EngineError::OpenGLContextCreation(e.to_string()))?;

        inner
            .gl_make_current(&_gl_context)
            .map_err(|e| EngineError::OpenGLContextCreation(e.to_string()))?;

        gl::load_with(|name| video.gl_get_proc_address(name) as *const _);

        Ok(Self { inner, _gl_context })
    }

    #[inline]
    pub fn present(&self) {
        self.inner.gl_swap_window();
    }
}
