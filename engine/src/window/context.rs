use imgui::Imgui;

use crate::assets::Assets;
use crate::render::immediate::{Draw, ImguiFrame};
use crate::render::packet::FramePacket;
use crate::render::retained::SceneRef;
use crate::window::WindowHandle;
use crate::window::audio::AudioHandle;
use crate::window::input::Input;
use crate::window::resources::Resources;
use crate::window::time::Time;

pub struct WindowContext {
    pub window: WindowHandle,
    pub input: Input,
    pub audio: AudioHandle,
    pub resources: Resources,
    pub packet: FramePacket,
}

pub struct ContextRef<'a, A = &'a mut Assets> {
    pub window: &'a mut WindowHandle,
    pub time: &'a mut Time,
    pub input: &'a mut Input,
    pub audio: &'a AudioHandle,
    pub assets: A,
    pub resources: &'a mut Resources,
}

pub type DrawContext<'a> = ContextRef<'a, &'a Assets>;

impl WindowContext {
    pub fn split_scene<'a>(
        &'a mut self,
        time: &'a mut Time,
        assets: &'a mut Assets,
    ) -> (ContextRef<'a>, SceneRef<'a>) {
        (
            ContextRef {
                window: &mut self.window,
                time,
                input: &mut self.input,
                audio: &self.audio,
                assets,
                resources: &mut self.resources,
            },
            SceneRef {
                packet: &mut self.packet,
            },
        )
    }

    pub fn split_draw<'a>(
        &'a mut self,
        time: &'a mut Time,
        assets: &'a mut Assets,
        imgui: &'a mut Imgui,
    ) -> (DrawContext<'a>, Draw<'a>) {
        let frame = ImguiFrame {
            logical: self.window.size(),
            pixel: self.window.pixel_size(),
            delta: time.delta(),
        };

        (
            ContextRef {
                window: &mut self.window,
                time,
                input: &mut self.input,
                audio: &self.audio,
                assets,
                resources: &mut self.resources,
            },
            Draw::new(&mut self.packet, assets, imgui, frame),
        )
    }
}
