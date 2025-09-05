use crate::Descriptor;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveKind {
    Point,
    Triangle,
}

impl PrimitiveKind {
    pub fn num_vertices(&self) -> u32 {
        match self {
            PrimitiveKind::Point => 1,
            PrimitiveKind::Triangle => 3,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawIndirectArgs {
    pub primitive: PrimitiveKind,
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

impl Descriptor for DrawIndirectArgs {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DrawIndirectArgs>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[],
        }
    }
}
