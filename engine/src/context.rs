use renderer::DrawCommand;
use renderer::FramePacket;
use renderer::Layer;

use crate::SceneManager;
use crate::Time;
use crate::Window;

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub scenes: SceneManager,
}

impl WindowContext {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            time: Time::new(),
            scenes: SceneManager::new(),
        }
    }
}

pub struct ContextMut<'a> {
    pub window: &'a Window, // Cannot be mutated, no sense in making it &mut
    pub time: &'a mut Time,
    pub scenes: &'a mut SceneManager,
}

pub struct ContextRef<'a> {
    pub window: &'a Window,
    pub time: &'a Time,
    pub scenes: &'a SceneManager,
}

impl WindowContext {
    pub fn as_mut<'a>(&'a mut self) -> ContextMut<'a> {
        ContextMut {
            window: &self.window,
            time: &mut self.time,
            scenes: &mut self.scenes,
        }
    }

    pub fn as_ref<'a>(&'a self) -> ContextRef<'a> {
        ContextRef {
            window: &self.window,
            time: &self.time,
            scenes: &self.scenes,
        }
    }

    pub fn split<'a>(&'a self, packet: &'a mut FramePacket) -> (ContextRef<'a>, Draw<'a>) {
        (self.as_ref(), Draw::new(packet))
    }
}

pub struct Draw<'a> {
    packet: &'a mut FramePacket,
    active_layer: Layer,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(packet: &'a mut FramePacket) -> Self {
        Self {
            packet,
            active_layer: Layer::World,
        }
    }

    fn expose_active(&mut self) -> &mut Vec<DrawCommand> {
        self.packet.expose(self.active_layer)
    }

    pub fn point(&mut self, x: f32, y: f32) {
        self.expose_active()
            .push(DrawCommand::ImmediatePoint { x, y });
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.expose_active()
            .push(DrawCommand::ImmediateLine { x1, y1, x2, y2 });
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.expose_active()
            .push(DrawCommand::ImmediateRect { x, y, w, h });
    }
}

pub struct SceneHandle<'a> {}
