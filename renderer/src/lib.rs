mod camera;
mod color;
mod config;
mod draw_calls;
#[cfg(feature = "imgui")]
mod imgui_state;
mod shaders;
mod state;
mod subrenderers;
mod util;
mod vertex;

use crate::{
    camera::Camera,
    color::Color,
    config::RendererConfig,
    draw_calls::DrawIndirectArgs,
    shaders::Shaders,
    state::GpuState,
    subrenderers::{RectInstance, RectRenderer},
    vertex::Vertex,
};
use err::RendererError;
use math::{Size, Vec2};
use std::sync::Arc;
use traccia::{info, warn};
use winit::window::Window;

// Re-exports
#[cfg(feature = "imgui")]
pub mod imgui {
    pub use ::imgui::{Condition, Ui};
}

#[cfg(feature = "imgui")]
use imgui_state::ImguiState;

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub struct Rect {
    pub position: Vec2,
    pub width: f32,
    pub height: f32,
    pub color: Color,
}

impl Rect {
    pub fn new(position: Vec2, width: f32, height: f32, color: Color) -> Self {
        Self {
            position,
            width,
            height,
            color,
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        let instance = RectInstance {
            position: self.position,
            scale: Vec2::new(self.width, self.height),
            color: self.color.into(),
        };

        renderer.rect_renderer.instances.push(instance);
    }
}

pub struct Renderer {
    state: GpuState,
    config: RendererConfig,
    camera: Camera,
    clear_color: Color,
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    indirect_buffer: wgpu::Buffer,

    rect_renderer: RectRenderer,

    #[cfg(feature = "imgui")]
    pub imgui: ImguiState,
}

impl Renderer {
    #[doc(hidden)]
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let window_clone = window.clone();
        let (state, wgpu_config) = GpuState::new(window_clone).await?;
        let config = RendererConfig::new(wgpu_config);
        let camera = Camera::new(
            &state.device,
            state.queue.clone(),
            Size {
                width: config.width,
                height: config.height,
            },
        );

        let vertex_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: std::mem::size_of::<Vertex>() as u64 * 10_000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Shaders::basic().into()),
            });

        let rect_shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Rect Shader"),
                source: wgpu::ShaderSource::Wgsl(Shaders::rect().into()),
            });

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[camera.bind_group_layout()],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        // Create rect pipeline
        let indirect_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Draw Buffer"),
            size: std::mem::size_of::<DrawIndirectArgs>() as u64 * 100, // Support up to 100 draw calls
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rect_renderer = RectRenderer::new(
            &state.device,
            &render_pipeline_layout,
            &rect_shader,
            config.format,
        );

        #[cfg(feature = "imgui")]
        let imgui = ImguiState::new(
            window.clone(),
            window.scale_factor() as f32,
            state.device.clone(),
            state.queue.clone(),
            config.format,
        );

        Ok(Self {
            state,
            indirect_buffer,
            config,
            camera,
            clear_color: Color::BLACK,
            vertex_buffer,
            render_pipeline,
            rect_renderer,
            #[cfg(feature = "imgui")]
            imgui,
        })
    }

    #[inline]
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.state.adapter.get_info()
    }

    #[inline]
    pub fn resize(&mut self, size: Size<u32>) {
        if size.width <= 0 || size.height <= 0 {
            warn!("Attempted to resize renderer to zero dimension");
            return;
        }

        self.camera.update_projection(size);

        self.config.width = size.width;
        self.config.height = size.height;
        self.state
            .surface
            .configure(&self.state.device, &self.config);

        info!("Resized renderer viewport to {:?}", size);
    }

    #[inline]
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    #[inline]
    #[doc(hidden)]
    pub fn present(&mut self) {
        let frame: wgpu::SurfaceTexture = match self.state.surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to acquire next swap chain texture: {:?}", e);
                return;
            }
        };

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render command encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            #[cfg(feature = "imgui")]
            if let Err(e) = self.imgui.renderer.render(
                self.imgui.context.render(),
                &self.state.queue,
                &self.state.device,
                &mut render_pass,
            ) {
                warn!("Failed to render imgui frame: {}", e);
            }

            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);

            // Render your regular content
            self.rect_renderer
                .flush(&self.state.queue, &self.indirect_buffer, &mut render_pass);
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
