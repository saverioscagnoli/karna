#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("{0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),

    #[error("{0}")]
    AdapterRequest(#[from] wgpu::RequestAdapterError),

    #[error("{0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),
}
