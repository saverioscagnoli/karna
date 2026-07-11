use gpu::Vertex;
use logging::warn;
use renderer::Camera;
use renderer::Color;
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

#[derive(Debug, Clone, Copy)]
struct DrawState {
    draw_color: math::Vector4<f32>,
    transform: math::Matrix4<f32>,
    depth: f32,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            draw_color: math::Vector4::new(1.0, 1.0, 1.0, 1.0),
            transform: math::Matrix4::identity(),
            depth: 0.0,
        }
    }
}

pub struct Draw<'a> {
    packet: &'a mut FramePacket,
    state_stack: Vec<DrawState>,
    current_state: DrawState,
    active_layer: Layer,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(packet: &'a mut FramePacket) -> Self {
        Self {
            packet,
            state_stack: Vec::new(),
            current_state: DrawState::default(),
            active_layer: Layer::World,
        }
    }

    pub fn push_state(&mut self) {
        self.state_stack.push(self.current_state);
    }

    pub fn pop_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.current_state = state;
        } else {
            warn!("Draw: popped a render state without pushing first");
        }
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.active_layer = layer;
    }

    pub fn clear_color(&self) -> Color {
        self.packet.clear_color.into()
    }

    pub fn set_clear_color<C: Into<math::Vector4<f32>>>(&mut self, color: C) {
        self.packet.clear_color = color.into();
    }

    pub fn color(&self) -> Color {
        self.current_state.draw_color.into()
    }

    pub fn set_color<C: Into<math::Vector4<f32>>>(&mut self, color: C) {
        self.current_state.draw_color = color.into();
    }

    pub fn depth(&self) -> f32 {
        self.current_state.depth
    }

    pub fn set_depth(&mut self, d: f32) {
        self.current_state.depth = d;
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.current_state.transform =
            self.current_state
                .transform
                .matmul(&math::Matrix4::from_translation(math::Vector3::new(
                    x, y, 0.0,
                )));
    }

    pub fn rotate(&mut self, angle_rad: f32) {
        self.current_state.transform = self
            .current_state
            .transform
            .matmul(&math::Matrix4::from_rotation_z(angle_rad));
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.current_state.transform = self
            .current_state
            .transform
            .matmul(&math::Matrix4::from_scale(math::Vector3::new(x, y, 1.0)));
    }

    fn vertex(&self, x: f32, y: f32, uv: math::Vector2<f32>) -> Vertex {
        let s = &self.current_state;
        let t = s.transform.mul_vec(&math::Vector4::new(x, y, 0.0, 1.0));
        let pos = math::Vector3::new(t.x, t.y, s.depth);

        Vertex::new(pos, s.draw_color, uv)
    }

    // ---- primitives ----

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let uv = math::Vector2::zero();

        let v = [
            self.vertex(x, y, uv),
            self.vertex(x + w, y, uv),
            self.vertex(x, y + h, uv),
            self.vertex(x + w, y + h, uv),
        ];

        self.packet
            .layer_mut(self.active_layer)
            .triangles
            .push(&v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        // Tessellate as a thin quad so it goes through the triangle path.
        let width = 1.0;
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt();

        if len <= f32::EPSILON {
            return;
        }

        let (nx, ny) = (-dy / len * width * 0.5, dx / len * width * 0.5);
        let uv = math::Vector2::zero();

        let v = [
            self.vertex(x1 + nx, y1 + ny, uv),
            self.vertex(x1 - nx, y1 - ny, uv),
            self.vertex(x2 + nx, y2 + ny, uv),
            self.vertex(x2 - nx, y2 - ny, uv),
        ];

        self.packet
            .layer_mut(self.active_layer)
            .triangles
            .push(&v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn point(&mut self, x: f32, y: f32) {
        let size = 1.0;
        self.rect(x - size * 0.5, y - size * 0.5, size, size);
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
