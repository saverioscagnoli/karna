mod batcher;
mod handle;

use crate::{
    Camera, color::Color, immediate::batcher::Batcher, immediate_shader, traits::LayoutDescriptor,
    vertex::Vertex,
};
use assets::{AssetServer, Font, Image};
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use macros::{Get, Set};
use math::{Vector2, Vector4};

pub use handle::*;
use utils::{FastHashMap, Handle, Label};

#[derive(Get, Set)]
pub struct ImmediateRenderer {
    triangle_batcher: Batcher,

    pub(crate) draw_color: Color,
    text_layout: Layout,
    char_cache: FastHashMap<u32, FastHashMap<char, (Vec<Vertex>, Vec<u32>)>>,
}

impl ImmediateRenderer {
    pub(crate) fn new(
        surface_format: wgpu::TextureFormat,
        camera: &Camera,
        atlas_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let triangle_pipeline = immediate_shader()
            .pipeline_builder()
            .label("Immediate Triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), atlas_bgl],
                &[Vertex::desc()],
            );

        let triangle_batcher = Batcher::new(triangle_pipeline);

        Self {
            draw_color: Color::Black,
            triangle_batcher,
            text_layout: Layout::new(CoordinateSystem::PositiveYDown),
            char_cache: FastHashMap::default(),
        }
    }

    #[inline]
    pub fn fill_rect(&mut self, pos: Vector2, w: f32, h: f32, assets: &AssetServer) {
        let color: Vector4 = self.draw_color.into();

        // Get white pixel UV coords for solid color rendering
        let (uv_x, uv_y, uv_w, uv_h, _, _) = assets.get_white_uv_coords();
        let uv_center: Vector2 = [uv_x + uv_w * 0.5, uv_y + uv_h * 0.5].into();

        let base = self.triangle_batcher.vertices.len() as u32;

        let [x, y] = pos.into();

        self.triangle_batcher.vertices.extend_from_slice(&[
            Vertex {
                position: [x, y, 0.0].into(),
                color,
                uv: uv_center,
            },
            Vertex {
                position: [x + w, y, 0.0].into(),
                color,
                uv: uv_center,
            },
            Vertex {
                position: [x + w, y + h, 0.0].into(),
                color,
                uv: uv_center,
            },
            Vertex {
                position: [x, y + h, 0.0].into(),
                color,
                uv: uv_center,
            },
        ]);

        self.triangle_batcher.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    #[inline]
    pub fn draw_image(&mut self, image: Handle<Image>, pos: Vector2, assets: &AssetServer) {
        let color: Vector4 = Color::White.into();

        let (uv_x, uv_y, uv_w, uv_h, w, h) = assets.get_texture_uv(image);

        // Define the four UV corners (not the center!)
        let uv_top_left: Vector2 = [uv_x, uv_y].into();
        let uv_top_right: Vector2 = [uv_x + uv_w, uv_y].into();
        let uv_bottom_right: Vector2 = [uv_x + uv_w, uv_y + uv_h].into();
        let uv_bottom_left: Vector2 = [uv_x, uv_y + uv_h].into();

        let base = self.triangle_batcher.vertices.len() as u32;

        let [x, y] = pos.into();

        self.triangle_batcher.vertices.extend_from_slice(&[
            Vertex {
                position: [x, y, 0.0].into(),
                color,
                uv: uv_top_left, // Top-left
            },
            Vertex {
                position: [x + w, y, 0.0].into(),
                color,
                uv: uv_top_right, // Top-right
            },
            Vertex {
                position: [x + w, y + h, 0.0].into(),
                color,
                uv: uv_bottom_right, // Bottom-right
            },
            Vertex {
                position: [x, y + h, 0.0].into(),
                color,
                uv: uv_bottom_left, // Bottom-left
            },
        ]);

        self.triangle_batcher.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    #[inline]
    pub fn draw_text(
        &mut self,
        handle: Handle<Font>,
        text: &str,
        x: f32,
        y: f32,
        assets: &AssetServer,
    ) {
        let color: Vector4 = self.draw_color.into();
        let font = assets.get_font(handle).expect("Failed to get font");

        self.text_layout.clear();
        self.text_layout.append(
            &[font.inner()],
            &TextStyle::new(text, font.size() as f32, 0),
        );

        let glyphs = self.text_layout.glyphs();
        let cache = self
            .char_cache
            .entry(handle.index())
            .or_insert_with(FastHashMap::default);

        for glyph in glyphs {
            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            let ch = glyph.parent;

            if let Some((cached_verts, cached_indices)) = cache.get(&ch) {
                let base_vertex = self.triangle_batcher.vertices.len() as u32;

                for mut vertex in cached_verts.iter().copied() {
                    vertex.position[0] += x + glyph.x;
                    vertex.position[1] += y + glyph.y;
                    vertex.color = color;

                    self.triangle_batcher.vertices.push(vertex);
                }

                for index in cached_indices {
                    self.triangle_batcher.indices.push(*index + base_vertex);
                }
            } else {
                // Create new geometry and cache it
                let (uv_x, uv_y, uv_w, uv_h, _, _) = assets.get_glyph_uv(handle, ch);

                let screen_x = x + glyph.x;
                let screen_y = y + glyph.y;
                let w = glyph.width as f32;
                let h = glyph.height as f32;

                let base = self.triangle_batcher.vertices.len() as u32;

                // Define the four UV corners
                let uv_top_left: Vector2 = [uv_x, uv_y].into();
                let uv_top_right: Vector2 = [uv_x + uv_w, uv_y].into();
                let uv_bottom_right: Vector2 = [uv_x + uv_w, uv_y + uv_h].into();
                let uv_bottom_left: Vector2 = [uv_x, uv_y + uv_h].into();

                self.triangle_batcher.vertices.extend_from_slice(&[
                    Vertex {
                        position: [screen_x, screen_y, 0.0].into(),
                        color,
                        uv: uv_top_left,
                    },
                    Vertex {
                        position: [screen_x + w, screen_y, 0.0].into(),
                        color,
                        uv: uv_top_right,
                    },
                    Vertex {
                        position: [screen_x + w, screen_y + h, 0.0].into(),
                        color,
                        uv: uv_bottom_right,
                    },
                    Vertex {
                        position: [screen_x, screen_y + h, 0.0].into(),
                        color,
                        uv: uv_bottom_left,
                    },
                ]);

                self.triangle_batcher.indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base,
                    base + 2,
                    base + 3,
                ]);

                // Cache relative to (0, 0) for reuse
                let cached_color = Color::White.into();
                let cached_vertices = vec![
                    Vertex {
                        position: [0.0, 0.0, 0.0].into(),
                        color: cached_color,
                        uv: uv_top_left,
                    },
                    Vertex {
                        position: [w, 0.0, 0.0].into(),
                        color: cached_color,
                        uv: uv_top_right,
                    },
                    Vertex {
                        position: [w, h, 0.0].into(),
                        color: cached_color,
                        uv: uv_bottom_right,
                    },
                    Vertex {
                        position: [0.0, h, 0.0].into(),
                        color: cached_color,
                        uv: uv_bottom_left,
                    },
                ];

                cache.insert(ch, (cached_vertices, vec![0, 1, 2, 0, 2, 3]));
            }
        }
    }

    #[inline]
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.triangle_batcher.present(render_pass);
    }
}
