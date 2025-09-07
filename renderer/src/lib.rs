mod camera;
mod color;
mod config;
mod fundamentals;
mod mesh;
mod shaders;
mod shapes;
mod state;
mod util;

#[cfg(feature = "imgui")]
mod imgui_state;

use crate::{
    camera::{Camera, CameraType},
    config::RendererConfig,
    fundamentals::{Descriptor, Vertex},
    mesh::{Mesh, MeshData, MeshInstance},
    shaders::Shaders,
    state::GpuState,
};
use err::RendererError;
use math::Size;
use std::{collections::HashMap, sync::Arc};
use traccia::{info, trace, warn};
use wgpu::{util::DeviceExt, wgt::DrawIndirectArgs};
use winit::window::Window;

// Re-exports
pub use color::Color;
pub use shapes::rect::Rect;

#[cfg(feature = "imgui")]
pub mod imgui {
    pub use ::imgui::{Condition, Ui};
}

#[cfg(feature = "imgui")]
use imgui_state::ImguiState;

pub struct Renderer {
    state: GpuState,
    config: RendererConfig,
    camera: Camera,
    clear_color: Color,
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    indirect_buffer: wgpu::Buffer,

    mesh_data: HashMap<u64, MeshData>, // Maps mesh id to its data

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
            CameraType::Orthographic,
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
                        buffers: &[Vertex::desc(), MeshInstance::desc()],
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
            mesh_data: HashMap::new(),
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

    pub fn add_mesh_instance<M: Mesh>(&mut self, mut instance: MeshInstance) {
        let mesh_id = M::id();

        // Get or create mesh data
        let mesh_data = self.mesh_data.entry(mesh_id).or_insert_with(|| {
            let vertices = M::vertices();
            let indices = M::indices();

            let vertex_buffer =
                self.state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Vertex Buffer {}", mesh_id)),
                        contents: util::as_u8_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

            let index_buffer =
                self.state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Index Buffer {}", mesh_id)),
                        contents: util::as_u8_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

            MeshData {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
                instances: Vec::new(),
                instance_buffer: None,
            }
        });

        // instance.translation.x += instance.scale.x / 2.0;
        // instance.translation.y += instance.scale.y / 2.0;

        // Add instance
        mesh_data.instances.push(instance);
    }

    // Call this before rendering to update instance buffers
    pub fn update_instance_buffers(&mut self) {
        for (mesh_id, mesh_data) in &mut self.mesh_data {
            if mesh_data.instances.is_empty() {
                continue;
            }

            let instance_count = mesh_data.instances.len();

            // Always create instance buffer for consistency
            let instance_buffer =
                self.state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Instance Buffer {}", mesh_id)),
                        contents: util::as_u8_slice(&mesh_data.instances),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });

            mesh_data.instance_buffer = Some(instance_buffer);
        }
    }

    // Clear all mesh instances (call at the beginning of each frame)
    pub fn clear_mesh_instances(&mut self) {
        for mesh_data in self.mesh_data.values_mut() {
            mesh_data.instances.clear();
            mesh_data.instance_buffer = None;
        }
    }

    // Get instance count for a specific mesh type
    pub fn get_instance_count<M: Mesh>(&self) -> usize {
        let mesh_id = M::id();
        self.mesh_data
            .get(&mesh_id)
            .map(|data| data.instances.len())
            .unwrap_or(0)
    }

    // Remove all instances of a specific mesh type
    pub fn clear_mesh_type<M: Mesh>(&mut self) {
        let mesh_id = M::id();
        if let Some(mesh_data) = self.mesh_data.get_mut(&mesh_id) {
            mesh_data.instances.clear();
            mesh_data.instance_buffer = None;
        }
    }

    #[inline]
    #[doc(hidden)]
    pub fn present(&mut self) {
        self.update_instance_buffers();

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
                })],
                depth_stencil_attachment: None,
                label: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);

            // Render all meshes
            for (mesh_id, mesh_data) in &self.mesh_data {
                if mesh_data.instances.is_empty() {
                    continue;
                }

                render_pass.set_vertex_buffer(0, mesh_data.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                let instance_count = mesh_data.instances.len() as u32;

                if let Some(instance_buffer) = &mesh_data.instance_buffer {
                    render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

                    if instance_count > 1 {
                        // Instanced rendering - single draw call for multiple instances
                        render_pass.draw_indexed(0..mesh_data.index_count, 0, 0..instance_count);
                        trace!(
                            "Rendered {} instances of mesh {} in single draw call",
                            instance_count, mesh_id
                        );
                    } else {
                        // Single instance
                        render_pass.draw_indexed(0..mesh_data.index_count, 0, 0..1);
                    }
                }
            }

            #[cfg(feature = "imgui")]
            if let Err(e) = self.imgui.renderer.render(
                self.imgui.context.render(),
                &self.state.queue,
                &self.state.device,
                &mut render_pass,
            ) {
                warn!("Failed to render imgui frame: {}", e);
            }
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
