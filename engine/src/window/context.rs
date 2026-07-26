use crate::{
    render::{
        color::Color,
        immediate::Draw,
        layer::{Layer, LayerCameras},
        packet::FramePacket,
        retained::SceneRef,
    },
    window::{WindowHandle, input::Input, resources::Resources, time::Time},
};

#[derive(Debug)]
pub struct WindowContext {
    // This will be split to context ref
    // Primarily just data
    pub window: WindowHandle,
    pub time: Time,
    pub input: Input,
    pub resources: Resources,

    // This will be split to scene ref
    // Primarily scene / render stuff
    pub cameras: LayerCameras,
    pub packet: FramePacket,
}

pub struct ContextRef<'a> {
    pub window: &'a mut WindowHandle,
    pub time: &'a mut Time,
    pub input: &'a mut Input,
    pub resources: &'a mut Resources,
}

impl WindowContext {
    pub fn split_scene<'a>(&'a mut self) -> (ContextRef<'a>, SceneRef<'a>) {
        (
            ContextRef {
                window: &mut self.window,
                time: &mut self.time,
                input: &mut self.input,
                resources: &mut self.resources,
            },
            SceneRef {
                active_layer: Layer::default(),
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
                resources: &mut self.resources,
            },
            Draw {
                packet: &mut self.packet,
                color: Color::default(),
                active_layer: Layer::default(),
            },
        )
    }
}
