mod camera;
mod color;
mod gpu;
mod shader;
mod texture;

use crate::{color::Color, gpu::gpu};
use macros::{Get, Set};
use math::Size;
use std::sync::Arc;
use winit::window::Window;

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    #[get]
    #[get(mut)]
    #[set(into)]
    clear_color: Color,
}

impl Renderer {
    #[doc(hidden)]
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let gpu = gpu();

        let surface = gpu
            .instance
            .create_surface(Arc::clone(&window))
            .expect("Failed to create surface");

        let caps = surface.get_capabilities(&gpu.adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&gpu.device, &config);

        let shader = shader::create_default_shader(&gpu.device);

        // Create a separate 1x1 white texture for untextured meshes
        let white_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("white texture bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let white_texture = Arc::new(texture::Texture::new_empty(
            "White Pixel",
            &gpu.device,
            Size::new(1, 1),
            &white_bind_group_layout,
        ));

        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &white_texture.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &[255u8, 255u8, 255u8, 255u8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let triangle_pipeline = {
            let texture_atlas = gpu.texture_atlas.load();

            Self::create_render_pipeline(
                "triangle pipeline",
                &gpu.device,
                &shader,
                &[
                    &camera.view_projection_bind_group_layout,
                    &*texture_atlas.bind_group_layout,
                ],
                format,
                wgpu::PrimitiveTopology::TriangleList,
                wgpu::PolygonMode::Fill,
            )
        };
    }
}
