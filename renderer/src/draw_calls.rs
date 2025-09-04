use crate::Descriptor;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
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
