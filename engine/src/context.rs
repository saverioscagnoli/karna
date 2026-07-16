use std::ops::Index;
use std::ops::IndexMut;

use assets::AssetServer;
use assets::AssetsRead;
use assets::Font;
use assets::Image;
use glyph_brush_layout::GlyphPositioner;
use logging::warn;
use renderer::Camera;
use renderer::CircleVertex;
use renderer::Color;
use renderer::FramePacket;
use renderer::Geometry;
use renderer::Layer;
use renderer::Material;
use renderer::MaterialDesc;
use renderer::Mesh;
use renderer::Projection;
use renderer::Renderer;
use renderer::Vertex;
use utils::Handle;

use crate::SceneManager;
use crate::Time;
use crate::Window;
use crate::input::Input;
use crate::mixer::Mixer;
use crate::resources::Resources;

#[cfg(feature = "net")]
use crate::net::Net;

pub struct Cameras {
    world: Camera,
    ui: Camera,
    debug: Camera,
}

impl Index<Layer> for Cameras {
    type Output = Camera;

    fn index(&self, l: Layer) -> &Self::Output {
        match l {
            Layer::World => &self.world,
            Layer::Ui => &self.ui,
            Layer::Debug => &self.debug,
        }
    }
}

impl IndexMut<Layer> for Cameras {
    fn index_mut(&mut self, l: Layer) -> &mut Self::Output {
        match l {
            Layer::World => &mut self.world,
            Layer::Ui => &mut self.ui,
            Layer::Debug => &mut self.debug,
        }
    }
}

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub scenes: SceneManager,
    pub mixer: Mixer,
    pub assets: AssetServer,
    pub resources: Resources,
    pub renderer: Renderer,

    // Frame stuff
    pub packet: FramePacket,
    pub cameras: Cameras,

    #[cfg(feature = "net")]
    pub net: Net,
}

impl WindowContext {
    pub fn new(window: Window, assets: AssetServer, renderer: Renderer) -> Self {
        let view = window.size();

        Self {
            window,
            time: Time::new(),
            input: Input::new(),
            scenes: SceneManager::new(),
            mixer: Mixer::new(assets.reader()),
            assets,
            resources: Resources::new(),
            renderer,

            packet: FramePacket::default(),
            cameras: Cameras {
                world: Camera::new(Projection::standard_2d(view)),
                ui: Camera::new(Projection::standard_2d(view)),
                debug: Camera::new(Projection::standard_2d(view)),
            },

            #[cfg(feature = "net")]
            net: Net::new(),
        }
    }
}

pub struct ContextMut<'a> {
    pub window: &'a Window, // Cannot be mutated, no sense in making it &mut
    pub time: &'a mut Time,
    pub input: &'a mut Input,
    pub scenes: &'a mut SceneManager,
    pub mixer: &'a Mixer,        // Cannot be mutated, no sense making it &mut
    pub assets: &'a AssetServer, // Uses interior mutability, no sense making it &mut
    pub resources: &'a mut Resources,

    #[cfg(feature = "net")]
    pub net: &'a mut Net,
}

impl WindowContext {
    pub fn split_mut<'a>(&'a mut self) -> (ContextMut<'a>, SceneHandle<'a>) {
        let Self {
            window,
            time,
            input,
            scenes,
            mixer,
            assets,
            resources,
            renderer,
            packet,
            cameras,

            #[cfg(feature = "net")]
            net,
        } = self;

        (
            ContextMut {
                window,
                time,
                input,
                scenes,
                mixer,
                assets,
                resources,

                #[cfg(feature = "net")]
                net,
            },
            SceneHandle {
                renderer: renderer,
                active_layer: Layer::World,
                packet,
                cameras,
            },
        )
    }

    pub fn split<'a>(&'a mut self, imgui: &'a imgui::Ui) -> (ContextMut<'a>, Draw<'a>) {
        (
            ContextMut {
                window: &self.window,
                time: &mut self.time,
                input: &mut self.input,
                scenes: &mut self.scenes,
                mixer: &mut self.mixer,
                assets: &self.assets,
                resources: &mut self.resources,

                #[cfg(feature = "net")]
                net: &mut self.net,
            },
            Draw::new(&mut self.packet, self.assets.read(), imgui),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct DrawState {
    draw_color: math::Vector4<f32>,
    transform: math::Matrix4<f32>,
    depth: f32,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            draw_color: math::Vector4::new(1.0, 1.0, 1.0, 1.0),
            transform: math::Matrix4::identity(),
            depth: 0.0,
        }
    }
}

pub struct Draw<'a> {
    packet: &'a mut FramePacket,
    state_stack: Vec<DrawState>,
    current_state: DrawState,
    active_layer: Layer,
    assets: AssetsRead<'a>,
    imgui: &'a imgui::Ui,
    white_uv: math::Vector2<f32>,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(
        packet: &'a mut FramePacket,
        assets: AssetsRead<'a>,
        imgui: &'a imgui::Ui,
    ) -> Self {
        let white_uv = assets.get_image(assets.white_handle()).uv.xy();

        Self {
            packet,
            state_stack: Vec::new(),
            current_state: DrawState::default(),
            active_layer: Layer::World,
            assets,
            imgui,
            white_uv,
        }
    }

    pub fn push_state(&mut self) {
        self.state_stack.push(self.current_state);
    }

    pub fn pop_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.current_state = state;
        } else {
            warn!("Draw: popped a render state without pushing first");
        }
    }

    pub fn viewport(&self) -> math::Size<u32> {
        self.packet.viewport
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.active_layer = layer;
    }

    pub fn clear_color(&self) -> Color {
        self.packet.clear_color.into()
    }

    pub fn set_clear_color<C: Into<math::Vector4<f32>>>(&mut self, color: C) {
        self.packet.clear_color = color.into();
    }

    pub fn color(&self) -> Color {
        self.current_state.draw_color.into()
    }

    pub fn set_color<C: Into<math::Vector4<f32>>>(&mut self, color: C) {
        self.current_state.draw_color = color.into();
    }

    pub fn depth(&self) -> f32 {
        self.current_state.depth
    }

    pub fn set_depth(&mut self, d: f32) {
        self.current_state.depth = d;
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.current_state.transform =
            self.current_state
                .transform
                .matmul(&math::Matrix4::from_translation(math::Vector3::new(
                    x, y, 0.0,
                )));
    }

    pub fn rotate(&mut self, angle_rad: f32) {
        self.current_state.transform = self
            .current_state
            .transform
            .matmul(&math::Matrix4::from_rotation_z(angle_rad));
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.current_state.transform = self
            .current_state
            .transform
            .matmul(&math::Matrix4::from_scale(math::Vector3::new(x, y, 1.0)));
    }

    pub fn tp(&self, x: f32, y: f32) -> math::Vector3<f32> {
        let pos = math::Vector4::new(x, y, 0.0, 1.0);
        let t = self.current_state.transform.mul_vec(&pos);

        math::Vector3::new(t.x, t.y, self.current_state.depth)
    }

    fn vertex(&self, x: f32, y: f32, uv: math::Vector2<f32>) -> Vertex {
        let p = self.tp(x, y);
        Vertex::new(p, self.current_state.draw_color, uv)
    }

    fn circle_vertex(&self, x: f32, y: f32, center: math::Vector2<f32>, r: f32) -> CircleVertex {
        let p = self.tp(x, y);
        let c = self.tp(center.x, center.y);
        CircleVertex::new(
            p,
            self.current_state.draw_color,
            math::Vector2::new(c.x, c.y),
            r,
        )
    }

    // ---- primitives ----

    pub fn point(&mut self, x: f32, y: f32) {
        let v = [self.vertex(x, y, self.white_uv)];

        self.packet
            .layer_mut(self.active_layer)
            .points
            .push(&v, &[0]);
    }

    pub fn point_v<P>(&mut self, pos: P)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos: math::Vector2<f32> = pos.into();

        self.point(pos.x, pos.y);
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let v = [
            self.vertex(x1, y1, self.white_uv),
            self.vertex(x2, y2, self.white_uv),
        ];

        self.packet
            .layer_mut(self.active_layer)
            .lines
            .push(&v, &[0, 1]);
    }

    pub fn line_v<P, Q>(&mut self, p: P, q: Q)
    where
        P: Into<math::Vector2<f32>>,
        Q: Into<math::Vector2<f32>>,
    {
        let p: math::Vector2<f32> = p.into();
        let q: math::Vector2<f32> = q.into();

        self.line(p.x, p.y, q.x, q.y);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let v = [
            self.vertex(x, y, self.white_uv),
            self.vertex(x + w, y, self.white_uv),
            self.vertex(x, y + h, self.white_uv),
            self.vertex(x + w, y + h, self.white_uv),
        ];

        self.packet
            .layer_mut(self.active_layer)
            .triangles
            .push(&v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn rect_v<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<math::Vector2<f32>>,
        S: Into<math::Size<f32>>,
    {
        let pos: math::Vector2<f32> = pos.into();
        let size: math::Size<f32> = size.into();

        self.rect(pos.x, pos.y, size.width, size.height);
    }

    pub fn circle(&mut self, x: f32, y: f32, r: f32) {
        let c = math::Vector2::new(x, y);

        let v = [
            self.circle_vertex(x - r, y - r, c, r), // top-left
            self.circle_vertex(x + r, y - r, c, r), // top-right
            self.circle_vertex(x - r, y + r, c, r), // bottom-left
            self.circle_vertex(x + r, y + r, c, r), // bottom-right
        ];

        self.packet
            .layer_mut(self.active_layer)
            .circles
            .push(&v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn circle_v<P>(&mut self, pos: P, r: f32)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos: math::Vector2<f32> = pos.into();

        self.circle(pos.x, pos.y, r);
    }

    pub fn image(&mut self, image: Handle<Image>, x: f32, y: f32) {
        let image = self.assets.get_image(image);
        let size = image.size.as_f32();
        let uv = image.uv;
        let uv_min = uv.xy();
        let uv_max = math::Vector2::new(uv.x + uv.z, uv.y + uv.w);

        let v = [
            self.vertex(x, y, uv_min),
            self.vertex(x + size.width, y, math::Vector2::new(uv_max.x, uv_min.y)),
            self.vertex(x, y + size.height, math::Vector2::new(uv_min.x, uv_max.y)),
            self.vertex(x + size.width, y + size.height, uv_max),
        ];

        self.packet
            .layer_mut(self.active_layer)
            .triangles
            .push(&v, &[0, 1, 2, 2, 1, 3])
    }

    pub fn image_v<P>(&mut self, image: Handle<Image>, pos: P)
    where
        P: Into<math::Vector2<f32>>,
    {
        let pos: math::Vector2<f32> = pos.into();

        self.image(image, pos.x, pos.y);
    }

    pub fn text<T>(&mut self, font: Handle<Font>, text: T, x: f32, y: f32)
    where
        T: AsRef<str>,
    {
        let text = text.as_ref();
        let view = self.packet.viewport.as_f32();
        let font_asset = self.assets.get_font(font);

        let geometry = glyph_brush_layout::SectionGeometry {
            screen_position: (x, y),
            bounds: (view.width, view.height),
        };

        let sections = &[glyph_brush_layout::SectionText {
            text,
            scale: glyph_brush_layout::ab_glyph::PxScale::from(font_asset.size() as f32),
            font_id: glyph_brush_layout::FontId(0),
        }];

        let fonts = [font_asset.inner()];
        let glyphs = glyph_brush_layout::Layout::default_wrap()
            .calculate_glyphs(&fonts, &geometry, sections);

        for sg in &glyphs {
            let ch = text[sg.byte_index..]
                .chars()
                .next()
                .expect("Invalid byte index");
            if ch.is_whitespace() {
                continue; // whitespace has no cached glyph/quad, position already accounts for its advance
            }

            let info = self.assets.get_glyph(font, ch, font_asset.size());

            let draw_x = sg.glyph.position.x + info.bearing.x;
            let draw_y = sg.glyph.position.y + info.bearing.y;

            let uv = info.uv; // Vector4: xy = origin, zw = extent
            let uv_min = uv.xy();
            let uv_max = math::Vector2::new(uv.x + uv.z, uv.y + uv.w);

            let v = [
                self.vertex(draw_x, draw_y, uv_min),
                self.vertex(
                    draw_x + info.size.width,
                    draw_y,
                    math::Vector2::new(uv_max.x, uv_min.y),
                ),
                self.vertex(
                    draw_x,
                    draw_y + info.size.height,
                    math::Vector2::new(uv_min.x, uv_max.y),
                ),
                self.vertex(draw_x + info.size.width, draw_y + info.size.height, uv_max),
            ];

            self.packet
                .layer_mut(self.active_layer)
                .triangles
                .push(&v, &[0, 1, 2, 2, 1, 3]);
        }
    }

    pub fn debug_text<T>(&mut self, text: T, x: f32, y: f32)
    where
        T: AsRef<str>,
    {
        let debug_font = self.assets.debug_font_handle();

        self.text(debug_font, text, x, y);
    }

    pub fn measure_text<T>(&self, font: Handle<Font>, text: T) -> math::Size<f32>
    where
        T: AsRef<str>,
    {
        use glyph_brush_layout::ab_glyph::Font as _;
        use glyph_brush_layout::ab_glyph::ScaleFont as _;

        let text = text.as_ref();
        let font_asset = self.assets.get_font(font);
        let scale = glyph_brush_layout::ab_glyph::PxScale::from(font_asset.size() as f32);

        let scaled = font_asset.inner().as_scaled(scale);
        let line_height = scaled.ascent() - scaled.descent() + scaled.line_gap();

        if text.is_empty() {
            return math::Size::new(0.0, line_height);
        }

        let geometry = glyph_brush_layout::SectionGeometry {
            screen_position: (0.0, 0.0),
            bounds: (f32::INFINITY, f32::INFINITY),
        };

        let sections = &[glyph_brush_layout::SectionText {
            text,
            scale,
            font_id: glyph_brush_layout::FontId(0),
        }];

        let fonts = [font_asset.inner()];
        let glyphs = glyph_brush_layout::Layout::default_wrap()
            .calculate_glyphs(&fonts, &geometry, sections);

        let width = glyphs
            .iter()
            .map(|sg| sg.glyph.position.x + scaled.h_advance(sg.glyph.id))
            .fold(0.0f32, f32::max);

        let height = glyphs
            .iter()
            .map(|sg| sg.glyph.position.y)
            .fold(0.0f32, f32::max)
            - scaled.descent();

        math::Size::new(width, height)
    }

    pub fn imgui<F>(&mut self, mut f: F)
    where
        F: FnMut(&imgui::Ui),
    {
        f(&self.imgui)
    }
}

pub struct SceneHandle<'a> {
    renderer: &'a mut Renderer,
    active_layer: Layer,
    packet: &'a mut FramePacket,
    cameras: &'a mut Cameras,
}

impl<'a> SceneHandle<'a> {
    pub fn clear_color(&self) -> Color {
        self.packet.clear_color.into()
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        self.packet.clear_color = color.into();
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.active_layer = layer;
    }

    pub fn add_geometry(&mut self, geometry: Geometry) -> Handle<Geometry> {
        self.renderer.geometries.insert(geometry)
    }

    pub fn get_geometry(&self, handle: Handle<Geometry>) -> &Geometry {
        self.renderer
            .geometries
            .get(handle)
            .expect("Failed to get geometry")
    }

    pub fn get_geometry_mut(&mut self, handle: Handle<Geometry>) -> &mut Geometry {
        self.renderer
            .geometries
            .get_mut(handle)
            .expect("Failed to get geometry")
    }

    pub fn add_material(&mut self, desc: MaterialDesc) -> Handle<Material> {
        self.renderer
            .materials
            .insert(Material::new(desc, &self.renderer.assets.read()))
    }

    pub fn get_material(&self, handle: Handle<Material>) -> &Material {
        self.renderer
            .materials
            .get(handle)
            .expect("Failed to get material")
    }

    pub fn get_material_mut(&mut self, handle: Handle<Material>) -> &mut Material {
        self.renderer
            .materials
            .get_mut(handle)
            .expect("Failed to get material")
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.renderer.data[self.active_layer]
            .retained
            .meshes
            .insert(mesh)
    }

    pub fn get_mesh(&mut self, handle: Handle<Mesh>) -> &Mesh {
        self.renderer.data[self.active_layer]
            .retained
            .meshes
            .get(handle)
            .expect("Failed to get mesh")
    }

    pub fn get_mesh_mut(&mut self, handle: Handle<Mesh>) -> &mut Mesh {
        self.renderer.data[self.active_layer]
            .retained
            .meshes
            .get_mut(handle)
            .expect("Failed to get mesh")
    }

    pub fn camera(&self) -> &Camera {
        &self.cameras[self.active_layer]
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.cameras[self.active_layer]
    }

    pub fn camera_projection(&self) -> &Projection {
        &self.cameras[self.active_layer].projection
    }

    pub fn camera_projection_mut(&mut self) -> &mut Projection {
        &mut self.cameras[self.active_layer].projection
    }

    pub fn set_camera_projection(&mut self, proj: Projection) {
        *self.camera_projection_mut() = proj;
    }
}
