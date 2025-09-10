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
    mesh::{MeshDrawData, MeshGeometry, MeshInstance},
    shaders::Shaders,
    state::GpuState,
};
use err::RendererError;
use globals::{
    DEFAULT_INDEX_BUFFER_SIZE, DEFAULT_INDIRECT_BUFFER_SIZE, DEFAULT_INSTANCE_BUFFER_SIZE,
    DEFAULT_VERTEX_BUFFER_SIZE,
};
use math::Size;
use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
    sync::Arc,
};
use traccia::{info, trace, warn};
use wgpu::{
    naga::FastHashMap,
    util::DeviceExt,
    wgt::{DrawIndexedIndirectArgs, DrawIndirectArgs},
};
use winit::window::Window;

// Re-exports
pub use color::Color;
pub use mesh::Mesh;
pub use shapes::rect::Rect;

#[cfg(feature = "imgui")]
pub mod imgui {
    pub use ::imgui::{Condition, Ui};
}

#[cfg(feature = "imgui")]
use imgui_state::ImguiState;

pub struct Renderer {
    // Essentials
    state: GpuState,
    config: RendererConfig,
    camera: Camera,
    clear_color: Color,

    // Pipelines
    point_pipeline: wgpu::RenderPipeline,
    triangle_pipeline: wgpu::RenderPipeline,

    // Buffers
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,

    // Buffer management
    vertex_buffer_size: u64,
    index_buffer_size: u64,
    instance_buffer_size: u64,
    indirect_buffer_size: u64,

    // Buffer offsets
    vertex_buffer_offset: u64,
    index_buffer_offset: u64,
    instance_buffer_offset: u64,

    // Mesh data
    mesh_geometries: FastHashMap<u64, MeshGeometry>,
    mesh_data: FastHashMap<u64, MeshDrawData>,

    // Metrics
    draw_calls: u32,

    #[cfg(feature = "imgui")]
    pub imgui: ImguiState,
}

impl Deref for Renderer {
    type Target = RendererConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for Renderer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl Renderer {
    fn create_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera: &Camera,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[camera.bind_group_layout()],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), MeshInstance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
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
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
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
        })
    }

    #[doc(hidden)]
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let window_clone = window.clone();
        let (state, wgpu_config) = GpuState::new(window_clone).await?;
        let config = RendererConfig::new(state.surface.clone(), state.device.clone(), wgpu_config);
        let camera = Camera::new(
            &state.device,
            state.queue.clone(),
            Size {
                width: config.wgpu().width,
                height: config.wgpu().height,
            },
            CameraType::Orthographic,
        );

        // Compute initial buffer sizes
        let vertex_buffer_size = std::mem::size_of::<Vertex>() as u64 * DEFAULT_VERTEX_BUFFER_SIZE;
        let index_buffer_size = std::mem::size_of::<u16>() as u64 * DEFAULT_INDEX_BUFFER_SIZE;
        let instance_buffer_size =
            std::mem::size_of::<MeshInstance>() as u64 * DEFAULT_INSTANCE_BUFFER_SIZE;
        let indirect_buffer_size =
            std::mem::size_of::<DrawIndirectArgs>() as u64 * DEFAULT_INDIRECT_BUFFER_SIZE;

        let vertex_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: vertex_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: index_buffer_size,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: instance_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indirect_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Indirect Buffer"),
            size: indirect_buffer_size,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Shaders::basic().into()),
            });

        let point_pipeline = Self::create_pipeline(&state.device, config.wgpu(), &camera, &shader);
        let triangle_pipeline =
            Self::create_pipeline(&state.device, config.wgpu(), &camera, &shader);

        #[cfg(feature = "imgui")]
        let imgui = ImguiState::new(
            window.clone(),
            window.scale_factor() as f32,
            state.device.clone(),
            state.queue.clone(),
            config.wgpu().format,
        );

        Ok(Self {
            state,
            config,
            camera,
            clear_color: Color::BLACK,
            point_pipeline,
            triangle_pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            indirect_buffer,
            vertex_buffer_size,
            index_buffer_size,
            instance_buffer_size,
            indirect_buffer_size,
            vertex_buffer_offset: 0,
            index_buffer_offset: 0,
            instance_buffer_offset: 0,
            mesh_geometries: FastHashMap::default(),
            mesh_data: FastHashMap::default(),
            draw_calls: 0,
            #[cfg(feature = "imgui")]
            imgui,
        })
    }

    #[inline]
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.state.adapter.get_info()
    }

    #[inline]
    pub fn draw_calls(&self) -> u32 {
        self.draw_calls
    }

    #[inline]
    pub fn resize(&mut self, size: Size<u32>) {
        if size.width <= 0 || size.height <= 0 {
            warn!("Attempted to resize renderer to zero dimension");
            return;
        }

        self.camera.update_projection(size);

        self.config.resize(size.width, size.height);
        info!("Resized renderer viewport to {:?}", size);
    }

    #[inline]
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn register_mesh<M: Mesh>(&mut self) -> Result<(), ()> {
        let mesh_id = M::id();

        if self.mesh_geometries.contains_key(&mesh_id) {
            trace!("Mesh already registered: {}", mesh_id);
            return Ok(());
        }

        let vertices = M::vertices();
        let indices = M::indices();

        let vertex_data_size = (vertices.len() * std::mem::size_of::<Vertex>()) as u64;
        let index_data_size = (indices.len() * std::mem::size_of::<u16>()) as u64;

        if (self.vertex_buffer_offset * std::mem::size_of::<Vertex>() as u64 + vertex_data_size)
            > self.vertex_buffer_size
        {
            return Err(());
        }

        let vertex_offset = self.vertex_buffer_offset;

        self.state.queue.write_buffer(
            &self.vertex_buffer,
            vertex_offset * std::mem::size_of::<Vertex>() as u64,
            util::as_u8_slice(&vertices),
        );

        let adjusted_indices = indices
            .iter()
            .map(|&i| i + vertex_offset as u16)
            .collect::<Vec<_>>();

        let index_offset = self.index_buffer_offset;

        self.state.queue.write_buffer(
            &self.index_buffer,
            index_offset as u64 * std::mem::size_of::<u16>() as u64,
            util::as_u8_slice(&adjusted_indices),
        );

        let geometry = MeshGeometry {
            vertex_offset: vertex_offset as u32,
            vertex_count: vertices.len() as u32,
            index_offset: index_offset as u32,
            index_count: indices.len() as u32,
        };

        self.mesh_geometries.insert(mesh_id, geometry);

        let mesh_data = MeshDrawData::new(geometry);

        self.mesh_data.insert(mesh_id, mesh_data);

        self.vertex_buffer_offset += vertices.len() as u64;
        self.index_buffer_offset += indices.len() as u64;

        Ok(())
    }

    pub(crate) fn upsert_mesh_instance<M: Mesh>(&mut self, instance: &mut MeshInstance) {
        let mesh_id = M::id();

        if !self.mesh_geometries.contains_key(&mesh_id) {
            match self.register_mesh::<M>() {
                Ok(_) => info!("Registered new mesh: {}", mesh_id),
                Err(_) => {
                    warn!("Failed to register mesh: {}", mesh_id);
                    return;
                }
            }
        }

        let mesh_data = match self.mesh_data.get_mut(&mesh_id) {
            Some(data) => data,
            None => {
                warn!("Mesh data not found for mesh id: {}", mesh_id);
                return;
            }
        };

        mesh_data.instances.push(instance.clone());

        self.state.queue.write_buffer(
            &self.instance_buffer,
            0,
            util::as_u8_slice(&mesh_data.instances),
        );
    }

    fn generate_draw_commands(&self) -> Vec<DrawIndexedIndirectArgs> {
        let mut commands = Vec::new();

        for mesh_data in self.mesh_data.values() {
            if mesh_data.instances.is_empty() {
                continue;
            }

            commands.push(DrawIndexedIndirectArgs {
                index_count: mesh_data.geometry.index_count,
                instance_count: mesh_data.instances.len() as u32,
                first_index: mesh_data.geometry.index_offset,
                base_vertex: mesh_data.geometry.vertex_offset as i32,
                first_instance: mesh_data.base_instance,
            });
        }

        self.state
            .queue
            .write_buffer(&self.indirect_buffer, 0, util::as_u8_slice(&commands));

        commands
    }

    #[inline]
    #[doc(hidden)]
    pub fn present(&mut self) {
        self.draw_calls = 0;

        // Use the optimized buffer update method that only updates dirty ranges
        let commands = self.generate_draw_commands();

        println!("Draw commands: {:?}", commands);

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

            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            render_pass.set_pipeline(&self.triangle_pipeline);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            if !commands.is_empty() {
                render_pass.multi_draw_indexed_indirect(
                    &self.indirect_buffer,
                    0,
                    commands.len() as u32,
                );

                self.draw_calls = 1;
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
