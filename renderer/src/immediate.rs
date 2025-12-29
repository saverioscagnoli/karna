use crate::{Vertex, profiling};
use assets::AssetManager;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::{Size, Vector2, Vector3, Vector4};
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

    /// (FontLabel -> (Text -> (Vertices, Indices)))
    text_cache: LabelMap<FastHashMap<String, (Vec<Vertex>, Vec<u32>)>>,
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
            text_cache: LabelMap::default(),
        }
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.text_layout.clear();
    }

    #[inline]
    pub fn fill_rect(&mut self, pos: Vector2, size: Size<f32>, color: Vector4) {
        let top_left = pos;
        let top_right = pos + Vector2::new(size.width, 0.0);
        let bottom_left = pos + Vector2::new(0.0, size.height);
        let bottom_right = pos + Vector2::from(size);

        // Get white pixel UV coords for solid color rendering
        let (uv_x, uv_y, uv_w, uv_h) = self.assets.get_white_uv_coords();
        let uv_center = Vector2::new(uv_x + uv_w * 0.5, uv_y + uv_h * 0.5);

        let top_left = Vertex {
            position: top_left.extend(0.0),
            color,
            uv_coords: uv_center,
        };

        let top_right = Vertex {
            position: top_right.extend(0.0),
            color,
            uv_coords: uv_center,
        };

        let bottom_left = Vertex {
            position: bottom_left.extend(0.0),
            color,
            uv_coords: uv_center,
        };

        let bottom_right = Vertex {
            position: bottom_right.extend(0.0),
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
        let cache = self
            .text_cache
            .entry(font_label.clone())
            .or_insert_with(FastHashMap::default);

        // Check if the text is already cached
        if let Some((vertices, indices)) = cache.get(&text) {
            let base_vertex = self.vertices.len() as u32;

            for vertex in vertices {
                self.vertices.push(Vertex {
                    position: vertex.position + Vector3::new(x, y, 0.0),
                    color,
                    uv_coords: vertex.uv_coords,
                });
            }

            for &index in indices {
                self.indices.push(index + base_vertex);
            }

            return;
        }

        let font = self.assets.get_font(&font_label);

        self.text_layout.clear();
        self.text_layout.append(
            &[font.inner()],
            &TextStyle::new(&text, font.size() as f32, 0),
        );

        let mut cached_vertices = Vec::new();
        let mut cached_indices = Vec::new();

        for glyph in self.text_layout.glyphs() {
            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            let texture_label = Label::new(&format!("{}_{}", font_label.raw(), glyph.parent));
            let (tex_x, tex_y, tex_w, tex_h) = self.assets.get_texture_coords(texture_label);

            let screen_x = x + glyph.x;
            let screen_y = y + glyph.y;

            let uv_x = tex_x;
            let uv_y = tex_y;
            let uv_w = tex_w;
            let uv_h = tex_h;

            let top_left = Vertex {
                position: Vector3::new(screen_x, screen_y, 0.0),
                color,
                uv_coords: Vector2::new(uv_x, uv_y),
            };

            let top_right = Vertex {
                position: Vector3::new(screen_x + glyph.width as f32, screen_y, 0.0),
                color,
                uv_coords: Vector2::new(uv_x + uv_w, uv_y),
            };

            let bottom_left = Vertex {
                position: Vector3::new(screen_x, screen_y + glyph.height as f32, 0.0),
                color,
                uv_coords: Vector2::new(uv_x, uv_y + uv_h),
            };

            let bottom_right = Vertex {
                position: Vector3::new(
                    screen_x + glyph.width as f32,
                    screen_y + glyph.height as f32,
                    0.0,
                ),
                color,
                uv_coords: Vector2::new(uv_x + uv_w, uv_y + uv_h),
            };

            let base = self.vertices.len() as u32;

            self.vertices.push(top_left);
            self.vertices.push(top_right);
            self.vertices.push(bottom_left);
            self.vertices.push(bottom_right);

            self.indices.extend_from_slice(&[
                base,
                base + 2,
                base + 1,
                base + 1,
                base + 2,
                base + 3,
            ]);

            // For caching, store vertices relative to (0, 0)
            let cache_base = cached_vertices.len() as u32;

            cached_vertices.push(Vertex {
                position: Vector3::new(glyph.x, glyph.y, 0.0),
                color,
                uv_coords: Vector2::new(uv_x, uv_y),
            });

            cached_vertices.push(Vertex {
                position: Vector3::new(glyph.x + glyph.width as f32, glyph.y, 0.0),
                color,
                uv_coords: Vector2::new(uv_x + uv_w, uv_y),
            });

            cached_vertices.push(Vertex {
                position: Vector3::new(glyph.x, glyph.y + glyph.height as f32, 0.0),
                color,
                uv_coords: Vector2::new(uv_x, uv_y + uv_h),
            });

            cached_vertices.push(Vertex {
                position: Vector3::new(
                    glyph.x + glyph.width as f32,
                    glyph.y + glyph.height as f32,
                    0.0,
                ),
                color,
                uv_coords: Vector2::new(uv_x + uv_w, uv_y + uv_h),
            });

            cached_indices.extend_from_slice(&[
                cache_base,
                cache_base + 2,
                cache_base + 1,
                cache_base + 1,
                cache_base + 2,
                cache_base + 3,
            ]);
        }

        if !cached_vertices.is_empty() {
            cache.insert(text, (cached_vertices, cached_indices));
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
