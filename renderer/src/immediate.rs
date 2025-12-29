use crate::{Vertex, profiling};
use assets::AssetManager;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::Vector4;
use std::sync::Arc;
use utils::{Label, LabelMap, label};
use wgpu::naga::FastHashMap;

pub struct ImmediateRenderer {
    assets: Arc<AssetManager>,

    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: GpuBuffer<Vertex>,
    index_buffer: GpuBuffer<u32>,
    vertex_capacity: usize,
    index_capacity: usize,

    text_layout: Layout,

    /// (FontLabel -> (char -> (Vertices, Indices)))
    char_cache: LabelMap<FastHashMap<char, (Vec<Vertex>, Vec<u32>)>>,
    zstep: f32,
}

impl ImmediateRenderer {
    const BASE_VERTEX_CAPACITY: usize = 1024;
    const BASE_INDEX_CAPACITY: usize = 1024;

    pub(crate) fn new(assets: Arc<AssetManager>) -> Self {
        let vertex_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer vertex buffer")
            .capacity(Self::BASE_VERTEX_CAPACITY)
            .vertex()
            .copy_dst()
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("Immediate Renderer index buffer")
            .capacity(Self::BASE_INDEX_CAPACITY)
            .index()
            .copy_dst()
            .build();

        Self {
            assets,
            vertices: Vec::with_capacity(Self::BASE_VERTEX_CAPACITY),
            indices: Vec::with_capacity(Self::BASE_INDEX_CAPACITY),
            vertex_buffer,
            index_buffer,
            vertex_capacity: Self::BASE_VERTEX_CAPACITY,
            index_capacity: Self::BASE_INDEX_CAPACITY,
            text_layout: Layout::new(CoordinateSystem::PositiveYDown),
            char_cache: LabelMap::default(),
            zstep: 0.0,
        }
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.text_layout.clear();
    }

    #[inline]
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Vector4) {
        let color: [f32; 4] = color.into();

        // Get white pixel UV coords for solid color rendering
        let (uv_x, uv_y, uv_w, uv_h) = self.assets.get_white_uv_coords();
        let uv_center = [uv_x + uv_w * 0.5, uv_y + uv_h * 0.5];

        let top_left = Vertex {
            position: [x, y, self.zstep], // top-left
            color,
            uv_coords: uv_center,
        };

        let top_right = Vertex {
            position: [x + w, y, self.zstep], // top-right
            color,
            uv_coords: uv_center,
        };

        let bottom_left = Vertex {
            position: [x, y + h, self.zstep], // bottom-left
            color,
            uv_coords: uv_center,
        };

        let bottom_right = Vertex {
            position: [x + w, y + h, self.zstep], // bottom-right
            color,
            uv_coords: uv_center,
        };

        let base = self.vertices.len() as u32;

        self.vertices.push(top_left);
        self.vertices.push(top_right);
        self.vertices.push(bottom_left);
        self.vertices.push(bottom_right);

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]);
    }

    #[inline]
    pub fn draw_text(&mut self, font_label: Label, text: String, x: f32, y: f32, color: Vector4) {
        let color: [f32; 4] = color.into();
        let font = self.assets.get_font(&font_label);

        self.text_layout.clear();
        self.text_layout.append(
            &[font.inner()],
            &TextStyle::new(&text, font.size() as f32, 0),
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

                for vertex in cached_verts.iter() {
                    self.vertices.push(Vertex {
                        position: [
                            vertex.position[0] + x + glyph.x,
                            vertex.position[1] + y + glyph.y,
                            self.zstep,
                        ],
                        color,
                        uv_coords: vertex.uv_coords,
                    });
                }

                for index in cached_indices {
                    self.indices.push(*index + base_vertex);
                }
            } else {
                let texture_label = Label::new(&format!("{}_{}", font_label.raw(), ch));
                let (uv_x, uv_y, uv_w, uv_h) = self.assets.get_texture_coords(texture_label);

                let screen_x = x + glyph.x;
                let screen_y = y + glyph.y;
                let w = glyph.width as f32;
                let h = glyph.height as f32;

                let base = self.vertices.len() as u32;

                self.vertices.extend_from_slice(&[
                    Vertex {
                        position: [screen_x, screen_y, self.zstep],
                        color,
                        uv_coords: [uv_x, uv_y],
                    },
                    Vertex {
                        position: [screen_x + w, screen_y, self.zstep],
                        color,
                        uv_coords: [uv_x + uv_w, uv_y],
                    },
                    Vertex {
                        position: [screen_x, screen_y + h, self.zstep],
                        color,
                        uv_coords: [uv_x, uv_y + uv_h],
                    },
                    Vertex {
                        position: [screen_x + w, screen_y + h, self.zstep],
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
                let cached_vertices = vec![
                    Vertex {
                        position: [0.0, 0.0, 0.0],
                        color,
                        uv_coords: [uv_x, uv_y],
                    },
                    Vertex {
                        position: [w, 0.0, 0.0],
                        color,
                        uv_coords: [uv_x + uv_w, uv_y],
                    },
                    Vertex {
                        position: [0.0, h, 0.0],
                        color,
                        uv_coords: [uv_x, uv_y + uv_h],
                    },
                    Vertex {
                        position: [w, h, 0.0],
                        color,
                        uv_coords: [uv_x + uv_w, uv_y + uv_h],
                    },
                ];

                cache.insert(ch, (cached_vertices, vec![0, 2, 1, 1, 2, 3]));
            }
        }
    }

    #[inline]
    pub fn debug_text(&mut self, text: String, x: f32, y: f32, color: Vector4) {
        let font_label = label!("debug");
        self.draw_text(font_label, text, x, y, color);
    }

    #[inline]
    pub(crate) fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.vertices.is_empty() || self.indices.is_empty() {
            return;
        }

        if self.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = (self.vertices.len() * 2).max(Self::BASE_VERTEX_CAPACITY);
            self.vertex_buffer.resize(self.vertex_capacity);
        }

        if self.indices.len() > self.index_capacity {
            self.index_capacity = (self.indices.len() * 2).max(Self::BASE_INDEX_CAPACITY);
            self.index_buffer.resize(self.index_capacity);
        }

        self.vertex_buffer.write(0, &self.vertices);
        self.index_buffer.write(0, &self.indices);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        profiling::record_draw_call(self.vertices.len() as u32, self.indices.len() as u32);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);

        self.clear();
    }
}
