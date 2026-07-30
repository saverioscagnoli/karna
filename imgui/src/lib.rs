mod context;
mod platform;
mod renderer;
mod textures;
mod widgets;

use std::ptr;

use dear_imgui_sys::*;
use sdl3::event::Event;

pub use crate::widgets::*;
pub use crate::{
    context::ImguiContext, platform::Capture, renderer::ImguiRenderer, renderer::ImguiVertex,
};

/// The geometry imgui produced for one frame
#[derive(Debug, Clone, Copy)]
pub struct DrawData(*mut ImDrawData);

impl DrawData {
    pub fn is_empty(self) -> bool {
        self.0.is_null()
    }

    pub(crate) fn as_ptr(self) -> *mut ImDrawData {
        self.0
    }
}

pub struct Imgui {
    context: ImguiContext,
    pub renderer: ImguiRenderer,
    capture: Capture,
    in_frame: bool,
}

impl Imgui {
    pub fn new() -> Self {
        Self {
            context: ImguiContext::new(),
            renderer: ImguiRenderer::new(),
            capture: Capture::default(),
            in_frame: false,
        }
    }

    pub fn capture(&self) -> Capture {
        self.capture
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.context.set_current();
        self.capture = platform::handle_event(self.context.io(), event);
    }

    pub fn begin_frame(
        &mut self,
        logical: math::Size<u32>,
        pixel: math::Size<u32>,
        delta: f32,
    ) -> Ui<'_> {
        self.context.set_current();

        if !self.in_frame {
            platform::new_frame(self.context.io(), logical, pixel, delta);
            self.in_frame = true;
        }

        Ui::new()
    }

    pub fn end_frame(&mut self) -> DrawData {
        if !self.in_frame {
            return DrawData(ptr::null_mut());
        }

        self.context.set_current();
        self.in_frame = false;

        unsafe {
            igRender();
            DrawData(igGetDrawData())
        }
    }
}
