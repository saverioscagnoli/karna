use crate::{
    audio::Audio, info, input::Input, math::Size, render::renderer::Renderer, time::Time,
    window::Window,
};

pub(crate) struct Sdl {
    pub(crate) sys: sdl2::Sdl,
    pub(crate) _video: sdl2::VideoSubsystem,
}

pub struct Context {
    pub(crate) running: bool,
    pub(crate) sdl: Sdl,
    pub render: Renderer,
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub audio: Audio,
}

impl Context {
    pub(crate) fn new<T: ToString, S: Into<Size>>(title: T, size: S) -> Result<Self, String> {
        let sys = sdl2::init()?;
        let video = sys.video()?;

        info!("SDL Version: {}", sdl2::version::version());
        info!("Karna Version: {}", env!("CARGO_PKG_VERSION"));

        let size = size.into();
        let window = video
            .window(title.to_string().as_str(), size.width, size.height)
            .position_centered()
            .hidden()
            .build()
            .map_err(|e| e.to_string())?;

        let window_clone = window.clone();

        let canvas = window_clone
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())?;

        let render = Renderer::new(canvas);
        let window = Window::new(window);
        let time = Time::new();
        let input = Input::new();
        let audio = Audio::new();

        Ok(Self {
            running: false,
            sdl: Sdl { sys, _video: video },
            render,
            window,
            time,
            input,
            audio,
        })
    }

    pub(crate) fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}
