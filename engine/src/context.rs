use assets::AssetServer;
use assets::AssetsRead;
use assets::Font;
use assets::Image;
use glyph_brush_layout::GlyphPositioner;
use gpu::Vertex;
use logging::warn;
use renderer::Camera;
use renderer::Color;
use renderer::FramePacket;
use renderer::Layer;
use renderer::Projection;
use utils::Handle;

use crate::SceneManager;
use crate::Time;
use crate::Window;
use crate::input::Input;
use crate::mixer::Mixer;

pub struct WindowContext {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub scenes: SceneManager,
    pub mixer: Mixer,
    pub assets: AssetServer,

    pub world_camera: Camera,
    pub ui_camera: Camera,
    pub debug_camera: Camera,
}

impl WindowContext {
    pub fn new(window: Window, assets: AssetServer) -> Self {
        let view = window.size();

        Self {
            window,
            time: Time::new(),
            input: Input::new(),
            scenes: SceneManager::new(),
            mixer: Mixer::new(assets.reader()),
            assets,
            world_camera: Camera::new(Projection::standard_2d(view)),
            ui_camera: Camera::new(Projection::standard_2d(view)),
            debug_camera: Camera::new(Projection::standard_2d(view)),
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
}

pub struct ContextRef<'a> {
    pub window: &'a Window,
    pub time: &'a Time,
    pub input: &'a Input,
    pub scenes: &'a SceneManager,
    pub mixer: &'a Mixer,
    pub assets: AssetsRead<'a>,
}

impl WindowContext {
    pub fn as_ref<'a>(&'a self) -> ContextRef<'a> {
        ContextRef {
            window: &self.window,
            time: &self.time,
            input: &self.input,
            scenes: &self.scenes,
            mixer: &self.mixer,
            assets: self.assets.read(),
        }
    }

    pub fn split_mut<'a>(
        &'a mut self,
        packet: &'a mut FramePacket,
    ) -> (ContextMut<'a>, SceneHandle<'a>) {
        let Self {
            window,
            time,
            input,
            scenes,
            mixer,
            assets,
            world_camera,
            ui_camera,
            debug_camera,
        } = self;

        (
            ContextMut {
                window,
                time,
                input,
                scenes,
                mixer,
                assets,
            },
            SceneHandle {
                active_layer: Layer::World,
                packet,
                world_camera,
                ui_camera,
                debug_camera,
            },
        )
    }

    pub fn split<'a>(&'a self, packet: &'a mut FramePacket) -> (ContextRef<'a>, Draw<'a>) {
        (self.as_ref(), Draw::new(packet, self.assets.read()))
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
    white_uv: math::Vector2<f32>,
}

impl<'a> Draw<'a> {
    pub(crate) fn new(packet: &'a mut FramePacket, assets: AssetsRead<'a>) -> Self {
        let white_uv = assets.get_image(assets.white_handle()).uv.xy();

        Self {
            packet,
            state_stack: Vec::new(),
            current_state: DrawState::default(),
            active_layer: Layer::World,
            assets,
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

    fn vertex(&self, x: f32, y: f32, uv: math::Vector2<f32>) -> Vertex {
        let s = &self.current_state;
        let t = s.transform.mul_vec(&math::Vector4::new(x, y, 0.0, 1.0));
        let pos = math::Vector3::new(t.x, t.y, s.depth);

        Vertex::new(pos, s.draw_color, uv)
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
}

pub struct SceneHandle<'a> {
    active_layer: Layer,
    packet: &'a mut FramePacket,
    pub world_camera: &'a mut Camera,
    pub ui_camera: &'a mut Camera,
    pub debug_camera: &'a mut Camera,
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

    pub fn camera(&self) -> &Camera {
        match self.active_layer {
            Layer::World => &self.world_camera,
            Layer::Ui => &self.ui_camera,
            Layer::Debug => &self.debug_camera,
        }
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        match self.active_layer {
            Layer::World => &mut self.world_camera,
            Layer::Ui => &mut self.ui_camera,
            Layer::Debug => &mut self.debug_camera,
        }
    }
}
