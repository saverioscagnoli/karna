#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Error while creating the surface: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),

    #[error("Error while requesting the adapter: {0}")]
    AdapterRequest(#[from] wgpu::RequestAdapterError),

    #[error("Error while requesting the device: {0}")]
    DeviceRequest(#[from] wgpu::RequestDeviceError),
}
