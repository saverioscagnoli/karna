mod color;
mod draw;

use gpu::GpuState;
use logging::warn;

use crate::window::context::FramePacket;

pub use crate::render::color::Color;
pub use crate::render::draw::Draw;

pub struct Renderer {
    surface: gpu::WindowSurface,
}

impl Renderer {
    pub(crate) fn new(surface: gpu::WindowSurface) -> Self {
        Self { surface }
    }

    pub(crate) fn resize(&mut self, gpu: &gpu::GpuState, size: math::Size<u32>) {
        self.surface.resize(gpu, size);
    }

    pub(crate) fn present(&mut self, packet: &FramePacket) {
        let gpu = GpuState::get();
        let output = match self.surface.acquire() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.resize(gpu, packet.viewport);
                warn!("Received an outdated texture, skipping this frame");
                return;
            }

            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                warn!("Failed to acquire a texture, skipping this frame");
                return;
            }

            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Device lost");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color::Cyan.into()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        gpu.queue.present(output);
    }
}
