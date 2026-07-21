use crate::assets::AssetServer;
use crate::assets::AssetsView;
use crate::render::Color;
use crate::render::Draw;
use crate::render::FramePacket;
use crate::render::Layer;
use crate::render::Renderer;
use crate::scene::SceneManager;
use crate::window::Window;
use crate::window::input::Input;
use crate::window::scene::Cameras;
use crate::window::scene::SceneView;
use crate::window::time::Time;

pub struct WindowContext {
    pub window: Window,
    pub input: Input,
    pub time: Time,
    pub scenes: SceneManager,
    pub assets: AssetServer,
    pub cameras: Cameras,
    pub renderer: Renderer,
}

pub struct Context<'a> {
    pub window: &'a mut Window,
    pub input: &'a mut Input,
    pub time: &'a mut Time,
    pub assets: AssetsView<'a>,
    pub scenes: &'a mut SceneManager,
}

impl WindowContext {
    pub fn split_scene<'a>(
        &'a mut self,
        packet: &'a mut FramePacket,
    ) -> (Context<'a>, SceneView<'a>) {
        (
            Context {
                window: &mut self.window,
                input: &mut self.input,
                time: &mut self.time,
                assets: AssetsView::new(&self.assets),
                scenes: &mut self.scenes,
            },
            SceneView {
                cameras: &mut self.cameras,
                active_layer: Layer::default(),
                packet,
            },
        )
    }

    pub fn split_draw<'a>(&'a mut self, packet: &'a mut FramePacket) -> (Context<'a>, Draw<'a>) {
        (
            Context {
                window: &mut self.window,
                input: &mut self.input,
                time: &mut self.time,
                assets: AssetsView::new(&self.assets),
                scenes: &mut self.scenes,
            },
            Draw {
                r: &mut self.renderer,
                active_layer: Layer::default(),
                color: Color::White.into(),
                assets: AssetsView::new(&self.assets),
                packet,
            },
        )
    }
}
