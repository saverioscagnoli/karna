mod camera;
mod color;
mod layer;
mod mesh;
mod shader;
mod sprite;
mod text;

use crate::{
    camera::{Camera, Projection},
    layer::RenderLayer,
    mesh::{Descriptor, GpuMesh},
    text::TextRenderer,
};
use assets::AssetManager;
use macros::{Get, Set};
use math::Size;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use traccia::info;

use winit::window::Window;

// Re-exports
pub use color::Color;
pub use mesh::{Geometry, Material, Mesh, TextureKind, Transform, Vertex};
pub use shader::Shader;
pub use sprite::{Frame, Sprite};
pub use text::Text;

#[derive(Get, Set)]
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,

    #[get]
    #[set(into)]
    clear_color: Color,

    /// Asset manager
    assets: Arc<AssetManager>,

    /// Default render layers
    world: RenderLayer,
    pub ui: RenderLayer,

    // Cache window size for camera updates
    size: Size<u32>,

    triangle_pipeline: wgpu::RenderPipeline,
}

impl Deref for Renderer {
    type Target = RenderLayer;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for Renderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

impl Renderer {
    pub fn new(window: Arc<Window>, assets: Arc<AssetManager>) -> Self {
        let gpu = gpu::get();
        let size = window.inner_size();

        let surface = gpu
            .instance()
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let surface_caps = surface.get_capabilities(gpu.adapter());
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(gpu.device(), &config);

        let camera = Camera::new(Projection::Orthographic {
            left: 0.0,
            right: size.width as f32,
            bottom: size.height as f32,
            top: 0.0,
            z_near: -1.0,
            z_far: 1.0,
        });

        let ui_camera = Camera::new(Projection::Orthographic {
            left: 0.0,
            right: size.width as f32,
            bottom: 0.0,
            top: size.height as f32,
            z_near: -1.0,
            z_far: 1.0,
        });

        let shader =
            Shader::from_wgsl_file(include_str!("../../shaders/basic_2d.wgsl"), Some("shader"));

        let triangle_pipeline = shader
            .pipeline_builder()
            .label("triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc(), GpuMesh::desc()],
            );

        let text_renderer = TextRenderer::new(surface_format, &camera, &assets);
        let ui_text_renderer = TextRenderer::new(surface_format, &ui_camera, &assets);

        let world = RenderLayer::new(assets.clone(), camera, text_renderer);
        let ui = RenderLayer::new(assets.clone(), ui_camera, ui_text_renderer);

        Self {
            surface,
            config,
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            size: Size::new(size.width, size.height),
            assets,
            triangle_pipeline,
            world,
            ui,
        }
    }

    #[inline]
    /// Gets adapter information
    pub fn info() -> wgpu::AdapterInfo {
        gpu::adapter().get_info()
    }

    #[inline]
    #[doc(hidden)]
    pub fn frame_start(&mut self) {
        // Reset instance counts in all layers
        self.world.frame_start();
        self.ui.frame_start();
    }

    #[inline]
    #[doc(hidden)]
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        info!("Resized to  {}x{}", width, height);

        self.surface.configure(&gpu::device(), &self.config);
        self.world.resize(width, height);
        self.ui.resize(width, height);

        self.config.width = width;
        self.config.height = height;
        self.size.width = width;
        self.size.height = height;
    }

    #[inline]
    pub fn present(&mut self, dt: f32) -> Result<(), wgpu::SurfaceError> {
        let gpu = gpu::get();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.world.update(self.size.width, self.size.height, dt);
        self.ui.update(self.size.width, self.size.height, dt);

        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.world
                .present(&mut render_pass, &self.triangle_pipeline);
            self.ui.present(&mut render_pass, &self.triangle_pipeline);
        }

        self.world.clear();
        self.ui.clear();

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
