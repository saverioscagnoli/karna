use renderer::Draw;
use renderer::Renderer;

use crate::input::Input;
use crate::monitors::Monitor;
use crate::monitors::Monitors;
use crate::scene::SceneManager;
use crate::time::Time;
use crate::window::Window;

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub scenes: SceneManager,
    pub renderer: Renderer,
    pub monitors: Monitors,
}

impl WindowContext {
    pub fn new(window: Window, renderer: Renderer, monitors: Vec<Monitor>) -> Self {
        // Arc::clone
        let winit_window = window.inner.clone();

        Self {
            window,
            time: Time::new(),
            input: Input::new(),
            scenes: SceneManager::new(),
            renderer,
            monitors: Monitors::new(winit_window, monitors),
        }
    }

    pub fn as_ref_mut<'ctx>(&'ctx mut self) -> ContextRefMut<'ctx> {
        ContextRefMut {
            window: &self.window,
            time: &mut self.time,
            input: &mut self.input,
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
            scenes: &self.scenes,
            monitors: &self.monitors,
        };

        let draw = Draw::_new(&mut self.renderer);

        (ctx, draw)
    }
}

pub struct ContextRefMut<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx mut Time,
    pub input: &'ctx mut Input,
    pub scenes: &'ctx mut SceneManager,
    pub render: &'ctx mut Renderer,
    pub monitors: &'ctx Monitors,
}

pub struct ContextRef<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx Time,
    pub input: &'ctx Input,
    pub scenes: &'ctx SceneManager,
    pub monitors: &'ctx Monitors,
}
