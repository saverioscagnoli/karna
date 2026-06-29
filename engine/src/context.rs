use renderer::Renderer;

use crate::scene::SceneManager;
use crate::time::Time;
use crate::window::Window;

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub scenes: SceneManager,
    pub renderer: Renderer,
}

impl WindowContext {
    pub fn new(window: Window, renderer: Renderer) -> Self {
        Self {
            window,
            time: Time::new(),
            scenes: SceneManager::new(),
            renderer,
        }
    }

    pub fn as_ref_mut<'ctx>(&'ctx mut self) -> ContextRefMut<'ctx> {
        ContextRefMut {
            window: &self.window,
            time: &mut self.time,
            scenes: &mut self.scenes,
            render: &mut self.renderer,
        }
    }

    pub fn as_ref<'ctx>(&'ctx self) -> ContextRef<'ctx> {
        ContextRef {
            window: &self.window,
            time: &self.time,
            scenes: &self.scenes,
            render: &self.renderer,
        }
    }
}

unsafe impl Send for WindowContext {}
unsafe impl Sync for WindowContext {}

pub struct ContextRefMut<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx mut Time,
    pub scenes: &'ctx mut SceneManager,
    pub render: &'ctx mut Renderer,
}

pub struct ContextRef<'ctx> {
    pub window: &'ctx Window,
    pub time: &'ctx Time,
    pub scenes: &'ctx SceneManager,
    pub render: &'ctx Renderer,
}
