use crate::{Color, Vertex, immediate::ImmediateRenderer};
use assets::AssetManager;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use globals::profiling;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::Vector4;
use utils::{Label, LabelMap};
use wgpu::naga::FastHashMap;

pub struct TriangleBatcher {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: GpuBuffer<Vertex>,
    index_buffer: GpuBuffer<u32>,

    text_layout: Layout,
    /// (FontLabel -> (char -> (Vertices, Indices)))
    char_cache: LabelMap<FastHashMap<char, (Vec<Vertex>, Vec<u32>)>>,

    pipeline: wgpu::RenderPipeline,
}

impl TriangleBatcher {
    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        let vertex_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer triangle vertex buffer")
            .capacity(ImmediateRenderer::BASE_VERTEX_CAPACITY)
            .vertex()
            .copy_dst()
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer triangle index buffer")
            .capacity(ImmediateRenderer::BASE_INDEX_CAPACITY)
            .index()
            .copy_dst()
            .build();

        Self {
            vertices: Vec::with_capacity(ImmediateRenderer::BASE_VERTEX_CAPACITY),
            indices: Vec::with_capacity(ImmediateRenderer::BASE_INDEX_CAPACITY),
            vertex_buffer,
            index_buffer,
            text_layout: Layout::new(CoordinateSystem::PositiveYDown),
            char_cache: LabelMap::default(),
            pipeline,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.text_layout.clear();
    }

    #[inline]
    pub fn fill_rect(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
        h: f32,
        color: Vector4,
        assets: &AssetManager,
    ) {
        let color: [f32; 4] = color.into();

        // Get white pixel UV coords for solid color rendering
        let (uv_x, uv_y, uv_w, uv_h) = assets.get_white_uv_coords();
        let uv_center = [uv_x + uv_w * 0.5, uv_y + uv_h * 0.5];
        let base = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&[
            Vertex {
                position: [x, y, z],
                color,
                uv_coords: uv_center,
            },
            Vertex {
                position: [x + w, y, z],
                color,
                uv_coords: uv_center,
            },
            Vertex {
                position: [x + w, y + h, z],
                color,
                uv_coords: uv_center,
            },
            Vertex {
                position: [x, y + h, z],
                color,
                uv_coords: uv_center,
            },
        ]);

        self.indices.extend_from_slice(&[
            base + 0,
            base + 1,
            base + 2,
            base + 0,
            base + 2,
            base + 3,
        ]);
    }

    #[inline]
    pub fn draw_text(
        &mut self,
        font_label: Label,
        text: &str,
        x: f32,
        y: f32,
        z: f32,
        color: Vector4,
        assets: &AssetManager,
    ) {
        let color: [f32; 4] = color.into();
        let font = assets.get_font(&font_label);

        self.text_layout.clear();
        self.text_layout.append(
            &[font.inner()],
            &TextStyle::new(text, font.size() as f32, 0),
        );

        let glyphs: Vec<_> = self.text_layout.glyphs().iter().copied().collect();

        let cache = self
            .char_cache
            .entry(font_label)
            .or_insert_with(FastHashMap::default);

        for glyph in glyphs {
            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            let ch = glyph.parent;

            if let Some((cached_verts, cached_indices)) = cache.get(&ch) {
                let base_vertex = self.vertices.len() as u32;

                for mut vertex in cached_verts.iter().copied() {
                    vertex.position[0] += x + glyph.x;
                    vertex.position[1] += y + glyph.y;
                    vertex.position[2] = z;
                    vertex.color = color;

                    self.vertices.push(vertex);
                }

                for index in cached_indices {
                    self.indices.push(*index + base_vertex);
                }
            } else {
                let texture_label = Label::new(&format!("{}_{}", font_label.raw(), ch));
                let (uv_x, uv_y, uv_w, uv_h) = assets.get_texture_coords(texture_label);

                let screen_x = x + glyph.x;
                let screen_y = y + glyph.y;
                let w = glyph.width as f32;
                let h = glyph.height as f32;

                let base = self.vertices.len() as u32;

                self.vertices.extend_from_slice(&[
                    Vertex {
                        position: [screen_x, screen_y, z],
                        color,
                        uv_coords: [uv_x, uv_y],
                    },
                    Vertex {
                        position: [screen_x + w, screen_y, z],
                        color,
                        uv_coords: [uv_x + uv_w, uv_y],
                    },
                    Vertex {
                        position: [screen_x, screen_y + h, z],
                        color,
                        uv_coords: [uv_x, uv_y + uv_h],
                    },
                    Vertex {
                        position: [screen_x + w, screen_y + h, z],
                        color,
                        uv_coords: [uv_x + uv_w, uv_y + uv_h],
                    },
                ]);

                self.indices.extend_from_slice(&[
                    base,
                    base + 2,
                    base + 1,
                    base + 1,
                    base + 2,
                    base + 3,
                ]);

                // Cache relative to (0, 0)
                let cached_color = Color::White.into();
                let cached_vertices = vec![
                    Vertex {
                        position: [0.0, 0.0, 0.0],
                        color: cached_color,
                        uv_coords: [uv_x, uv_y],
                    },
                    Vertex {
                        position: [w, 0.0, 0.0],
                        color: cached_color,
                        uv_coords: [uv_x + uv_w, uv_y],
                    },
                    Vertex {
                        position: [0.0, h, 0.0],
                        color: cached_color,
                        uv_coords: [uv_x, uv_y + uv_h],
                    },
                    Vertex {
                        position: [w, h, 0.0],
                        color: cached_color,
                        uv_coords: [uv_x + uv_w, uv_y + uv_h],
                    },
                ];

                cache.insert(ch, (cached_vertices, vec![0, 2, 1, 1, 2, 3]));
            }
        }
    }

    #[inline]
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertices.is_empty() {
            return;
        }

        let vertices_n = self.vertices.len() as u32;
        let indices_n = self.indices.len() as u32;

        if vertices_n > self.vertex_buffer.capacity() as u32 {
            let new_capacity = self.vertex_buffer.capacity() * 2;
            self.vertex_buffer.resize(new_capacity);
        }

        if indices_n > self.index_buffer.capacity() as u32 {
            let new_capacity = self.index_buffer.capacity() * 2;
            self.index_buffer.resize(new_capacity);
        }

        self.vertex_buffer.write_all(&self.vertices);
        self.index_buffer.write_all(&self.indices);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..indices_n, 0, 0..1);
        profiling::record_draw_call(vertices_n, indices_n);
        profiling::record_triangles(indices_n);

        self.clear();
    }
}
