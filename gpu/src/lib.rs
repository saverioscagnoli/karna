mod buffer;
mod pipeline;
mod shaders;
mod texture;
mod vertex;

use logging::debug;
use logging::warn;
use sdl3::video::Window;

pub use sdl3::gpu::CommandBuffer;
pub use sdl3::gpu::CopyPass;
pub use sdl3::gpu::PresentMode;
pub use sdl3::gpu::PrimitiveType as PrimitiveTopology;
pub use sdl3::gpu::RenderPass;
pub use sdl3::gpu::ShaderFormat;
pub use sdl3::gpu::SwapchainComposition;
pub use sdl3::gpu::TextureFormat;
pub use sdl3::gpu::TextureSamplerBinding;
pub use sdl3::gpu::VertexElementFormat;

pub use crate::buffer::*;
pub use crate::pipeline::*;
pub use crate::shaders::ShaderDesc;
pub use crate::shaders::ShaderRef;
pub use crate::shaders::ShaderRegistry;
pub use crate::texture::Texture;
pub use crate::vertex::VertexAttribute;
pub use crate::vertex::VertexLayout;

/// The GPU context. Owned by the app and only ever touched from the main
/// thread: SDL GPU requires swapchain operations to happen on the thread
/// that created the window, so all rendering lives there.
pub struct Gpu {
    device: sdl3::gpu::Device,
    sampler: sdl3::gpu::Sampler,
    pub shaders: ShaderRegistry,
}

impl Gpu {
    /// Only SPIR-V shaders are shipped, which pins SDL to the Vulkan backend.
    /// (D3D12/Metal would need DXIL/MSL variants of the shaders.)
    pub fn new() -> Self {
        let device = sdl3::gpu::Device::new(ShaderFormat::SPIRV, cfg!(debug_assertions))
            .expect("Failed to create GPU device");

        debug!("GPU device created (formats: {:?})", device.get_shader_formats());

        let sampler = device
            .create_sampler(
                sdl3::gpu::SamplerCreateInfo::new()
                    .with_min_filter(sdl3::gpu::Filter::Nearest)
                    .with_mag_filter(sdl3::gpu::Filter::Nearest)
                    .with_mipmap_mode(sdl3::gpu::SamplerMipmapMode::Nearest)
                    .with_address_mode_u(sdl3::gpu::SamplerAddressMode::ClampToEdge)
                    .with_address_mode_v(sdl3::gpu::SamplerAddressMode::ClampToEdge)
                    .with_address_mode_w(sdl3::gpu::SamplerAddressMode::ClampToEdge),
            )
            .expect("Failed to create sampler");

        Self {
            device,
            sampler,
            shaders: ShaderRegistry::new(),
        }
    }

    pub fn device(&self) -> &sdl3::gpu::Device {
        &self.device
    }

    pub fn load_shader(&mut self, index: usize, desc: ShaderDesc) {
        let Self {
            device, shaders, ..
        } = self;

        shaders.load(device, index, desc);
    }

    pub fn sampler(&self) -> &sdl3::gpu::Sampler {
        &self.sampler
    }

    /// Creates a swapchain for the window. Must be called on the main thread
    /// before rendering to it.
    pub fn claim_window(&mut self, window: &Window) {
        self.device = self
            .device
            .clone()
            .with_window(window)
            .expect("Failed to claim window for GPU device");

        // Prefer a linear (sRGB-view) swapchain so the shaders keep working
        // in linear space, and mailbox presentation like the wgpu renderer.
        let attempts = [
            (PresentMode::Mailbox, SwapchainComposition::SdrLinear),
            (PresentMode::Vsync, SwapchainComposition::SdrLinear),
            (PresentMode::Mailbox, SwapchainComposition::Sdr),
        ];

        for (present, composition) in attempts {
            if self
                .device
                .set_swapchain_parameters(window, present, composition)
                .is_ok()
            {
                debug!("Swapchain configured: {:?} / {:?}", present, composition);
                return;
            }
        }

        warn!("Falling back to default swapchain parameters (vsync, sdr)");
    }

    pub fn swapchain_format(&self, window: &Window) -> TextureFormat {
        self.device.get_swapchain_texture_format(window)
    }

    pub fn acquire_command_buffer(&self) -> CommandBuffer {
        self.device
            .acquire_command_buffer()
            .expect("Failed to acquire command buffer")
    }

    pub fn begin_copy_pass(&self, cmd: &CommandBuffer) -> CopyPass {
        self.device
            .begin_copy_pass(cmd)
            .expect("Failed to begin copy pass")
    }

    pub fn end_copy_pass(&self, pass: CopyPass) {
        self.device.end_copy_pass(pass);
    }
}
