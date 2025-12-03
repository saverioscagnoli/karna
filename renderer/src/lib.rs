mod atlas;
mod camera;
mod color;
mod mesh;

use crate::{
    camera::{Camera, Projection},
    mesh::{
        dirty::DirtyTracked,
        material::{Material, Texture},
    },
};

// Re-exports
pub use crate::atlas::{AtlasRegion, TextureAtlas};
pub use crate::color::Color;
pub use crate::mesh::transform::Transform2D;
pub use crate::mesh::*;

use common::utils;
use macros::{Get, Set};
use math::Size;
use std::sync::{Arc, RwLock};
use wgpu::{PipelineCompilationOptions, naga::FastHashMap, util::DeviceExt};

#[derive(Debug)]
pub struct SharedGPU {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub default_white_texture: Arc<Texture>,
    pub atlas: RwLock<TextureAtlas>,
}

impl SharedGPU {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to fetch adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::default(),
                label: Some("shared device"),
                required_features: wgpu::Features::default(),
                ..Default::default()
            })
            .await
            .expect("Failed to request device");

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture bind group layout"),
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

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([255, 255, 255, 255]),
        ));

        let default_white_texture = Arc::new(
            Texture::from_image(
                &device,
                &queue,
                &img,
                Some("default white"),
                &texture_bind_group_layout,
            )
            .unwrap(),
        );

        let atlas = RwLock::new(TextureAtlas::new(&device, Size::new(2048, 2048)));

        Self {
            instance,
            device,
            queue,
            adapter,
            texture_bind_group_layout,
            default_white_texture,
            atlas,
        }
    }

    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub fn load_atlas_image(
        &self,
        name: String,
        bytes: &[u8],
    ) -> Result<crate::AtlasRegion, String> {
        self.atlas
            .write()
            .unwrap()
            .load_image(&self.queue, name, bytes)
    }

    pub fn get_atlas_region(&self, name: &str) -> Option<crate::AtlasRegion> {
        self.atlas.read().unwrap().get_region(name)
    }

    pub fn get_atlas_uv(&self, name: &str) -> Option<(math::Vector2, math::Vector2)> {
        self.atlas.read().unwrap().get_uv(name)
    }

    pub fn create_texture(&self, bytes: &[u8]) -> Result<Arc<Texture>, image::ImageError> {
        Ok(Arc::new(Texture::from_bytes(
            &self.device,
            &self.queue,
            bytes,
            "image",
            &self.texture_bind_group_layout,
        )?))
    }
}

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Renderer {
    gpu: Arc<SharedGPU>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    config: wgpu::SurfaceConfiguration,

    pub camera: DirtyTracked<Camera>,

    #[get(copied)]
    #[set(into)]
    clear_color: Color,
    window_size: Size<u32>,

    msaa_texture: wgpu::Texture,
    msaa_view: wgpu::TextureView,

    meshes: FastHashMap<(u32, Option<usize>), MeshBuffer>,

    point_pipeline: wgpu::RenderPipeline,
    triangle_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
    pub fn new(window: Arc<winit::window::Window>, gpu: Arc<SharedGPU>) -> Self {
        let (width, height) = window.inner_size().into();

        // Use the shared instance
        let surface = gpu
            .instance
            .create_surface(window)
            .expect("Failed to create surface");

        let surface_capabilities = surface.get_capabilities(&gpu.adapter);
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

        surface.configure(&gpu.device, &config);

        let camera = Camera::new(
            &gpu.device,
            Projection::Orthographic {
                left: 0.0,
                right: width as f32,
                bottom: height as f32,
                top: 0.0,
                z_near: -1.0,
                z_far: 1.0,
            },
        );

        let msaa_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/basic.wgsl").into()),
            });

        let point_pipeline = Self::create_render_pipeline(
            "point pipeline",
            &gpu.device,
            &shader,
            &[
                &camera.view_projection_bind_group_layout,
                &gpu.texture_bind_group_layout,
            ],
            surface_format,
            wgpu::PrimitiveTopology::PointList,
            wgpu::PolygonMode::Fill,
        );

        let triangle_pipeline = Self::create_render_pipeline(
            "mesh pipeline",
            &gpu.device,
            &shader,
            &[
                &camera.view_projection_bind_group_layout,
                &gpu.texture_bind_group_layout,
            ],
            surface_format,
            wgpu::PrimitiveTopology::TriangleList,
            wgpu::PolygonMode::Fill,
        );

        Self {
            gpu,
            surface,
            surface_format,
            config,
            camera: camera.into(),
            clear_color: Color::default(),
            window_size: Size::new(width, height),
            msaa_texture,
            msaa_view,
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
                buffers: &[Vertex::desc(), MeshInstanceGPU::desc()],
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
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    fn get_batch_id(mesh: &Mesh) -> (u32, Option<usize>) {
        let geometry_id = mesh.geometry.id;
        let texture_id = mesh
            .material
            .texture
            .as_ref()
            .map(|t| Arc::as_ptr(t) as usize);
        (geometry_id, texture_id)
    }

    fn register_mesh(&mut self, mesh: &Mesh) {
        let batch_id = Self::get_batch_id(mesh);

        if self.meshes.contains_key(&batch_id) {
            return;
        }

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
            material: mesh.material.clone(),
        };

        self.meshes.insert(batch_id, mesh_buffer);
    }

    #[inline]
    pub(crate) fn draw_instance(&mut self, mesh: &Mesh) {
        let batch_id = Self::get_batch_id(mesh);

        if !self.meshes.contains_key(&batch_id) {
            self.register_mesh(mesh);
        }

        self.meshes
            .get_mut(&batch_id)
            .unwrap()
            .instances
            .push(mesh.instance.to_gpu());
    }

    pub fn render_atlas_debug(&mut self) {
        let atlas = self.gpu.atlas.read().unwrap();
        let atlas_size = atlas.size;
        let atlas_texture = atlas.texture.clone();
        drop(atlas); // Release the lock before draw_instance

        let geometry = MeshGeometry::rect();
        let material = Material {
            texture: Some(atlas_texture),
            ..Default::default()
        };

        let mesh = Mesh::new(
            geometry,
            material,
            Transform2D {
                position: math::Vector2::new(0.0, 0.0),
                scale: math::Vector2::new(atlas_size.width as f32, atlas_size.height as f32),
                rotation: 0.0,
            },
        );

        self.draw_instance(&mesh);
    }

    #[inline]
    pub fn resize(&mut self, size: Size<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.gpu.device, &self.config);
        self.camera.update(&size, &self.gpu.queue);
        self.window_size = size;

        self.msaa_texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.msaa_view = self
            .msaa_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    #[inline]
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.gpu.adapter.get_info()
    }

    #[inline]
    pub fn present(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };

        let output = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        if self.camera.dirty {
            self.camera.update(&self.window_size, &self.gpu.queue);
            self.camera.dirty = false;
        }

        for mesh_buffer in self.meshes.values_mut() {
            if mesh_buffer.instances.is_empty() {
                continue;
            }

            let instance_data = utils::as_u8_slice(&mesh_buffer.instances);
            let required_size = instance_data.len() as u64;

            if required_size > mesh_buffer.instance_buffer.size() {
                mesh_buffer.instance_buffer =
                    self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("instance buffer"),
                        size: required_size,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
            }

            self.gpu
                .queue
                .write_buffer(&mesh_buffer.instance_buffer, 0, instance_data);
        }

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_view,
                    resolve_target: Some(&output),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Discard,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                label: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

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
                render_pass.set_bind_group(0, &self.camera.view_projection_bind_group, &[]);

                for mesh_buffer in buffers {
                    let texture_bind_group = match &mesh_buffer.material.texture {
                        Some(tex) => &tex.bind_group,
                        None => &self.gpu.default_white_texture.bind_group,
                    };

                    render_pass.set_bind_group(1, texture_bind_group, &[]);

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

        self.gpu.queue.submit([encoder.finish()]);
        frame.present();

        for mesh_buffer in self.meshes.values_mut() {
            mesh_buffer.instances.clear();
        }
    }
}
