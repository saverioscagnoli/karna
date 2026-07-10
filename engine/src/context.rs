use renderer::Camera;
use renderer::Color;
use renderer::DrawCommand;
use renderer::FramePacket;
use renderer::Layer;
use renderer::Projection;

use crate::SceneManager;
use crate::Time;
use crate::Window;
use crate::input::Input;

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub scenes: SceneManager,

    pub world_camera: Camera,
    pub ui_camera: Camera,
    pub debug_camera: Camera,
}

impl WindowContext {
    pub fn new(window: Window) -> Self {
        let view = window.size();

        Self {
            window,
            time: Time::new(),
            input: Input::new(),
            scenes: SceneManager::new(),
            world_camera: Camera::new(Projection::standard_2d(view)),
            ui_camera: Camera::new(Projection::standard_2d(view)),
            debug_camera: Camera::new(Projection::standard_2d(view)),
        }
    }
}

pub struct ContextMut<'a> {
    pub window: &'a Window, // Cannot be mutated, no sense in making it &mut
    pub time: &'a mut Time,
    pub input: &'a mut Input,
    pub scenes: &'a mut SceneManager,
}

pub struct ContextRef<'a> {
    pub window: &'a Window,
    pub time: &'a Time,
    pub input: &'a Input,
    pub scenes: &'a SceneManager,
}

impl WindowContext {
    pub fn as_ref<'a>(&'a self) -> ContextRef<'a> {
        ContextRef {
            window: &self.window,
            time: &self.time,
            input: &self.input,
            scenes: &self.scenes,
        }
    }

    pub fn split_mut<'a>(
        &'a mut self,
        packet: &'a mut FramePacket,
    ) -> (ContextMut<'a>, SceneHandle<'a>) {
        let Self {
            window,
            time,
            input,
            scenes,
            world_camera,
            ui_camera,
            debug_camera,
        } = self;

        (
            ContextMut {
                window,
                time,
                input,
                scenes,
            },
            SceneHandle {
                active_layer: Layer::World,
                packet,
                world_camera,
                ui_camera,
                debug_camera,
            },
        )
    }

    pub fn split<'a>(&'a self, packet: &'a mut FramePacket) -> (ContextRef<'a>, Draw<'a>) {
        (self.as_ref(), Draw::new(packet))
    }
}

pub struct Draw<'a> {
    packet: &'a mut FramePacket,
    color: math::Vector4<f32>,
    active_layer: Layer,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(packet: &'a mut FramePacket) -> Self {
        Self {
            packet,
            color: Color::White.into(),
            active_layer: Layer::World,
        }
    }

    fn expose_active(&mut self) -> &mut Vec<DrawCommand> {
        self.packet.expose(self.active_layer)
    }

    pub fn clear_color(&self) -> Color {
        self.packet.clear_color.into()
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        self.packet.clear_color = color.into();
    }

    pub fn color(&self) -> Color {
        self.color.into()
    }

    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        self.color = color.into();
    }

    pub fn point(&mut self, x: f32, y: f32) {
        let color = self.color;

        self.expose_active()
            .push(DrawCommand::ImmediatePoint { x, y, color });
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let color = self.color;

        self.expose_active().push(DrawCommand::ImmediateLine {
            x1,
            y1,
            x2,
            y2,
            color,
        });
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let color = self.color;

        self.expose_active()
            .push(DrawCommand::ImmediateRect { x, y, w, h, color });
    }
}

pub struct SceneHandle<'a> {
    active_layer: Layer,
    packet: &'a mut FramePacket,
    pub world_camera: &'a mut Camera,
    pub ui_camera: &'a mut Camera,
    pub debug_camera: &'a mut Camera,
}

impl<'a> SceneHandle<'a> {
    fn new(ctx: &'a mut WindowContext, packet: &'a mut FramePacket) -> Self {
        Self {
            active_layer: Layer::World,
            packet,
            world_camera: &mut ctx.world_camera,
            ui_camera: &mut ctx.ui_camera,
            debug_camera: &mut ctx.debug_camera,
        }
    }

    pub fn clear_color(&self) -> Color {
        self.packet.clear_color.into()
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        self.packet.clear_color = color.into();
    }
}
