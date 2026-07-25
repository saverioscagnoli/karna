use crate::{
    render::{
        color::Color,
        immediate::Draw,
        layer::{Layer, LayerCameras},
        packet::FramePacket,
        retained::SceneRef,
    },
    window::{WindowHandle, input::Input, time::Time},
};

#[derive(Debug)]
pub struct WindowContext {
    pub window: WindowHandle,
    pub time: Time,
    pub input: Input,
    pub cameras: LayerCameras,
    pub packet: FramePacket,
}

pub struct ContextRef<'a> {
    pub window: &'a mut WindowHandle,
    pub time: &'a mut Time,
    pub input: &'a mut Input,
}

impl WindowContext {
    pub fn split_scene<'a>(&'a mut self) -> (ContextRef<'a>, SceneRef<'a>) {
        (
            ContextRef {
                window: &mut self.window,
                time: &mut self.time,
                input: &mut self.input,
            },
            SceneRef {
                cameras: &mut self.cameras,
            },
        )
    }

    pub fn split_draw<'a>(&'a mut self) -> (ContextRef<'a>, Draw<'a>) {
        (
            ContextRef {
                window: &mut self.window,
                time: &mut self.time,
                input: &mut self.input,
            },
            Draw {
                packet: &mut self.packet,
                color: Color::default(),
                active_layer: Layer::default(),
            },
        )
    }
}
