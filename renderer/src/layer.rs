use assets::AssetManager;
use math::Vector2;
use std::sync::Arc;
use utils::label;
use wgpu::naga::FastHashMap;

use crate::{
    Mesh, Text,
    camera::Camera,
    mesh::{GeometryBuffer, GpuMesh, InstanceBuffer},
    text::TextRenderer,
};

pub struct RenderLayer {
    assets: Arc<AssetManager>,
    pub camera: Camera,
    text_renderer: TextRenderer,

    /// All the texts that are / were written with `draw_debug_text`
    debug_texts: FastHashMap<String, Text>,

    /// The texts that are currently being used
    debug_texts_used: Vec<String>,

    /// Per-layer geometry buffers (vertices/indices)
    geometry_buffers: FastHashMap<u32, GeometryBuffer>,

    /// Per-layer instance buffers for each mesh geometry
    instance_buffers: FastHashMap<u32, InstanceBuffer>,
}

impl RenderLayer {
    #[inline]
    pub fn new(assets: Arc<AssetManager>, camera: Camera, text_renderer: TextRenderer) -> Self {
        Self {
            assets,
            camera,
            text_renderer,
            debug_texts: FastHashMap::default(),
            debug_texts_used: Vec::new(),
            geometry_buffers: FastHashMap::default(),
            instance_buffers: FastHashMap::default(),
        }
    }

    #[inline]
    pub(crate) fn frame_start(&mut self) {
        // Reset instance counts for all mesh buffers in this layer
        for instance_buffer in self.instance_buffers.values_mut() {
            instance_buffer.reset();
        }
    }

    #[inline]
    fn register_geometry(&mut self, mesh: &Mesh) {
        let geometry_id = mesh.geometry().id();

        if self.geometry_buffers.contains_key(&geometry_id) {
            return;
        }

        let index_count = mesh.geometry().indices().len() as u32;

        let vertex_buffer = gpu::core::Buffer::new(
            &format!("Mesh id '{:?}' vertex buffer", geometry_id),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            &mesh.geometry().vertices(),
        );

        let index_buffer = gpu::core::Buffer::new(
            &format!("Mesh id '{:?}' index buffer", geometry_id),
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            &mesh.geometry().indices(),
        );

        let geometry_buffer = GeometryBuffer {
            vertex_buffer,
            index_buffer,
            index_count,
            topology: mesh.geometry().topology(),
        };

        self.geometry_buffers.insert(geometry_id, geometry_buffer);
    }

    #[inline]
    pub fn draw_mesh(&mut self, mesh: &Mesh) {
        self.register_geometry(mesh);

        let geometry_id = mesh.geometry().id();

        if !self.instance_buffers.contains_key(&geometry_id) {
            self.instance_buffers.insert(
                geometry_id,
                InstanceBuffer::new(Mesh::INITIAL_INSTANCE_CAPACITY),
            );
        }

        let instance_buffer = self
            .instance_buffers
            .get_mut(&geometry_id)
            .expect("Cannot fail");

        let needs_new_slot = match mesh.instance_index() {
            Some(idx) if idx < instance_buffer.instance_count => {
                if mesh.is_dirty() {
                    instance_buffer.instances[idx] = mesh.for_gpu(&self.assets);
                    instance_buffer.dirty_indices.push(idx);
                    mesh.clean();
                }
                false
            }
            _ => true,
        };

        if needs_new_slot {
            let instance_idx = instance_buffer.instance_count;
            mesh.set_instance_index(instance_idx);

            if instance_idx >= instance_buffer.instances.capacity() {
                let new_capacity = instance_buffer.instances.capacity() * 2;

                instance_buffer.instance_buffer.resize(new_capacity);
                instance_buffer
                    .instances
                    .reserve(new_capacity - instance_buffer.instances.len());

                // Mark all existing instances as dirty to ensure they're written to the new buffer
                instance_buffer.dirty_indices.clear();

                for i in 0..instance_buffer.instances.len() {
                    instance_buffer.dirty_indices.push(i);
                }
            }

            if instance_idx >= instance_buffer.instances.len() {
                instance_buffer.instances.push(mesh.for_gpu(&self.assets));
            } else {
                instance_buffer.instances[instance_idx] = mesh.for_gpu(&self.assets);
            }

            instance_buffer.dirty_indices.push(instance_idx);
            instance_buffer.instance_count += 1;

            mesh.clean();
        }
    }

    #[inline]
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    #[inline]
    pub(crate) fn update(&mut self, width: u32, height: u32, dt: f32) {
        if self.camera.dirty() {
            self.camera.resize(width, height);
        }

        self.camera.update_shake(dt);
    }

    #[inline]
    pub fn draw_text(&mut self, text: &mut Text) {
        text.rebuild(&self.assets);

        for glyph in text.glyph_instances() {
            self.text_renderer.add_glyph(*glyph);
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

        // Trigger dirty
        if text.content() != content {
            text.set_content(content);
        }

        if text.position() != &pos {
            text.set_position(pos);
        }

        text.rebuild(&self.assets);

        for glyph in text.glyph_instances() {
            self.text_renderer.add_glyph(*glyph);
        }

        self.debug_texts_used.push(key);
    }

    #[inline]
    pub(crate) fn present<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        triangle_pipeline: &'a wgpu::RenderPipeline,
    ) {
        render_pass.set_pipeline(triangle_pipeline);
        render_pass.set_bind_group(0, self.camera.view_projection_bind_group(), &[]);
        render_pass.set_bind_group(1, self.assets.bind_group(), &[]);

        for (geometry_id, instance_buffer) in &self.instance_buffers {
            if instance_buffer.instance_count == 0 {
                continue;
            }

            let geometry_buffer = match self.geometry_buffers.get(geometry_id) {
                Some(buf) => buf,
                None => continue, // Geometry not registered yet
            };

            // Write dirty instances to GPU
            if !instance_buffer.dirty_indices.is_empty() {
                for &dirty_idx in &instance_buffer.dirty_indices {
                    let offset = (dirty_idx * std::mem::size_of::<GpuMesh>()) as u64;
                    instance_buffer
                        .instance_buffer
                        .write_at(offset, &[instance_buffer.instances[dirty_idx]]);
                }
            }

            // Set up buffers and draw
            render_pass.set_vertex_buffer(0, geometry_buffer.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.instance_buffer.slice(..));
            render_pass.set_index_buffer(
                geometry_buffer.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.draw_indexed(
                0..geometry_buffer.index_count,
                0,
                0..instance_buffer.instance_count as u32,
            );
        }

        self.text_renderer
            .render(render_pass, &self.camera, &self.assets);
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.text_renderer.clear();
        self.camera.clean();

        self.debug_texts
            .retain(|key, _| self.debug_texts_used.contains(key));
        self.debug_texts_used.clear();

        // Clear dirty indices for all instance buffers
        for instance_buffer in self.instance_buffers.values_mut() {
            instance_buffer.clear_dirty();
        }
    }
}
