use std::collections::HashSet;

use crate::{flags::LoopFlag, input::Input, render::Renderer, time::Time, window::Window};

pub struct Context {
    pub(crate) running: bool,
    pub window: Window,
    pub time: Time,
    pub render: Renderer,
    pub input: Input,
}

impl Context {
    pub(crate) fn new(window: Window, flags: &HashSet<LoopFlag>) -> Self {
        let window_clone = window.clone();
        let mut canvas = window.0.into_canvas();

        if flags.contains(&LoopFlag::Accelerated) {
            canvas = canvas.accelerated();
        }

        if flags.contains(&LoopFlag::VSync) {
            canvas = canvas.present_vsync();
        }

        Self {
            running: false,
            window: window_clone,
            time: Time::new(),
            render: Renderer::new(canvas.build().unwrap()),
            input: Input::new(),
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}
