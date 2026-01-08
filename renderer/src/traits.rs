pub trait LayoutDescriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}
