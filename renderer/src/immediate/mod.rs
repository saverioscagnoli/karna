mod batcher;
mod draw_handle;
mod vertex;

use assets::AssetServerGuard;
use assets::Font;
use assets::Image;
pub use draw_handle::Draw;
use fontdue::layout::CoordinateSystem;
use fontdue::layout::Layout;
use fontdue::layout::TextStyle;
use logging::warn;
use math::Matrix4;
use math::Vector2;
use math::Vector3;
use math::Vector4;
use utils::Handle;

use crate::Color;
use crate::immediate::batcher::Batcher;
use crate::immediate::vertex::ImmediateCircleVertex;
use crate::immediate::vertex::ImmediateVertex;
use crate::immediate_circle_shader;
use crate::immediate_shader;

#[derive(Debug, Clone, Copy)]
struct RenderState {
    draw_color: Vector4,
    transform: Matrix4,
    depth: f32,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            draw_color: Color::White.into(),
            transform: Matrix4::identity(),
            depth: 0.0,
        }
    }
}

pub struct ImmediateRenderer {
    current_state: RenderState,
    state_stack: Vec<RenderState>,

    text_layout: Layout,
    point_batcher: Batcher<ImmediateVertex>,
    line_batcher: Batcher<ImmediateVertex>,
    triangle_batcher: Batcher<ImmediateVertex>,
    circle_batcher: Batcher<ImmediateCircleVertex>,
}

impl ImmediateRenderer {
    pub fn new(
        surface_format: wgpu::TextureFormat,
        camera_bgl: &wgpu::BindGroupLayout,
        atlas_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let make_immediate_pipeline = |label, topology| {
            immediate_shader()
                .pipeline_builder()
                .label(label)
                .vertex_entry("vs_main")
                .fragment_entry("fs_main")
                .topology(topology)
                .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
                .build(
                    surface_format,
                    &[camera_bgl, atlas_bgl],
                    &[ImmediateVertex::desc()],
                )
        };

        let point_pipeline = make_immediate_pipeline(
            "Immediate Point pipeline",
            wgpu::PrimitiveTopology::PointList,
        );
        let line_pipeline =
            make_immediate_pipeline("Immediate Line pipeline", wgpu::PrimitiveTopology::LineList);
        let triangle_pipeline = make_immediate_pipeline(
            "Immediate Triangle pipeline",
            wgpu::PrimitiveTopology::TriangleList,
        );

        let circle_pipeline = immediate_circle_shader()
            .pipeline_builder()
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera_bgl, atlas_bgl],
                &[ImmediateCircleVertex::desc()],
            );

        Self {
            current_state: RenderState::default(),
            state_stack: Vec::new(),
            text_layout: Layout::new(CoordinateSystem::PositiveYDown),
            point_batcher: Batcher::new(point_pipeline),
            line_batcher: Batcher::new(line_pipeline),
            triangle_batcher: Batcher::new(triangle_pipeline),
            circle_batcher: Batcher::new(circle_pipeline),
        }
    }

    #[inline]
    pub fn push_state(&mut self) {
        self.state_stack.push(self.current_state);
    }

    #[inline]
    pub fn pop_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.current_state = state;
        } else {
            warn!("Immediate renderer: popped a state without pushing first");
        }
    }

    #[inline]
    pub fn draw_color(&self) -> Vector4 {
        self.current_state.draw_color
    }
    #[inline]
    pub fn set_draw_color(&mut self, color: Vector4) {
        self.current_state.draw_color = color;
    }

    #[inline]
    pub fn depth(&self) -> f32 {
        self.current_state.depth
    }
    #[inline]
    pub fn set_depth(&mut self, depth: f32) {
        self.current_state.depth = depth;
    }

    #[inline]
    pub fn translate(&mut self, x: f32, y: f32) {
        self.current_state.transform =
            self.current_state.transform * Matrix4::from_translation(Vector3::new(x, y, 0.0));
    }

    #[inline]
    pub fn rotate(&mut self, angle_radians: f32) {
        self.current_state.transform =
            self.current_state.transform * Matrix4::from_rotation_z(angle_radians);
    }

    #[inline]
    pub fn scale(&mut self, x: f32, y: f32) {
        self.current_state.transform =
            self.current_state.transform * Matrix4::from_scale(Vector3::new(x, y, 1.0));
    }

    /// Transform a 2-D point through the current matrix, returning a 3-D position
    /// that carries the configured depth in the z component.
    #[inline]
    fn tp(&self, x: f32, y: f32) -> Vector3 {
        let pos = Vector4::new(x, y, 0.0, 1.0);
        let transformed = self.current_state.transform * pos;

        Vector3::new(transformed.x, transformed.y, self.current_state.depth)
    }

    /// Build an [`ImmediateVertex`] at `(x, y)` with the current color and the
    /// given atlas UV.
    #[inline]
    fn vert(&self, x: f32, y: f32, uv: Vector2) -> ImmediateVertex {
        let p = self.tp(x, y);
        ImmediateVertex::new(p.x, p.y, p.z, self.current_state.draw_color, uv)
    }

    /// Push `vertices` into `batcher` and append indices that offset each
    /// element in `pattern` by the current vertex base.
    #[inline]
    fn push_verts<V: Copy>(batcher: &mut Batcher<V>, vertices: &[V], pattern: &[u32]) {
        let base = batcher.vertices.len() as u32;
        batcher.vertices.extend_from_slice(vertices);
        batcher.indices.extend(pattern.iter().map(|i| base + i));
    }

    #[inline]
    pub fn push_point(&mut self, assets: &AssetServerGuard, x: f32, y: f32) {
        let white = assets.white_pixel_handle();
        let uv = assets.get_image(white).uv.xy();
        let v = self.vert(x, y, uv);

        Self::push_verts(&mut self.point_batcher, &[v], &[0]);
    }

    #[inline]
    pub fn push_line(&mut self, assets: &AssetServerGuard, x1: f32, y1: f32, x2: f32, y2: f32) {
        let white = assets.white_pixel_handle();
        let uv = assets.get_image(white).uv.xy();
        let verts = [self.vert(x1, y1, uv), self.vert(x2, y2, uv)];

        Self::push_verts(&mut self.line_batcher, &verts, &[0, 1]);
    }

    #[inline]
    fn push_quad_uvs(&mut self, x: f32, y: f32, w: f32, h: f32, uv: Vector4) {
        let uv_tl = uv.xy();
        let uv_tr = uv_tl + Vector2::new(uv.z, 0.0);
        let uv_bl = uv_tl + Vector2::new(0.0, uv.w);
        let uv_br = uv_tl + Vector2::new(uv.z, uv.w);

        let verts = [
            self.vert(x, y, uv_tl),
            self.vert(x + w, y, uv_tr),
            self.vert(x, y + h, uv_bl),
            self.vert(x + w, y + h, uv_br),
        ];

        Self::push_verts(&mut self.triangle_batcher, &verts, &[0, 1, 2, 2, 1, 3]);
    }

    #[inline]
    pub fn push_rect(&mut self, assets: &AssetServerGuard, x: f32, y: f32, w: f32, h: f32) {
        let white = assets.white_pixel_handle();
        let uv = assets.get_image(white).uv;

        self.push_quad_uvs(x, y, w, h, uv);
    }

    #[inline]
    pub fn push_rect_outline(&mut self, assets: &AssetServerGuard, x: f32, y: f32, w: f32, h: f32) {
        let white = assets.white_pixel_handle();
        let uv = assets.get_image(white).uv.xy();

        let verts = [
            self.vert(x, y, uv),         // 0 TL
            self.vert(x + w, y, uv),     // 1 TR
            self.vert(x + w, y + h, uv), // 2 BR
            self.vert(x, y + h, uv),     // 3 BL
        ];

        Self::push_verts(&mut self.line_batcher, &verts, &[0, 1, 1, 2, 2, 3, 3, 0]);
    }

    /// Push a quad with a custom UV rect into the atlas.
    ///
    /// `uv` is expressed as `(u0, v0, du, dv)` in atlas space.
    #[inline]
    pub fn push_quad_uv(&mut self, x: f32, y: f32, w: f32, h: f32, uv: Vector4) {
        self.push_quad_uvs(x, y, w, h, uv);
    }

    /// Push a sub-region of an atlas `image`.
    ///
    /// - `src_*` are in *pixels* within the source image (not the atlas).
    /// - `dst_w` / `dst_h` are destination size in world pixels.
    /// - `flip_x` / `flip_y` flip by mirroring UVs (no transform side effects).
    #[inline]
    pub fn push_quad_region_ex(
        &mut self,
        image: Handle<Image>,
        assets: &AssetServerGuard,
        x: f32,
        y: f32,
        dst_w: f32,
        dst_h: f32,
        src_x: u32,
        src_y: u32,
        src_w: u32,
        src_h: u32,
        flip_x: bool,
        flip_y: bool,
    ) {
        let img = assets.get_image(image);

        // Convert pixel region inside `img` into atlas UVs.
        let iw = img.size.width.max(1) as f32;
        let ih = img.size.height.max(1) as f32;

        let mut u0 = img.uv.x + (src_x as f32 / iw) * img.uv.z;
        let mut v0 = img.uv.y + (src_y as f32 / ih) * img.uv.w;
        let mut du = (src_w as f32 / iw) * img.uv.z;
        let mut dv = (src_h as f32 / ih) * img.uv.w;

        if flip_x {
            u0 += du;
            du = -du;
        }
        if flip_y {
            v0 += dv;
            dv = -dv;
        }

        self.push_quad_uvs(x, y, dst_w, dst_h, Vector4::new(u0, v0, du, dv));
    }

    /// Convenience wrapper for the common case where destination size matches
    /// the region size and no flipping is required.
    #[inline]
    pub fn push_quad_region(
        &mut self,
        image: Handle<Image>,
        assets: &AssetServerGuard,
        x: f32,
        y: f32,
        src_x: u32,
        src_y: u32,
        src_w: u32,
        src_h: u32,
    ) {
        self.push_quad_region_ex(
            image,
            assets,
            x,
            y,
            src_w as f32,
            src_h as f32,
            src_x,
            src_y,
            src_w,
            src_h,
            false,
            false,
        );
    }

    #[inline]
    pub fn push_quad(&mut self, image: Handle<Image>, assets: &AssetServerGuard, x: f32, y: f32) {
        let image = assets.get_image(image);
        let w = image.size.width as f32;
        let h = image.size.height as f32;

        self.push_quad_uvs(x, y, w, h, image.uv);
    }

    #[inline]
    pub fn push_triangle(
        &mut self,
        assets: &AssetServerGuard,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    ) {
        let white = assets.white_pixel_handle();
        let uv = assets.get_image(white).uv.xy();
        let verts = [
            self.vert(x1, y1, uv),
            self.vert(x2, y2, uv),
            self.vert(x3, y3, uv),
        ];

        Self::push_verts(&mut self.triangle_batcher, &verts, &[0, 1, 2]);
    }

    #[inline]
    pub fn push_circle(&mut self, x: f32, y: f32, radius: f32) {
        let center_world = self.tp(x, y);
        let color = self.current_state.draw_color;
        let center_2d = Vector2::new(center_world.x, center_world.y);

        let p_edge = self.tp(x + radius, y);
        let transformed_radius = (Vector2::new(p_edge.x, p_edge.y) - center_2d).length();

        let verts = [
            ImmediateCircleVertex::new(
                self.tp(x - radius, y - radius),
                color,
                center_2d,
                transformed_radius,
            ),
            ImmediateCircleVertex::new(
                self.tp(x + radius, y - radius),
                color,
                center_2d,
                transformed_radius,
            ),
            ImmediateCircleVertex::new(
                self.tp(x + radius, y + radius),
                color,
                center_2d,
                transformed_radius,
            ),
            ImmediateCircleVertex::new(
                self.tp(x - radius, y + radius),
                color,
                center_2d,
                transformed_radius,
            ),
        ];

        Self::push_verts(&mut self.circle_batcher, &verts, &[0, 1, 2, 0, 2, 3]);
    }

    #[inline]
    pub fn push_text(
        &mut self,
        font: Handle<Font>,
        text: &str,
        assets: &AssetServerGuard,
        x: f32,
        y: f32,
    ) {
        let font = assets.get_font(font);

        self.text_layout.clear();
        self.text_layout.append(
            &[font.inner()],
            &TextStyle::new(text, font.size() as f32, 0),
        );

        // Collect first to avoid borrowing `self` through `font` while also
        // calling `push_textured_quad`.
        let glyphs: Vec<_> = self
            .text_layout
            .glyphs()
            .iter()
            .filter(|g| g.width > 0 && g.height > 0)
            .map(|g| (font.glyph_image(g.parent), g.x, g.y))
            .collect();

        for (image, gx, gy) in glyphs {
            self.push_quad(image, assets, x + gx, y + gy);
        }
    }

    /// Flush all batchers to the render pass.
    ///
    /// State (color, transform, depth) is intentionally **not** reset here —
    /// use [`push_state`] / [`pop_state`] to scope mutations, or reset
    /// manually.  Resetting silently caused hard-to-find bugs when drawing
    /// spanned across multiple subsystems in the same frame.
    #[inline]
    pub fn present<'pass>(&'pass mut self, render_pass: &mut wgpu::RenderPass<'pass>) {
        self.point_batcher.present(render_pass);
        self.line_batcher.present(render_pass);
        self.triangle_batcher.present(render_pass);
        self.circle_batcher.present(render_pass);

        self.current_state = RenderState::default();
    }
}
