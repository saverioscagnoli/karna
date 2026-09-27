use nostd::alloc::vec::Vec;
use nostd::collections::HashMap;
use sdl3::gpu::Device;
use sdl3::shadercross::ShaderCross;
use traccia::error;
use traccia::warn;

use crate::assets::AssetServer;
use crate::context::UserContext;
use crate::event::AppOutboxes;
use crate::input::Input;
use crate::render::Renderer;
use crate::scene::BoxedScene;
use crate::scene::SceneBuilder;
use crate::scene::SceneData;
use crate::scene::SceneId;
use crate::storage::SharedStore;
use crate::time::Clock;
use crate::time::FramePacer;
use crate::time::TimeData;
use crate::window::SdlWindow;
use crate::window::WindowData;

pub struct SceneSlot {
    pub builder: SceneBuilder,
    pub scene: Option<BoxedScene>,
    pub data: SceneData,
}

impl SceneSlot {
    pub fn new(builder: SceneBuilder, viewport: math::Size<u32>) -> Self {
        Self {
            builder,
            scene: None,
            data: SceneData::new(viewport),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UpdatePhase {
    Fixed,
    Unrestrained,
}

pub struct WindowState {
    pub time: TimeData,
    pub ctx: UserContext,
    pub renderer: Renderer,
    pub scenes: HashMap<SceneId, SceneSlot>,
    pub active_scenes: Vec<SceneId>,
}

impl WindowState {
    pub fn init(
        device: &Device,
        shadercross: &ShaderCross,
        window: &SdlWindow,
        scenes: HashMap<SceneId, SceneSlot>,
        active_scenes: Vec<SceneId>,
    ) -> Self {
        let ctx = UserContext {
            window_data: WindowData::init(window),
            time_data: TimeData::default(),
        };

        Self {
            time: TimeData::default(),
            ctx,
            renderer: Renderer::new(device.share(), shadercross, window),
            scenes,
            active_scenes,
        }
    }

    #[inline]
    pub fn sync_time(&mut self, clock: &Clock, pacer: &FramePacer) {
        self.ctx.time_data.sync(clock, pacer);
    }

    #[inline]
    pub fn sync_window(&mut self, window: &SdlWindow) {
        self.ctx.window_data.sync(window);

        let viewport = window.pixel_size();

        for slot in self.scenes.values_mut() {
            slot.data.sync(viewport);
        }
    }

    pub fn load_scene(
        &mut self,
        scene_id: SceneId,
        outboxes: &mut AppOutboxes,
        input: &Input,
        assets: &mut AssetServer,
        shared: &mut SharedStore,
    ) {
        let Self { ctx, scenes, .. } = self;

        let Some(slot) = scenes.get_mut(&scene_id) else {
            error!("Trying to load an invalid scene: {}", scene_id);
            return;
        };

        if slot.scene.is_none() {
            slot.data = SceneData::new(ctx.window_data.pixel_size());
            slot.scene = Some((slot.builder)(&mut ctx.for_load(
                outboxes,
                input,
                assets,
                shared,
                &mut slot.data,
            )));
        }
    }

    pub fn unload_scene(
        &mut self,
        scene_id: SceneId,
        outboxes: &mut AppOutboxes,
        input: &Input,
        assets: &mut AssetServer,
        shared: &mut SharedStore,
    ) {
        let Some(slot) = self.scenes.get_mut(&scene_id) else {
            error!("Trying to unload an invalid scene: {}", scene_id);
            return;
        };

        if let Some(ref mut scene) = slot.scene {
            scene.unload(
                &mut self
                    .ctx
                    .for_load(outboxes, input, assets, shared, &mut slot.data),
            );
        }

        slot.scene = None;
    }

    pub fn activate_scene(
        &mut self,
        scene_id: SceneId,
        outboxes: &mut AppOutboxes,
        input: &Input,
        assets: &mut AssetServer,
        shared: &mut SharedStore,
    ) {
        if !self.scenes.contains_key(&scene_id) {
            error!("Trying to activate an invalid scene: {}", scene_id);
            return;
        }

        self.load_scene(scene_id, outboxes, input, assets, shared);

        if !self.active_scenes.contains(&scene_id) {
            self.active_scenes.push(scene_id);
        }
    }

    pub fn deactivate_scene(&mut self, scene_id: SceneId) {
        self.active_scenes.retain(|id| &scene_id != id);
    }

    pub fn load_active_scenes(
        &mut self,
        outboxes: &mut AppOutboxes,
        input: &Input,
        assets: &mut AssetServer,
        shared: &mut SharedStore,
    ) {
        for id in self.active_scenes.clone() {
            self.load_scene(id, outboxes, input, assets, shared);
        }
    }

    pub fn update_active_scenes(
        &mut self,
        phase: UpdatePhase,
        outboxes: &mut AppOutboxes,
        input: &Input,
        assets: &mut AssetServer,
        shared: &mut SharedStore,
    ) {
        #[rustfmt::skip]
        let Self { ctx, scenes, active_scenes, .. } = self;

        for id in active_scenes {
            let Some(slot) = scenes.get_mut(id) else {
                error!("Trying to update an invalid scene: {}", id);
                continue;
            };

            let Some(ref mut scene) = slot.scene else {
                warn!("Trying to update an unloaded scene: {}", id);
                continue;
            };

            let ctx = &mut ctx.for_update(outboxes, input, assets, shared, &mut slot.data);

            match phase {
                UpdatePhase::Fixed => scene.fixed_update(ctx),
                UpdatePhase::Unrestrained => scene.update(ctx),
            }
        }
    }

    pub fn draw_active_scenes(
        &mut self,
        input: &Input,
        assets: &mut AssetServer,
        shared: &SharedStore,
    ) {
        #[rustfmt::skip]
        let Self { ctx, renderer, scenes, active_scenes, .. } = self;

        assets.text_mut().begin_frame();
        renderer.begin_frame();

        for id in active_scenes {
            let Some(slot) = scenes.get_mut(id) else {
                error!("Trying to draw an invalid scene: {}", id);
                continue;
            };

            let Some(ref mut scene) = slot.scene else {
                warn!("Trying to draw an unloaded scene: {}", id);
                continue;
            };

            let mut draw = renderer.draw_handle(assets);

            scene.draw(
                &mut ctx.for_draw(input, assets, shared, &slot.data),
                &mut draw,
            );

            renderer.commit(&slot.data);
        }
    }
}
