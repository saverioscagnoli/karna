mod camera;
mod color;
mod config;
mod draw_calls;
mod shaders;
mod state;
mod util;
mod vertex;

use crate::{
    camera::Camera, color::Color, config::RendererConfig, shaders::Shaders, state::GpuState,
    vertex::Vertex,
};
use err::RendererError;
use math::Size;
use std::sync::Arc;
use traccia::{info, warn};
use wgpu::{util::DeviceExt, wgt::DrawIndirectArgs};
use winit::window::Window;

pub mod render {
    pub use crate::Renderer;
    pub use crate::color::Color;
}

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub struct Renderer {
    state: GpuState,
    config: RendererConfig,
    camera: Camera,
    clear_color: Color,
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    indirect_buffer: wgpu::Buffer,
}

impl Renderer {
    pub async fn _new(window: Arc<Window>) -> Result<Self, RendererError> {
        let (state, wgpu_config) = GpuState::new(window).await?;
        let config = RendererConfig::new(wgpu_config);
        let camera = Camera::new(
            &state.device,
            state.queue.clone(),
            Size {
                width: config.width,
                height: config.height,
            },
        );

        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: util::as_u8_slice(&[
                    // Triangle 1 (vertices 0-2)
                    Vertex {
                        position: [10.0, 10.0, 0.0].into(),
                        color: Color::RED.into(),
                    },
                    Vertex {
                        position: [100.0, 10.0, 0.0].into(),
                        color: Color::GREEN.into(),
                    },
                    Vertex {
                        position: [55.0, 100.0, 0.0].into(),
                        color: Color::BLUE.into(),
                    },
                    // Triangle 2 (vertices 3-5)
                    Vertex {
                        position: [120.0, 10.0, 0.0].into(),
                        color: Color::YELLOW.into(),
                    },
                    Vertex {
                        position: [200.0, 10.0, 0.0].into(),
                        color: Color::MAGENTA.into(),
                    },
                    Vertex {
                        position: [160.0, 100.0, 0.0].into(),
                        color: Color::CYAN.into(),
                    },
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Shaders::basic().into()),
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

        let draws: Vec<DrawIndirectArgs> = vec![
            DrawIndirectArgs {
                vertex_count: 3, // First triangle
                instance_count: 1,
                first_vertex: 0,
                first_instance: 0,
            },
            DrawIndirectArgs {
                vertex_count: 3, // Second triangle
                instance_count: 1,
                first_vertex: 3,
                first_instance: 0,
            },
        ];

        let indirect_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("indirect args"),
                contents: util::as_u8_slice(&draws),
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            });

        Ok(Self {
            state,
            indirect_buffer,
            config,
            camera,
            clear_color: Color::BLACK,
            vertex_buffer,
            render_pipeline,
        })
    }

    #[inline]
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.state.adapter.get_info()
    }

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

    pub fn present(&mut self) {
        let frame = match self.state.surface.get_current_texture() {
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
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                label: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline);

            // Use multi-indirect drawing instead of direct draw
            render_pass.multi_draw_indirect(&self.indirect_buffer, 0, 2);
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
