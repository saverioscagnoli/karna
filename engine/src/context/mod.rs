//mod input;
//mod time;
//mod window;
//
//use crate::context::{input::Input, time::Time, window::Window};
//use renderer::{Renderer, SharedGPU};
//use std::sync::Arc;
//
//pub struct Context {
//    pub window: Window,
//    pub time: Time,
//    pub input: Input,
//    pub render: Renderer,
//    pub gpu: Arc<SharedGPU>,
//}
//
//impl Context {
//    pub(crate) fn new(window: Arc<winit::window::Window>, gpu: Arc<SharedGPU>) -> Self {
//        Self {
//            window: Window::from_winit(window.clone()),
//            time: Time::new(),
//            input: Input::new(),
//            render: Renderer::new(window, gpu.clone()),
//            gpu,
//        }
//    }
//}
mod input;
mod time;

use common::utils::Label;
use renderer::{Renderer, SurfaceState};
use std::sync::Arc;
use wgpu::naga::FastHashMap;
use winit::window::Window;

use crate::context::{input::Input, time::Time};

pub struct Context {
    pub render: Arc<Renderer>,
    pub windows: FastHashMap<Label, (Arc<Window>, SurfaceState)>,
    pub time: Time,
    pub input: Input,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            render: Arc::new(pollster::block_on(Renderer::new())),
            windows: FastHashMap::default(),
            time: Time::new(),
            input: Input::new(),
        }
    }
    pub(crate) fn for_window(&mut self, label: &Label) -> ScopedContext<'_> {
        self.windows
            .get_mut(label)
            .map(|(window, surface)| ScopedContext {
                render: &self.render,
                window,
                time: &self.time,
                input: &self.input,
                surface,
            })
            .unwrap()
    }
}

pub struct ScopedContext<'a> {
    pub window: &'a Arc<Window>,
    pub render: &'a Arc<Renderer>,
    pub time: &'a mut Time,
    pub input: &'a mut Input,
    pub surface: &'a mut SurfaceState,
}
