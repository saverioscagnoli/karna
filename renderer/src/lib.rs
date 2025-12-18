mod camera;
mod color;
mod mesh;
mod shader;
mod text;

use crate::{
    camera::{Camera, Projection},
    mesh::{Descriptor, GpuMesh, MeshBuffer},
};
use assets::AssetManager;
use macros::{Get, Set};
use math::Vector2;
use std::sync::Arc;
use traccia::info;
use utils::map::{Label, LabelMap};
use wgpu::{naga::FastHashMap, util::DeviceExt};
use winit::window::Window;

// Re-exports
pub use color::Color;
pub use gpu::GpuState;
pub use mesh::{Geometry, Material, Mesh, Transform, Vertex};
pub use shader::Shader;
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

    camera: Camera,

    mesh_buffers: FastHashMap<u32, MeshBuffer>,
    glyph_meshes: LabelMap<Mesh>,
    triangle_pipeline: wgpu::RenderPipeline,
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

        let shader =
            Shader::from_wgsl_file(include_str!("../../shaders/basic_2d.wgsl"), Some("shader"));

        let triangle_pipeline = shader
            .pipeline_builder()
            .label("triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc(), GpuMesh::desc()],
            );

        Self {
            surface,
            config,
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            camera,
            assets,
            mesh_buffers: FastHashMap::default(),
            glyph_meshes: LabelMap::default(),
            triangle_pipeline,
        }
    }

    #[inline]
    /// Gets adapter information
    pub fn info() -> wgpu::AdapterInfo {
        gpu::adapter().get_info()
    }

    #[inline]
    #[doc(hidden)]
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        info!("Resized to  {}x{}", width, height);

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&gpu::device(), &self.config);
        self.camera.update(width, height);
    }

    #[inline]
    fn register_mesh(&mut self, mesh: &Mesh) {
        let gpu = gpu::get();
        let index_count = mesh.geometry().indices.len() as u32;

        let vertex_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Mesh id '{:?}' vertex buffer", mesh.geometry().id)),
                contents: utils::as_u8_slice(&mesh.geometry().vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Mesh id '{:?}' index buffer", mesh.geometry().id)),
                contents: utils::as_u8_slice(&mesh.geometry().indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let instance_buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (std::mem::size_of::<GpuMesh>() * Mesh::INITIAL_INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mesh_buffer = MeshBuffer {
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer,
            instances: Vec::with_capacity(Mesh::INITIAL_INSTANCE_CAPACITY),
            topology: mesh.geometry().topology,
            dirty_indices: Vec::new(),
            instance_count: 0,
        };

        self.mesh_buffers.insert(mesh.geometry().id, mesh_buffer);
    }

    #[inline]
    pub fn draw_mesh(&mut self, mesh: &Mesh) {
        if !self.mesh_buffers.contains_key(&mesh.geometry().id) {
            self.register_mesh(mesh);
        }

        let mesh_buffer = self
            .mesh_buffers
            .get_mut(&mesh.geometry().id)
            .expect("Cannot fail");

        // Check if this mesh already has an instance slot
        if let Some(instance_idx) = mesh.instance_index() {
            // Mesh already has a slot, update it if dirty
            if mesh.is_dirty() {
                mesh_buffer.instances[instance_idx] = mesh.for_gpu(&self.assets);
                mesh_buffer.dirty_indices.push(instance_idx);
                mesh.clean();
            }
        } else {
            // New mesh, allocate a slot
            let instance_idx = mesh_buffer.instance_count;
            mesh.set_instance_index(instance_idx);

            if instance_idx >= mesh_buffer.instances.len() {
                mesh_buffer.instances.push(mesh.for_gpu(&self.assets));
            } else {
                mesh_buffer.instances[instance_idx] = mesh.for_gpu(&self.assets);
            }

            mesh_buffer.dirty_indices.push(instance_idx);
            mesh_buffer.instance_count += 1;
            mesh.clean();
        }
    }

    #[inline]
    pub fn draw_text(&mut self, text: &Text) {
        let font_label = text.font_label();
        let font = self.assets.get_font(font_label);
        let mut pos = Vector2::new(0.0, 0.0);

        let chars = text
            .content()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<Vec<_>>();

        // Collect labels first
        let texture_labels: Vec<Label> = text
            .content()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .map(|ch| Label::new(&format!("{}_{}", font_label.raw(), ch)))
            .collect();

        // Ensure all meshes exist
        for (ch, texture_label) in chars.into_iter().zip(&texture_labels) {
            if !self.glyph_meshes.contains_key(texture_label) {
                let glyph = font.get_glyph(&ch);
                let mesh = Mesh::new(
                    Geometry::unit_rect(),
                    Material::new_texture(texture_label.clone()),
                    Transform::default()
                        .with_scale(Vector2::new(glyph.width as f32, glyph.height as f32)),
                );
                self.glyph_meshes.insert(texture_label.clone(), mesh);
            }
        }

        // Now draw - get raw pointer to avoid borrow checker issues
        for texture_label in texture_labels {
            // SAFETY: We know the mesh exists and we're not modifying glyph_meshes in draw_mesh
            let mesh_ptr = self
                .glyph_meshes
                .get(&texture_label)
                .expect("Mesh must exist") as *const Mesh;
            unsafe {
                self.draw_mesh(&*mesh_ptr);
            }
            pos += 16.0;
        }
    }

    #[inline]
    pub fn present(&mut self) -> Result<(), wgpu::SurfaceError> {
        let gpu = gpu::get();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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

            render_pass.set_pipeline(&self.triangle_pipeline);
            render_pass.set_bind_group(0, self.camera.view_projection_bind_group(), &[]);
            render_pass.set_bind_group(1, self.assets.bind_group(), &[]);

            for mesh_buffer in self.mesh_buffers.values() {
                // Only write buffer for dirty instances using partial writes
                if !mesh_buffer.dirty_indices.is_empty() {
                    for &dirty_idx in &mesh_buffer.dirty_indices {
                        let offset = (dirty_idx * std::mem::size_of::<GpuMesh>()) as u64;
                        gpu.queue().write_buffer(
                            &mesh_buffer.instance_buffer,
                            offset,
                            utils::as_u8_slice(&[mesh_buffer.instances[dirty_idx]]),
                        );
                    }
                }

                render_pass.set_vertex_buffer(0, mesh_buffer.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, mesh_buffer.instance_buffer.slice(..));
                render_pass.set_index_buffer(
                    mesh_buffer.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(
                    0..mesh_buffer.index_count,
                    0,
                    0..mesh_buffer.instance_count as u32,
                );
            }
        }

        for mesh_buffer in self.mesh_buffers.values_mut() {
            mesh_buffer.dirty_indices.clear();
        }

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
