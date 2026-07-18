use crate::render::Color;
use crate::render::Draw;
use crate::render::Renderer;
use crate::scene::SceneManager;
use crate::window::Window;
use crate::window::input::Input;
use crate::window::time::Time;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex2D {
    pub pos: [f32; 2], // pixels; shader converts to NDC via a viewport uniform
    pub color: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct FramePacket {
    pub viewport: math::Size<u32>,
    pub clear_color: math::Vector4<f32>,
    pub vertices: Vec<Vertex2D>,
    pub indices: Vec<u32>,

    /// SDL timestamp (ns) of the oldest input event this frame is the first to reflect.
    pub input_timestamp: Option<u64>,
}

impl Default for FramePacket {
    fn default() -> Self {
        Self {
            viewport: math::Size::new(1280, 720),
            clear_color: Color::Gray.into(),
            vertices: Vec::new(),
            indices: Vec::new(),
            input_timestamp: None,
        }
    }
}

impl FramePacket {
    pub fn clear(&mut self) {
        self.vertices.clear(); // keeps capacity — no realloc next frame
        self.indices.clear();
        self.input_timestamp = None;
    }
}

pub struct WindowContext {
    pub window: Window,
    pub input: Input,
    pub time: Time,
    pub scenes: SceneManager,
    pub renderer: Renderer,
}

pub struct Context<'a> {
    pub window: &'a mut Window,
    pub input: &'a mut Input,
    pub time: &'a mut Time,
    pub scenes: &'a mut SceneManager,
}

impl WindowContext {
    pub fn split_draw<'a>(&'a mut self, packet: &'a mut FramePacket) -> (Context<'a>, Draw<'a>) {
        (
            Context {
                window: &mut self.window,
                input: &mut self.input,
                time: &mut self.time,
                scenes: &mut self.scenes,
            },
            Draw {
                r: &mut self.renderer,
                packet,
            },
        )
    }
}
