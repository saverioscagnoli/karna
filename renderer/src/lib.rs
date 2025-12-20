mod camera;
mod color;
mod mesh;
mod shader;
mod sprite;
mod text;

use crate::{
    camera::{Camera, Projection},
    mesh::{Descriptor, GpuMesh, MeshBuffer},
    text::TextRenderer,
};
use assets::AssetManager;
use macros::{Get, Set};
use math::{Size, Vector2};
use std::collections::HashMap;
use std::sync::Arc;
use traccia::info;
use utils::label;
use wgpu::naga::FastHashMap;
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

    pub camera: Camera,

    /// UI camera for fixed-position elements (always at 0,0)
    pub ui_camera: Camera,

    // Cache window size for camera updates
    window_size: Size<u32>,

    mesh_buffers: FastHashMap<u32, MeshBuffer>,
    triangle_pipeline: wgpu::RenderPipeline,

    text_renderer: TextRenderer,
    ui_text_renderer: TextRenderer,

    /// Very simple Debug text implementation
    /// Just for showing things on the screen
    /// Cached by a unique key (content + position hash)
    debug_texts: HashMap<String, Text>,

    /// Tracks which debug texts were used this frame
    debug_texts_used: Vec<String>,
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

        // UI camera stays fixed at origin for screen-space rendering
        let ui_camera = Camera::new(Projection::Orthographic {
            left: 0.0,
            right: size.width as f32,
            bottom: size.height as f32,
            top: 0.0,
            z_near: -1.0,
            z_far: 1.0,
        });

        let ui_text_renderer = TextRenderer::new(surface_format, &ui_camera, &assets);

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
            ui_camera,
            window_size: Size::new(size.width, size.height),
            assets,
            mesh_buffers: FastHashMap::default(),
            triangle_pipeline,
            text_renderer,
            ui_text_renderer: TextRenderer::new(surface_format, &ui_camera, &assets),
            debug_texts: HashMap::new(),
            debug_texts_used: Vec::new(),
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
        self.ui_camera.update(width, height);
        self.window_size.width = width;
        self.window_size.height = height;
    }

    #[inline]
    pub fn begin_frame(&mut self) {
        // Reset instance counts for all mesh buffers
        // This allows dynamic content (like text) to reuse instance slots each frame
        for mesh_buffer in self.mesh_buffers.values_mut() {
            mesh_buffer.instance_count = 0;
        }
    }

    #[inline]
    fn register_mesh(&mut self, mesh: &Mesh) {
        let index_count = mesh.geometry().indices().len() as u32;

        let vertex_buffer = gpu::core::Buffer::new(
            &format!("Mesh id '{:?}' vertex buffer", mesh.geometry().id()),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            &mesh.geometry().vertices(),
        );

        let index_buffer = gpu::core::Buffer::new(
            &format!("Mesh id '{:?}' index buffer", mesh.geometry().id()),
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            &mesh.geometry().indices(),
        );

        let instance_buffer = gpu::core::Buffer::new_with_capacity(
            "mesh instance buffer",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            Mesh::INITIAL_INSTANCE_CAPACITY,
        );

        let mesh_buffer = MeshBuffer {
            vertex_buffer,
            index_buffer,
            index_count,
            instance_buffer,
            instances: Vec::with_capacity(Mesh::INITIAL_INSTANCE_CAPACITY),
            topology: mesh.geometry().topology(),
            dirty_indices: Vec::new(),
            instance_count: 0,
        };

        self.mesh_buffers.insert(mesh.geometry().id(), mesh_buffer);
    }

    #[inline]
    pub fn draw_mesh(&mut self, mesh: &Mesh) {
        if !self.mesh_buffers.contains_key(&mesh.geometry().id()) {
            self.register_mesh(mesh);
        }

        let mesh_buffer = self
            .mesh_buffers
            .get_mut(&mesh.geometry().id())
            .expect("Cannot fail");

        // Check if this mesh already has an instance slot AND it's still valid
        // (instance_count gets reset each frame, so old indices become invalid)
        let needs_new_slot = match mesh.instance_index() {
            Some(idx) if idx < mesh_buffer.instance_count => {
                // Valid slot, update it if dirty
                if mesh.is_dirty() {
                    mesh_buffer.instances[idx] = mesh.for_gpu(&self.assets);
                    mesh_buffer.dirty_indices.push(idx);
                    mesh.clean();
                }
                false
            }
            _ => true, // Either no slot or slot is beyond current instance_count
        };

        if needs_new_slot {
            // Allocate a new slot (or reallocate if instance_count was reset)
            let instance_idx = mesh_buffer.instance_count;
            mesh.set_instance_index(instance_idx);

            // Check if we need to resize the instance buffer
            if instance_idx >= mesh_buffer.instances.capacity() {
                let new_capacity = mesh_buffer.instances.capacity() * 2;

                mesh_buffer.instance_buffer.resize(new_capacity);
                mesh_buffer
                    .instances
                    .reserve(new_capacity - mesh_buffer.instances.len());

                // Mark all existing instances as dirty to ensure they're written to the new buffer
                mesh_buffer.dirty_indices.clear();

                for i in 0..mesh_buffer.instances.len() {
                    mesh_buffer.dirty_indices.push(i);
                }
            }

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
    pub fn draw_text(&mut self, text: &mut Text) {
        text.rebuild(&self.assets);

        // Add all glyphs to the text renderer
        for glyph_instance in text.glyph_instances() {
            self.text_renderer.add_glyph(*glyph_instance);
        }
    }

    #[inline]
    pub fn draw_debug_text<T: Into<String>, P: Into<Vector2>>(&mut self, text: T, pos: P) {
        let content = text.into();
        let pos = pos.into();

        let key = format!("{}:{}", pos.x.round() as i32, pos.y.round() as i32);

        let text = self
            .debug_texts
            .entry(key.clone())
            .or_insert_with(|| Text::new(label!("debug")));

        // Trigger dirty if content or
        if text.content() != content {
            text.set_content(content);
        }

        if text.position() != &pos {
            text.set_position(pos);
        }

        text.rebuild(&self.assets);

        for glyph_instance in text.glyph_instances() {
            self.text_renderer.add_glyph(*glyph_instance);
        }

        // Mark this debug text as used this frame
        self.debug_texts_used.push(key);
    }

    #[inline]
    /// Draw text using the UI camera (fixed position on screen, unaffected by main camera)
    pub fn draw_ui_text(&mut self, text: &mut Text) {
        text.rebuild(&self.assets);

        // Add all glyphs to the UI text renderer
        for glyph_instance in text.glyph_instances() {
            self.ui_text_renderer.add_glyph(*glyph_instance);
        }
    }

    #[inline]
    pub fn present(&mut self) -> Result<(), wgpu::SurfaceError> {
        let gpu = gpu::get();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update cameras if needed
        if self.camera.dirty() {
            self.camera
                .update(self.window_size.width, self.window_size.height);
        }

        if self.ui_camera.dirty() {
            self.ui_camera
                .update(self.window_size.width, self.window_size.height);
        }

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

            // Render meshes
            render_pass.set_pipeline(&self.triangle_pipeline);
            render_pass.set_bind_group(0, self.camera.view_projection_bind_group(), &[]);
            render_pass.set_bind_group(1, self.assets.bind_group(), &[]);

            for mesh_buffer in self.mesh_buffers.values() {
                // Only write buffer for dirty instances using partial writes
                if !mesh_buffer.dirty_indices.is_empty() {
                    for &dirty_idx in &mesh_buffer.dirty_indices {
                        let offset = (dirty_idx * std::mem::size_of::<GpuMesh>()) as u64;

                        mesh_buffer
                            .instance_buffer
                            .write_at(offset, &[mesh_buffer.instances[dirty_idx]]);
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

            // Render all text in a single batched draw call
            self.text_renderer
                .render(&mut render_pass, &self.camera, &self.assets);

            // Render UI text with the UI camera (fixed on screen)
            self.ui_text_renderer
                .render(&mut render_pass, &self.ui_camera, &self.assets);
        }

        // Clear text instances for next frame
        self.text_renderer.clear();
        self.ui_text_renderer.clear();
</text>


        // Remove debug texts that weren't used this frame to avoid memory leaks
        // This keeps the cache lean while still benefiting from frame-to-frame reuse
        self.debug_texts
            .retain(|key, _| self.debug_texts_used.contains(key));
        self.debug_texts_used.clear();

        for mesh_buffer in self.mesh_buffers.values_mut() {
            mesh_buffer.dirty_indices.clear();
        }

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
