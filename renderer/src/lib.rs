mod camera;
mod color;
mod gpu;
mod mesh;
mod shader;

use common::utils;
use macros::{Get, Set};
use math::Size;
use mesh::RawMesh;
use std::sync::Arc;
use traccia::{info, warn};
use wgpu::{Surface, SurfaceConfiguration, naga::FastHashMap, util::DeviceExt};
use winit::window::Window;

// Re-exports
pub use crate::camera::{Camera, Projection};
pub use color::Color;
pub use gpu::*;
pub use mesh::{
    Descriptor, Mesh, MeshBuffer, Vertex, geometry::MeshGeometry, transform::Transform,
};
pub use shader::*;

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Renderer {
    gpu: Arc<GPU>,
    surface: Surface<'static>,
    config: SurfaceConfiguration,

    #[get]
    #[set(into)]
    clear_color: Color,

    camera: Camera,
    triangle_pipeline: wgpu::RenderPipeline,

    mesh_cache: FastHashMap<u32, MeshBuffer>,
}

impl Renderer {
    pub fn new(gpu: Arc<GPU>, window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let surface = gpu
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let caps = surface.get_capabilities(&gpu.adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
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

        let camera = Camera::new(
            &gpu.device,
            Projection::Orthographic {
                left: 0.0,
                right: size.width as f32,
                bottom: size.height as f32,
                top: 0.0,
                z_near: -1.0,
                z_far: 1.0,
            },
        );

        let shader = shader::create_default_shader(&gpu.device);

        let triangle_pipeline = Self::create_render_pipeline(
            "triangle pipeline",
            &gpu.device,
            &shader,
            &[&camera.view_projection_bind_group_layout],
            format,
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
        );

        Self {
            gpu,
            surface,
            config,
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            camera,
            triangle_pipeline,
            mesh_cache: FastHashMap::default(),
        }
    }

    fn create_render_pipeline<L: AsRef<str>>(
        label: L,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        surface_format: wgpu::TextureFormat,
        topology: wgpu::PrimitiveTopology,
        polygon_mode: wgpu::PolygonMode,
    ) -> wgpu::RenderPipeline {
        let label = label.as_ref();
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), RawMesh::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode,
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

    #[inline]
    fn register_mesh(&mut self, mesh: &Mesh) {
        let index_count = mesh.geometry.indices.len() as u32;
        let vertex_buffer = self
            .gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Mesh id '{:?}' vertex buffer", mesh.geometry.id)),
                contents: utils::as_u8_slice(&mesh.geometry.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = self
            .gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Mesh id '{:?}' index buffer", mesh.geometry.id)),
                contents: utils::as_u8_slice(&mesh.geometry.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let instance_buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (std::mem::size_of::<RawMesh>() * Mesh::INITIAL_INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mesh_buffer = MeshBuffer {
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer,
            instances: Vec::new(),
            topology: mesh.geometry.topology,
        };

        self.mesh_cache.insert(mesh.geometry.id, mesh_buffer);
    }

    #[inline]
    pub fn draw_mesh(&mut self, mesh: &Mesh) {
        if !self.mesh_cache.contains_key(&mesh.geometry.id) {
            self.register_mesh(mesh);
        }

        self.mesh_cache
            .get_mut(&mesh.geometry.id)
            .unwrap()
            .instances
            .push(mesh.to_raw());
    }

    #[inline]
    pub fn resize(&mut self, size: Size<u32>) {
        if size.width == 0 || size.height == 0 {
            warn!("cannot set witdth or height to 0");
            return;
        }

        info!("Resizing window to {}x{}", size.width, size.height);

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.gpu.device, &self.config);

        self.camera.update(&size, &self.gpu.queue);
    }

    #[inline]
    pub fn present(&mut self, gpu: &GPU) -> Result<(), wgpu::SurfaceError> {
        // Write instance data to GPU buffers and resize if needed
        for mesh_buffer in self.mesh_cache.values_mut() {
            if mesh_buffer.instances.is_empty() {
                continue;
            }

            let instance_count = mesh_buffer.instances.len();
            let required_size = (std::mem::size_of::<RawMesh>() * instance_count) as u64;
            let current_size = mesh_buffer.instance_buffer.size();

            // Resize buffer if needed
            if required_size > current_size {
                let new_capacity = instance_count.next_power_of_two();
                mesh_buffer.instance_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("instance buffer"),
                    size: (std::mem::size_of::<RawMesh>() * new_capacity) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            // Write instance data to GPU
            gpu.queue.write_buffer(
                &mesh_buffer.instance_buffer,
                0,
                utils::as_u8_slice(&mesh_buffer.instances),
            );
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
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

            let mut topology_groups: FastHashMap<wgpu::PrimitiveTopology, Vec<&MeshBuffer>> =
                FastHashMap::default();

            for mesh_buffer in self.mesh_cache.values() {
                if mesh_buffer.instances.is_empty() {
                    continue;
                }

                topology_groups
                    .entry(mesh_buffer.topology)
                    .or_default()
                    .push(mesh_buffer);
            }

            for (topology, buffers) in topology_groups {
                let pipeline = match topology {
                    wgpu::PrimitiveTopology::TriangleList => &self.triangle_pipeline,
                    _ => todo!("?"),
                };

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &self.camera.view_projection_bind_group, &[]);

                for mesh_buffer in buffers {
                    let instance_count = mesh_buffer.instances.len() as u32;

                    render_pass.set_vertex_buffer(0, mesh_buffer.vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, mesh_buffer.instance_buffer.slice(..));
                    render_pass.set_index_buffer(
                        mesh_buffer.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    render_pass.draw_indexed(0..mesh_buffer.index_count, 0, 0..instance_count);
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Clear instances after rendering
        for mesh_buffer in self.mesh_cache.values_mut() {
            mesh_buffer.instances.clear();
        }

        Ok(())
    }
}
