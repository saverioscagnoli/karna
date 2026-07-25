use crate::render::{color::Color, layer::Layer, packet::FramePacket, vertex::Vertex};

pub struct Draw<'ctx> {
    pub(crate) packet: &'ctx mut FramePacket,
    pub(crate) color: Color,
    pub(crate) active_layer: Layer,
}

impl<'ctx> Draw<'ctx> {
    pub fn clear_color(&self) -> Color {
        self.packet.clear_color
    }

    pub fn clear_color_mut(&mut self) -> &mut Color {
        &mut self.packet.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.packet.clear_color = color.into();
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.color = color.into();
    }

    pub fn with_color<C, F>(&mut self, color: C, f: F)
    where
        C: Into<Color>,
        F: FnOnce(&mut Self),
    {
        let previous = self.color;
        self.color = color.into();
        f(self);
        self.color = previous;
    }

    pub fn layer(&self) -> Layer {
        self.active_layer
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.active_layer = layer;
    }

    pub fn on_layer<F>(&mut self, layer: Layer, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let previous = self.active_layer;
        self.active_layer = layer;
        f(self);
        self.active_layer = previous;
    }

    pub fn push_quad(&mut self, corners: [math::Vector2<f32>; 4], z: f32) {
        const UVS: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

        let color: math::Vector4<f32> = self.color.into();
        let data = &mut self.packet[self.active_layer].data;

        let base = data.vertices.len() as u32;

        for (corner, &(u, v)) in corners.iter().zip(UVS.iter()) {
            data.vertices.push(Vertex {
                position: math::Vector3::new(corner.x, corner.y, z),
                color,
                uv: math::Vector2::new(u, v),
            });
        }

        data.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.push_quad(
            [
                math::Vector2::new(x, y),
                math::Vector2::new(x + w, y),
                math::Vector2::new(x + w, y + h),
                math::Vector2::new(x, y + h),
            ],
            0.0,
        );
    }
}
