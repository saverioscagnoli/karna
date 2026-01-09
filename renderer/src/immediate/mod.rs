mod batcher;
mod handle;

use std::borrow::Borrow;

use crate::{
    Camera,
    color::Color,
    immediate::batcher::Batcher,
    immediate_circle_shader, immediate_shader,
    traits::LayoutDescriptor,
    vertex::{CircleVertex, Vertex},
};
use assets::{AssetServer, AssetServerGuard, Font, Image};
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use macros::{Get, Set};
use math::{Vector2, Vector3, Vector4};
use utils::{FastHashMap, Handle, label};

pub use handle::*;

#[derive(Get, Set)]
pub struct ImmediateRenderer {
    point_batcher: Batcher<Vertex>,
    linelist_batcher: Batcher<Vertex>,
    linestrip_batcher: Batcher<Vertex>,
    triangle_batcher: Batcher<Vertex>,
    circle_batcher: Batcher<CircleVertex>,

    pub(crate) draw_color: Color,
    text_layout: Layout,
    char_cache: FastHashMap<u32, FastHashMap<char, (Vec<Vertex>, Vec<u32>)>>,
}

impl ImmediateRenderer {
    pub(crate) fn new(
        surface_format: wgpu::TextureFormat,
        camera: &Camera,
        assets: &AssetServerGuard<'_>,
    ) -> Self {
        let point_pipeline = immediate_shader()
            .pipeline_builder()
            .label("Immediate Pixel pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::PointList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.atlas_bgl()],
                &[Vertex::desc()],
            );

        let linelist_pipeline = immediate_shader()
            .pipeline_builder()
            .label("Immediate Pixel pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::LineList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.atlas_bgl()],
                &[Vertex::desc()],
            );

        let linestrip_pipeline = immediate_shader()
            .pipeline_builder()
            .label("Immediate Pixel pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::LineStrip)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.atlas_bgl()],
                &[Vertex::desc()],
            );

        let triangle_pipeline = immediate_shader()
            .pipeline_builder()
            .label("Immediate Triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.atlas_bgl()],
                &[Vertex::desc()],
            );

        let circle_pipeline = immediate_circle_shader()
            .pipeline_builder()
            .label("Immediate Circle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.atlas_bgl()],
                &[CircleVertex::desc()],
            );

        let point_batcher = Batcher::new(point_pipeline);
        let linelist_batcher = Batcher::new(linelist_pipeline);
        let linestrip_batcher = Batcher::new(linestrip_pipeline);
        let triangle_batcher = Batcher::new(triangle_pipeline);

        let circle_batcher = Batcher::new(circle_pipeline);

        Self {
            draw_color: Color::White,
            point_batcher,
            linelist_batcher,
            linestrip_batcher,
            triangle_batcher,
            circle_batcher,
            text_layout: Layout::new(CoordinateSystem::PositiveYDown),
            char_cache: FastHashMap::default(),
        }
    }

    #[inline]
    pub fn draw_point(&mut self, pos: Vector2) {
        let color: Vector4 = self.draw_color.into();
        let base = self.point_batcher.vertices.len() as u32;

        self.point_batcher
            .vertices
            .push(Vertex::new(pos.extend(0.0), color, Vector2::zeros()));

        self.point_batcher.indices.push(base);
    }

    #[inline]
    pub fn draw_line(&mut self, p1: Vector2, p2: Vector2) {
        let color: Vector4 = self.draw_color.into();
        let base = self.linelist_batcher.vertices.len() as u32;

        self.linelist_batcher.vertices.extend_from_slice(&[
            Vertex::new(p1.extend(0.0), color, Vector2::zeros()),
            Vertex::new(p2.extend(0.0), color, Vector2::zeros()),
        ]);

        self.linelist_batcher
            .indices
            .extend_from_slice(&[base, base + 1]);
    }

    #[inline]
    pub fn draw_lines<I>(&mut self, points: I)
    where
        I: IntoIterator,
        I::Item: Borrow<(Vector2, Vector2)>,
    {
        let color: Vector4 = self.draw_color.into();
        let base = self.linestrip_batcher.vertices.len() as u32;

        let mut vertex_count = 0;

        for item in points {
            let (p1, p2) = *item.borrow();

            self.linestrip_batcher.vertices.extend_from_slice(&[
                Vertex::new(p1.extend(0.0), color, Vector2::zeros()),
                Vertex::new(p2.extend(0.0), color, Vector2::zeros()),
            ]);

            vertex_count += 2;
        }

        self.linestrip_batcher
            .indices
            .extend(base..base + vertex_count);
    }

    #[inline]
    pub fn fill_rect(&mut self, pos: Vector2, w: f32, h: f32, assets: &AssetServerGuard<'_>) {
        let color: Vector4 = self.draw_color.into();

        // Get white pixel UV coords for solid color rendering
        let (uv_x, uv_y, uv_w, uv_h, _, _) = assets.get_white_uv_coords();
        let uv_center: Vector2 = [uv_x + uv_w * 0.5, uv_y + uv_h * 0.5].into();

        let base = self.triangle_batcher.vertices.len() as u32;

        self.triangle_batcher.vertices.extend_from_slice(&[
            Vertex::new(pos.extend(0.0), color, uv_center),
            Vertex::new(Vector3::new(pos.x + w, pos.y, 0.0), color, uv_center),
            Vertex::new(Vector3::new(pos.x + w, pos.y + h, 0.0), color, uv_center),
            Vertex::new(Vector3::new(pos.x, pos.y + h, 0.0), color, uv_center),
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
    pub fn fill_circle(&mut self, center: Vector2, radius: f32) {
        let color: Vector4 = self.draw_color.into();
        let base = self.circle_batcher.vertices.len() as u32;

        self.circle_batcher.vertices.extend_from_slice(&[
            CircleVertex::new((center - radius).extend(0.0), color, center, radius),
            CircleVertex::new(
                Vector3::new(center.x + radius, center.y - radius, 0.0),
                color,
                center,
                radius,
            ),
            CircleVertex::new((center + radius).extend(0.0), color, center, radius),
            CircleVertex::new(
                Vector3::new(center.x - radius, center.y + radius, 0.0),
                color,
                center,
                radius,
            ),
        ]);

        self.circle_batcher.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    #[inline]
    pub fn draw_image(
        &mut self,
        image: Handle<Image>,
        pos: Vector2,
        assets: &AssetServerGuard<'_>,
    ) {
        let color: Vector4 = Color::White.into();

        let (uv_x, uv_y, uv_w, uv_h, w, h) = assets.get_texture_uv(image);

        let uv_top_left: Vector2 = [uv_x, uv_y].into();
        let uv_top_right: Vector2 = [uv_x + uv_w, uv_y].into();
        let uv_bottom_right: Vector2 = [uv_x + uv_w, uv_y + uv_h].into();
        let uv_bottom_left: Vector2 = [uv_x, uv_y + uv_h].into();

        let base = self.triangle_batcher.vertices.len() as u32;

        self.triangle_batcher.vertices.extend_from_slice(&[
            Vertex::new(pos.extend(0.0), color, uv_top_left),
            Vertex::new(Vector3::new(pos.x + w, pos.y, 0.0), color, uv_top_right),
            Vertex::new(
                Vector3::new(pos.x + w, pos.y + h, 0.0),
                color,
                uv_bottom_right,
            ),
            Vertex::new(Vector3::new(pos.x, pos.y + h, 0.0), color, uv_bottom_left),
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
    pub fn draw_atlas(&mut self, pos: Vector2, assets: &AssetServerGuard<'_>) {
        let color: Vector4 = Color::White.into();

        let (uv_x, uv_y, uv_w, uv_h, w, h) = assets.get_texture_uv_by_label(&label!("_atlas"));

        let uv_top_left: Vector2 = [uv_x, uv_y].into();
        let uv_top_right: Vector2 = [uv_x + uv_w, uv_y].into();
        let uv_bottom_right: Vector2 = [uv_x + uv_w, uv_y + uv_h].into();
        let uv_bottom_left: Vector2 = [uv_x, uv_y + uv_h].into();

        let base = self.triangle_batcher.vertices.len() as u32;

        self.triangle_batcher.vertices.extend_from_slice(&[
            Vertex::new(pos.extend(0.0), color, uv_top_left),
            Vertex::new(Vector3::new(pos.x + w, pos.y, 0.0), color, uv_top_right),
            Vertex::new(
                Vector3::new(pos.x + w, pos.y + h, 0.0),
                color,
                uv_bottom_right,
            ),
            Vertex::new(Vector3::new(pos.x, pos.y + h, 0.0), color, uv_bottom_left),
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
        assets: &AssetServerGuard<'_>,
    ) {
        let color: Vector4 = self.draw_color.into();
        let font = assets.get_font(handle);

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

                let uv_top_left: Vector2 = [uv_x, uv_y].into();
                let uv_top_right: Vector2 = [uv_x + uv_w, uv_y].into();
                let uv_bottom_right: Vector2 = [uv_x + uv_w, uv_y + uv_h].into();
                let uv_bottom_left: Vector2 = [uv_x, uv_y + uv_h].into();

                self.triangle_batcher.vertices.extend_from_slice(&[
                    Vertex::new(Vector3::new(screen_x, screen_y, 0.0), color, uv_top_left),
                    Vertex::new(
                        Vector3::new(screen_x + w, screen_y, 0.0),
                        color,
                        uv_top_right,
                    ),
                    Vertex::new(
                        Vector3::new(screen_x + w, screen_y + h, 0.0),
                        color,
                        uv_bottom_right,
                    ),
                    Vertex::new(
                        Vector3::new(screen_x, screen_y + h, 0.0),
                        color,
                        uv_bottom_left,
                    ),
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
                    Vertex::new(Vector3::new(0.0, 0.0, 0.0), cached_color, uv_top_left),
                    Vertex::new(Vector3::new(w, 0.0, 0.0), cached_color, uv_top_right),
                    Vertex::new(Vector3::new(w, h, 0.0), cached_color, uv_bottom_right),
                    Vertex::new(Vector3::new(0.0, h, 0.0), cached_color, uv_bottom_left),
                ];

                cache.insert(ch, (cached_vertices, vec![0, 1, 2, 0, 2, 3]));
            }
        }
    }

    #[inline]
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.point_batcher.present(render_pass);
        self.linelist_batcher.present(render_pass);
        self.linestrip_batcher.present(render_pass);
        self.triangle_batcher.present(render_pass);
        self.circle_batcher.present(render_pass);
    }
}
