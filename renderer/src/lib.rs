mod camera;
mod color;
mod mesh;

use crate::{
    camera::{Camera, Projection},
    mesh::dirty::DirtyTracked,
};

// Re-exports
pub use crate::color::Color;
pub use crate::mesh::transform::Transform2D;
pub use crate::mesh::*;

use common::utils;
use macros::{Get, Set};
use math::Size;
use std::sync::Arc;
use wgpu::{PipelineCompilationOptions, naga::FastHashMap, util::DeviceExt};

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter: wgpu::Adapter,

    #[allow(unused)]
    surface_format: wgpu::TextureFormat,
    config: wgpu::SurfaceConfiguration,

    pub camera: DirtyTracked<Camera>,

    #[get(copied)]
    #[set(into)]
    clear_color: Color,
    window_size: Size<u32>,

    meshes: FastHashMap<u32, MeshBuffer>,
    point_pipeline: wgpu::RenderPipeline,
    triangle_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    #[doc(hidden)]
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let (width, height) = window.inner_size().into();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to fetch adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::defaults(),
                label: Some("device"),
                required_features: wgpu::Features::default(),
                ..Default::default()
            })
            .await
            .expect("Failed to request device");

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let camera = Camera::new(
            &device,
            Projection::Orthographic {
                left: 0.0,
                right: width as f32,
                bottom: height as f32,
                top: 0.0,
                z_near: -1.0,
                z_far: 1.0,
            },
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/basic.wgsl").into()),
        });

        let point_pipeline = Self::create_render_pipeline(
            "point pipeline",
            &device,
            &shader,
            &[&camera.view_projection_bind_group_layout],
            surface_format,
            wgpu::PrimitiveTopology::PointList,
            wgpu::PolygonMode::Fill,
        );

        let triangle_pipeline = Self::create_render_pipeline(
            "mesh pipeline",
            &device,
            &shader,
            &[&camera.view_projection_bind_group_layout],
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
        );

        Self {
            surface,
            device,
            queue,
            adapter,
            surface_format,
            config,
            camera: camera.into(),
            clear_color: Color::default(),
            window_size: Size::new(width, height),
            meshes: FastHashMap::default(),
            triangle_pipeline,
            point_pipeline,
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
                buffers: &[Vertex::desc(), MeshInstanceGPU::desc()], // Both layouts
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
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

    fn register_mesh(&mut self, mesh: &Mesh) {
        if self.meshes.contains_key(&mesh.geometry.id) {
            return;
        }

        let index_count = mesh.geometry.indices.len() as u32;

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!(
                    "Mesh with id '{:?}' vertex buffer",
                    mesh.geometry.id
                )),
                contents: utils::as_u8_slice(&mesh.geometry.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!(
                    "Mesh with id '{:?}' index buffer",
                    mesh.geometry.id
                )),
                contents: utils::as_u8_slice(&mesh.geometry.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (std::mem::size_of::<MeshInstanceGPU>() * Mesh::INITIAL_INSTANCE_CAPACITY) as u64,
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

        self.meshes.insert(mesh.geometry.id, mesh_buffer);
    }

    #[inline]
    pub(crate) fn draw_instance(&mut self, mesh: &Mesh) {
        if !self.meshes.contains_key(&mesh.geometry.id) {
            self.register_mesh(mesh);
        }

        self.meshes
            .get_mut(&mesh.geometry.id)
            .unwrap()
            .instances
            .push(mesh.instance.to_gpu());
    }

    #[inline]
    #[doc(hidden)]
    pub fn resize(&mut self, size: Size<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera.update(&size, &self.queue);
        self.window_size = size;
    }

    #[inline]
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    #[inline]
    #[doc(hidden)]
    pub fn present(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        if self.camera.dirty {
            self.camera.update(&self.window_size, &self.queue);
            self.camera.dirty = false;
        }

        for mesh_buffer in self.meshes.values_mut() {
            if mesh_buffer.instances.is_empty() {
                continue;
            }

            let instance_data = utils::as_u8_slice(&mesh_buffer.instances);
            let required_size = instance_data.len() as u64;

            if required_size > mesh_buffer.instance_buffer.size() {
                mesh_buffer.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("instance buffer"),
                    size: required_size,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            self.queue
                .write_buffer(&mesh_buffer.instance_buffer, 0, instance_data);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
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

            render_pass.set_bind_group(0, &self.camera.view_projection_bind_group, &[]);

            let mut topology_groups: FastHashMap<wgpu::PrimitiveTopology, Vec<&MeshBuffer>> =
                FastHashMap::default();

            for mesh_buffer in self.meshes.values() {
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
                    wgpu::PrimitiveTopology::PointList => &self.point_pipeline,
                    wgpu::PrimitiveTopology::TriangleList => &self.triangle_pipeline,
                    _ => todo!("?"),
                };

                render_pass.set_pipeline(pipeline);

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

        self.queue.submit([encoder.finish()]);
        frame.present();

        for mesh_buffer in self.meshes.values_mut() {
            mesh_buffer.instances.clear();
        }
    }
}
