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
use wgpu::{naga::FastHashMap, util::DeviceExt, wgt::DrawIndirectArgs};
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

        let mesh_data = MeshDrawData {
            geometry,
            instances: Vec::new(),
            base_instance: 0,
            dirty_ranges: Vec::new(),
            buffer_instance_offset: 0,
        };

        self.mesh_data.insert(mesh_id, mesh_data);

        self.vertex_buffer_offset += vertices.len() as u64;
        self.index_buffer_offset += indices.len() as u64;

        Ok(())
    }

    pub fn add_mesh_instance<M: Mesh>(&mut self, instance: &MeshInstance) -> Result<(), ()> {
        let mesh_id = M::id();

        if !self.mesh_data.contains_key(&mesh_id) {
            self.register_mesh::<M>()?;
        }

        if let Some(data) = self.mesh_data.get_mut(&mesh_id) {
            data.add_instance(*instance);
        }

        Ok(())
    }

    pub fn update_mesh_instance<M: Mesh>(
        &mut self,
        index: usize,
        instance: MeshInstance,
    ) -> Result<(), ()> {
        let mesh_id = M::id();

        if let Some(data) = self.mesh_data.get_mut(&mesh_id) {
            data.update_instance(index, instance)
        } else {
            Err(())
        }
    }

    pub fn remove_mesh_instance<M: Mesh>(&mut self, index: usize) -> Result<(), ()> {
        let mesh_id = M::id();

        if let Some(data) = self.mesh_data.get_mut(&mesh_id) {
            data.remove_instance(index)
        } else {
            Err(())
        }
    }

    pub fn insert_mesh_instance<M: Mesh>(
        &mut self,
        index: usize,
        instance: MeshInstance,
    ) -> Result<(), ()> {
        let mesh_id = M::id();

        if let Some(data) = self.mesh_data.get_mut(&mesh_id) {
            data.insert_instance(index, instance)
        } else {
            Err(())
        }
    }

    /// Update multiple instances at once for better performance
    fn update_mesh_instances<M: Mesh>(
        &mut self,
        updates: Vec<(usize, MeshInstance)>,
    ) -> Result<(), ()> {
        let mesh_id = M::id();

        if let Some(data) = self.mesh_data.get_mut(&mesh_id) {
            if updates.is_empty() {
                return Ok(());
            }

            // Find the range that encompasses all updates
            let min_index = updates.iter().map(|(i, _)| *i).min().unwrap();
            let max_index = updates.iter().map(|(i, _)| *i).max().unwrap();

            // Apply all updates
            for (index, instance) in updates {
                if index < data.instances.len() {
                    data.instances[index] = instance;
                } else {
                    return Err(());
                }
            }

            // Mark the entire range as dirty (more efficient than individual ranges)
            data.mark_dirty_range(min_index, max_index + 1);

            Ok(())
        } else {
            Err(())
        }
    }

    /// Optimized buffer update with granular dirty range support
    fn update_buffers(&mut self) -> Result<Vec<wgpu::util::DrawIndexedIndirectArgs>, ()> {
        let mut draw_commands = Vec::new();
        let has_dirty = self.mesh_data.values().any(|data| data.has_dirty());

        if !has_dirty {
            // No dirty instances, just rebuild draw commands
            return self.rebuild_draw_commands();
        }

        // Calculate total instances and update buffer offsets
        let mut total_instances = 0;
        let mut meshes_with_instances: Vec<_> = self
            .mesh_data
            .iter()
            .filter(|(_, data)| !data.instances.is_empty())
            .collect();

        // Sort by mesh ID for consistent ordering
        meshes_with_instances.sort_by_key(|(id, _)| *id);

        for (_, data) in &meshes_with_instances {
            total_instances += data.instances.len();
        }

        // Check if we need to resize the instance buffer
        let required_size = total_instances * std::mem::size_of::<MeshInstance>();
        if required_size as u64 > self.instance_buffer_size {
            // Reallocate buffer if needed
            self.resize_instance_buffer(required_size as u64 * 2)?; // Double for growth
        }

        // Strategy: For now, we'll do full buffer updates when any range is dirty
        // This can be optimized further with sub-buffer updates
        let mut all_instances: Vec<MeshInstance> = Vec::with_capacity(total_instances);
        let mut base_instance = 0;

        for (_, data) in &mut self.mesh_data {
            if data.instances.is_empty() {
                continue;
            }

            data.base_instance = base_instance;
            data.buffer_instance_offset = all_instances.len();
            all_instances.extend(&data.instances);

            draw_commands.push(wgpu::util::DrawIndexedIndirectArgs {
                index_count: data.geometry.index_count,
                instance_count: data.instances.len() as u32,
                first_index: data.geometry.index_offset,
                base_vertex: data.geometry.vertex_offset as i32,
                first_instance: data.base_instance,
            });

            base_instance += data.instances.len() as u32;

            // Clear dirty ranges after processing
            data.clear_dirty();
        }

        // Update the entire instance buffer
        if !all_instances.is_empty() {
            println!(
                "Updating {} instances ({} dirty ranges)",
                all_instances.len(),
                self.mesh_data
                    .values()
                    .map(|d| d.dirty_ranges.len())
                    .sum::<usize>()
            );

            self.state.queue.write_buffer(
                &self.instance_buffer,
                0,
                util::as_u8_slice(&all_instances),
            );
        }

        // Update indirect buffer
        if !draw_commands.is_empty() {
            self.state.queue.write_buffer(
                &self.indirect_buffer,
                0,
                util::as_u8_slice(&draw_commands),
            );
        }

        self.instance_buffer_offset = all_instances.len() as u64;

        Ok(draw_commands)
    }

    /// Advanced update with sub-buffer writes (for large instance counts)
    fn update_buffers_optimized(&mut self) -> Result<Vec<wgpu::util::DrawIndexedIndirectArgs>, ()> {
        let has_dirty = self.mesh_data.values().any(|data| data.has_dirty());

        if !has_dirty {
            return self.rebuild_draw_commands();
        }

        // For each mesh with dirty ranges, update only those ranges
        for (mesh_id, data) in &mut self.mesh_data {
            if data.dirty_ranges.is_empty() || data.instances.is_empty() {
                continue;
            }

            // Update each dirty range separately
            for range in &data.dirty_ranges {
                if range.is_empty() {
                    continue;
                }

                let instances_slice = &data.instances[range.start..range.end];
                let buffer_offset = (data.buffer_instance_offset + range.start)
                    * std::mem::size_of::<MeshInstance>();

                println!(
                    "Updating mesh {} range {}..{} ({} instances)",
                    mesh_id,
                    range.start,
                    range.end,
                    instances_slice.len()
                );

                self.state.queue.write_buffer(
                    &self.instance_buffer,
                    buffer_offset as u64,
                    util::as_u8_slice(instances_slice),
                );
            }

            data.clear_dirty();
        }

        self.rebuild_draw_commands()
    }

    fn rebuild_draw_commands(&mut self) -> Result<Vec<wgpu::util::DrawIndexedIndirectArgs>, ()> {
        let mut draw_commands = Vec::new();
        let mut base_instance = 0;

        for (_, data) in &self.mesh_data {
            if data.instances.is_empty() {
                continue;
            }

            draw_commands.push(wgpu::util::DrawIndexedIndirectArgs {
                index_count: data.geometry.index_count,
                instance_count: data.instances.len() as u32,
                first_index: data.geometry.index_offset,
                base_vertex: data.geometry.vertex_offset as i32,
                first_instance: base_instance,
            });

            base_instance += data.instances.len() as u32;
        }

        if !draw_commands.is_empty() {
            self.state.queue.write_buffer(
                &self.indirect_buffer,
                0,
                util::as_u8_slice(&draw_commands),
            );
        }

        Ok(draw_commands)
    }

    fn resize_instance_buffer(&mut self, new_size: u64) -> Result<(), ()> {
        println!(
            "Resizing instance buffer from {} to {} bytes",
            self.instance_buffer_size, new_size
        );

        self.instance_buffer = self.state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: new_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.instance_buffer_size = new_size;

        // Mark all meshes as dirty since the buffer was reallocated
        for data in self.mesh_data.values_mut() {
            data.mark_all_dirty();
        }

        Ok(())
    }

    pub fn clear_mesh_instances(&mut self) {
        for data in self.mesh_data.values_mut() {
            data.clear_instances();
        }
        self.instance_buffer_offset = 0;
    }

    /// Get statistics about dirty ranges
    pub fn get_dirty_stats(&self) -> HashMap<u64, (usize, usize)> {
        self.mesh_data
            .iter()
            .map(|(id, data)| (*id, (data.instances.len(), data.dirty_instance_count())))
            .collect()
    }

    #[inline]
    #[doc(hidden)]
    pub fn present(&mut self) {
        self.draw_calls = 0;

        // Use the optimized buffer update method that only updates dirty ranges
        let commands = match self.update_buffers_optimized() {
            Ok(c) => c,
            Err(_) => {
                warn!("Failed to update buffers.");
                return;
            }
        };

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
