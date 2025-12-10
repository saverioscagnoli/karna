mod camera;
mod color;
mod gpu;
mod mesh;
mod shader;
mod sprite;
mod text;
mod texture;

use common::{
    label,
    utils::{self, Label},
};
use macros::{Get, Set};
use math::{Size, Vector2};
use mesh::RawMesh;
use std::sync::Arc;
use traccia::{info, warn};
use wgpu::{
    Surface, SurfaceConfiguration,
    naga::{FastHashMap, back::RayIntersectionType},
    util::DeviceExt,
};
use winit::window::{Window, WindowId};

// Re-exports
pub use crate::camera::{Camera, Projection};
use crate::texture::atlas;
pub use crate::{text::Font, texture::Texture};
pub use color::Color;
pub use gpu::gpu;
pub use gpu::*;
pub use mesh::{
    Descriptor, Mesh, MeshBuffer, Vertex,
    geometry::MeshGeometry,
    material::{Material, TextureKind, TextureRegion},
    transform::Transform,
};
pub use shader::*;
pub use sprite::{Frame, Sprite};
pub use text::Text;
pub use texture::atlas::TextureAtlas;

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Renderer {
    surface: Surface<'static>,
    config: SurfaceConfiguration,

    #[get]
    #[set(into)]
    clear_color: Color,

    camera: Camera,
    triangle_pipeline: wgpu::RenderPipeline,

    mesh_cache: FastHashMap<u32, MeshBuffer>,
    needs_camera_update: bool,
    white_texture: Arc<Texture>,
}

impl Renderer {
    #[doc(hidden)]
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let gpu = gpu();

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

        // Create a separate 1x1 white texture for untextured meshes
        let white_bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("white texture bind group layout"),
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

        let white_texture = Arc::new(texture::Texture::new_empty(
            "White Pixel",
            &gpu.device,
            Size::new(1, 1),
            &white_bind_group_layout,
        ));

        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &white_texture.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &[255u8, 255u8, 255u8, 255u8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let triangle_pipeline = {
            let texture_atlas = gpu.texture_atlas.load();

            Self::create_render_pipeline(
                "triangle pipeline",
                &gpu.device,
                &shader,
                &[
                    &camera.view_projection_bind_group_layout,
                    &*texture_atlas.bind_group_layout,
                ],
                format,
                wgpu::PrimitiveTopology::TriangleList,
                wgpu::PolygonMode::Fill,
            )
        };

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
            triangle_pipeline,
            mesh_cache: FastHashMap::default(),
            needs_camera_update: false,
            white_texture,
        }
    }

    #[inline]
    #[doc(hidden)]
    pub fn init(&self) {
        self.load_font(
            label!("debug"),
            include_bytes!("../../assets/DOS-V.ttf"),
            16,
        );

        info!("Loaded debug font with label 'debug'");
        info!("Renderer initalized");
    }

    #[inline]
    pub fn get_texture_size(&self, label: &Label) -> Option<Size<u32>> {
        let guard = gpu().texture_atlas.load();

        guard.get_texture_size(label)
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
        let gpu = gpu();
        let index_count = mesh.geometry.indices.len() as u32;
        let vertex_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Mesh id '{:?}' vertex buffer", mesh.geometry.id)),
                contents: utils::as_u8_slice(&mesh.geometry.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Mesh id '{:?}' index buffer", mesh.geometry.id)),
                contents: utils::as_u8_slice(&mesh.geometry.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        let instance_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
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
            textured_instances: Vec::new(),
            untextured_instances: Vec::new(),
            topology: mesh.geometry.topology,
        };

        self.mesh_cache.insert(mesh.geometry.id, mesh_buffer);
    }

    #[inline]
    pub fn draw_mesh(&mut self, mesh: &Mesh) {
        if !self.mesh_cache.contains_key(&mesh.geometry.id) {
            self.register_mesh(mesh);
        }

        let mut raw_mesh = mesh.to_raw();
        let mesh_buffer = self.mesh_cache.get_mut(&mesh.geometry.id).unwrap();

        if let Some(texture_kind) = &mesh.material.texture {
            let guard = gpu().texture_atlas.load();

            match texture_kind {
                TextureKind::Full(label) => {
                    // Use the entire texture from the atlas
                    if let Some(uv_coords) = guard.get_uv_coords(label) {
                        raw_mesh.uv_offset = [uv_coords.min_x, uv_coords.min_y];
                        raw_mesh.uv_scale = [
                            uv_coords.max_x - uv_coords.min_x,
                            uv_coords.max_y - uv_coords.min_y,
                        ];
                    }
                }
                TextureKind::Partial(label, region) => {
                    // Use a specific region of the texture
                    if let Some(base_uv) = guard.get_uv_coords(label) {
                        // Calculate the atlas size
                        let atlas_size = guard.size;

                        // Convert pixel coordinates to normalized texture coordinates within the atlas
                        let region_start_x = region.x as f32 / atlas_size.width as f32;
                        let region_start_y = region.y as f32 / atlas_size.height as f32;
                        let region_width = region.width as f32 / atlas_size.width as f32;
                        let region_height = region.height as f32 / atlas_size.height as f32;

                        // Offset from the base texture position in the atlas
                        raw_mesh.uv_offset = [
                            base_uv.min_x + region_start_x,
                            base_uv.min_y + region_start_y,
                        ];
                        raw_mesh.uv_scale = [region_width, region_height];
                    }
                }
            }

            mesh_buffer.textured_instances.push(raw_mesh);
        } else {
            mesh_buffer.untextured_instances.push(raw_mesh);
        }
    }

    /// Draws the entire texture atlas at the specified position.
    /// Useful for debugging and visualizing atlas contents.
    #[inline]
    pub fn draw_texture_atlas<P: Into<Vector2>>(&mut self, pos: P) {
        let pos = pos.into();
        let atlas_size = {
            let guard = gpu().texture_atlas.load();
            guard.size
        };

        let mesh = Mesh {
            geometry: MeshGeometry::rect(),
            material: Material {
                color: Some(Color::White),
                texture: None,
            },
            transform: Transform::default()
                .with_position(pos)
                .with_scale(Vector2::from(atlas_size)),
        };

        if !self.mesh_cache.contains_key(&mesh.geometry.id) {
            self.register_mesh(&mesh);
        }

        let raw_mesh = RawMesh {
            position: pos.extend(0.0).into(),
            scale: math::Vector2::new(atlas_size.width as f32, atlas_size.height as f32)
                .extend(1.0)
                .into(),
            rotation: [0.0, 0.0, 0.0],
            color: Color::White.into(),
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
        };

        self.mesh_cache
            .get_mut(&mesh.geometry.id)
            .unwrap()
            .textured_instances
            .push(raw_mesh);
    }

    #[inline]
    pub fn draw_debug_text<T: Into<String>, P: Into<Vector2>>(&mut self, text: T, position: P) {
        let rect_geometry = MeshGeometry::rect();
        let rect_id = rect_geometry.id;

        if !self.mesh_cache.contains_key(&rect_id) {
            let temp_mesh = Mesh {
                geometry: rect_geometry,
                material: Material {
                    color: Some(Color::White),
                    texture: None,
                },
                transform: Transform::default(),
            };

            self.register_mesh(&temp_mesh);
        }

        let atlas_guard = gpu().texture_atlas.load();
        let mesh_buffer = self.mesh_cache.get_mut(&rect_id).unwrap();
        let text = Text::new(label!("debug"), text)
            .with_transform(Transform::default().with_position(position));

        let glyphs = text.compute_glyphs();

        // Render each glyph from the layout
        for glyph in &*glyphs {
            let glyph_label =
                Label::new(&format!("{}_char_{}", text.font.raw(), glyph.parent as u32));

            if let Some(uv_coords) = atlas_guard.get_uv_coords(&glyph_label) {
                // Position includes the text transform
                let char_position = Vector2::new(
                    text.position.x + glyph.x * text.scale.x,
                    text.position.y + glyph.y * text.scale.y,
                );

                let char_scale = Vector2::new(
                    glyph.width as f32 * text.scale.x,
                    glyph.height as f32 * text.scale.y,
                );

                let raw_mesh = RawMesh {
                    position: char_position.extend(0.0).into(),
                    scale: char_scale.extend(1.0).into(),
                    rotation: [0.0, 0.0, text.rotation],
                    color: text.color.into(),
                    uv_offset: [uv_coords.min_x, uv_coords.min_y],
                    uv_scale: [
                        uv_coords.max_x - uv_coords.min_x,
                        uv_coords.max_y - uv_coords.min_y,
                    ],
                };

                mesh_buffer.textured_instances.push(raw_mesh);
            }
        }
    }

    #[inline]
    pub fn draw_text(&mut self, text: &Text) {
        // Get the rect geometry for rendering each character as a quad
        let rect_geometry = MeshGeometry::rect();
        let rect_id = rect_geometry.id;

        if !self.mesh_cache.contains_key(&rect_id) {
            let temp_mesh = Mesh {
                geometry: rect_geometry,
                material: Material {
                    color: Some(Color::White),
                    texture: None,
                },
                transform: Transform::default(),
            };

            self.register_mesh(&temp_mesh);
        }

        let atlas_guard = gpu().texture_atlas.load();
        let mesh_buffer = self.mesh_cache.get_mut(&rect_id).unwrap();
        let glyphs = text.compute_glyphs();

        // Render each glyph from the layout
        for glyph in &*glyphs {
            let glyph_label =
                Label::new(&format!("{}_char_{}", text.font.raw(), glyph.parent as u32));

            if let Some(uv_coords) = atlas_guard.get_uv_coords(&glyph_label) {
                // Position includes the text transform
                let char_position = Vector2::new(
                    text.position.x + glyph.x * text.scale.x,
                    text.position.y + glyph.y * text.scale.y,
                );

                let char_scale = Vector2::new(
                    glyph.width as f32 * text.scale.x,
                    glyph.height as f32 * text.scale.y,
                );

                let raw_mesh = RawMesh {
                    position: char_position.extend(0.0).into(),
                    scale: char_scale.extend(1.0).into(),
                    rotation: [0.0, 0.0, text.rotation],
                    color: text.color.into(),
                    uv_offset: [uv_coords.min_x, uv_coords.min_y],
                    uv_scale: [
                        uv_coords.max_x - uv_coords.min_x,
                        uv_coords.max_y - uv_coords.min_y,
                    ],
                };

                mesh_buffer.textured_instances.push(raw_mesh);
            }
        }
    }

    #[inline]
    #[doc(hidden)]
    pub fn resize(&mut self, size: Size<u32>) {
        if size.width == 0 || size.height == 0 {
            warn!("cannot set witdth or height to 0");
            return;
        }

        info!("Resizing window to {}x{}", size.width, size.height);

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&gpu().device, &self.config);

        self.needs_camera_update = true;
    }

    #[inline]
    pub fn load_texture(&self, label: Label, bytes: &[u8]) {
        let gpu = gpu();

        // Update texture atlas using RCU
        gpu.texture_atlas.rcu(|atlas| {
            let mut new_atlas = (**atlas).clone();
            new_atlas
                .load_image(&gpu.queue, label, bytes)
                .expect("Failed to load texture");
            new_atlas
        });
    }

    #[inline]
    pub fn load_font(&self, label: Label, bytes: &[u8], size: u8) {
        let gpu = gpu();
        let font = Arc::new(Font::new(label, bytes, size));

        gpu.texture_atlas.rcu(|atlas| {
            let mut new_atlas = (**atlas).clone();
            new_atlas.load_font(&font, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890!@#$%^&*()-_=+[]{}|;:'\",.<>/?", &gpu.queue).expect("Failed to load font");
            new_atlas
        });

        // Update fonts map using RCU
        gpu.fonts.rcu(|fonts| {
            let mut new_fonts = (**fonts).clone();
            new_fonts.insert(label, Arc::clone(&font));
            new_fonts
        });
    }

    #[inline]
    pub fn present(&mut self) -> Result<(), wgpu::SurfaceError> {
        let gpu = gpu();

        if self.needs_camera_update {
            let size = Size {
                width: self.config.width,
                height: self.config.height,
            };
            self.camera.update(&size, &gpu.queue);
            self.needs_camera_update = false;
        }

        // Write instance data to GPU buffers
        for mesh_buffer in self.mesh_cache.values_mut() {
            let all_instances: Vec<_> = mesh_buffer
                .textured_instances
                .iter()
                .chain(mesh_buffer.untextured_instances.iter())
                .copied()
                .collect();

            if all_instances.is_empty() {
                continue;
            }

            let required_size = (std::mem::size_of::<RawMesh>() * all_instances.len()) as u64;

            if required_size > mesh_buffer.instance_buffer.size() {
                let new_capacity = all_instances.len().next_power_of_two();
                mesh_buffer.instance_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("instance buffer"),
                    size: (std::mem::size_of::<RawMesh>() * new_capacity) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }

            gpu.queue.write_buffer(
                &mesh_buffer.instance_buffer,
                0,
                utils::as_u8_slice(&all_instances),
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

            let atlas_lock = gpu.texture_atlas.load();

            for mesh_buffer in self.mesh_cache.values() {
                let textured_count = mesh_buffer.textured_instances.len() as u32;
                let untextured_count = mesh_buffer.untextured_instances.len() as u32;

                if textured_count == 0 && untextured_count == 0 {
                    continue;
                }

                let pipeline = match mesh_buffer.topology {
                    wgpu::PrimitiveTopology::TriangleList => &self.triangle_pipeline,
                    _ => todo!("Unsupported topology"),
                };

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, &self.camera.view_projection_bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh_buffer.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, mesh_buffer.instance_buffer.slice(..));
                render_pass.set_index_buffer(
                    mesh_buffer.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                // Draw textured instances (first in buffer)
                if textured_count > 0 {
                    render_pass.set_bind_group(1, &atlas_lock.texture.bind_group, &[]);
                    render_pass.draw_indexed(0..mesh_buffer.index_count, 0, 0..textured_count);
                }

                // Draw untextured instances (after textured in buffer)
                if untextured_count > 0 {
                    render_pass.set_bind_group(1, &self.white_texture.bind_group, &[]);
                    render_pass.draw_indexed(
                        0..mesh_buffer.index_count,
                        0,
                        textured_count..(textured_count + untextured_count),
                    );
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        for mesh_buffer in self.mesh_cache.values_mut() {
            mesh_buffer.textured_instances.clear();
            mesh_buffer.untextured_instances.clear();
        }

        Ok(())
    }
}
