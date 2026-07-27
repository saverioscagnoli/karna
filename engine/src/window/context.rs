use crate::render::immediate::Draw;
use crate::render::packet::FramePacket;
use crate::render::retained::SceneRef;
use crate::window::Window;
use crate::window::input::Input;
use crate::window::resources::Resources;
use crate::window::time::Time;

pub struct WindowContext {
    pub window: Window,
    pub input: Input,
    pub resources: Resources,
    pub packet: FramePacket,
}

pub struct ContextRef<'a> {
    pub window: &'a mut Window,
    pub input: &'a mut Input,
    pub resources: &'a mut Resources,
    pub time: &'a mut Time,
}

impl WindowContext {
    pub fn split_scene<'a>(&'a mut self, time: &'a mut Time) -> (ContextRef<'a>, SceneRef<'a>) {
        (
            ContextRef {
                window: &mut self.window,
                input: &mut self.input,
                resources: &mut self.resources,
                time,
            },
            SceneRef {
                packet: &mut self.packet,
            },
        )
    }

    pub fn split_draw<'a>(&'a mut self, time: &'a mut Time) -> (ContextRef<'a>, Draw<'a>) {
        (
            ContextRef {
                window: &mut self.window,
                input: &mut self.input,
                resources: &mut self.resources,
                time,
            },
            Draw {
                packet: &mut self.packet,
            },
        )
    }
}
