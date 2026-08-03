use std::ops::Deref;
use std::ops::DerefMut;

use imgui::Imgui;

use crate::Time;
use crate::assets::Assets;
use crate::event::AppEvent;
use crate::event::EventDispatcher;
use crate::event::SdlEvent;
use crate::event::SdlWindowEvent;
use crate::render::layer::Layer;
use crate::scene::ScenePhase;
use crate::scene::World;
use crate::window::clock::Clock;
use crate::window::context::WindowContext;
use crate::window::pacer::FramePacer;
use crate::window::platform::PlatformWindow;

pub struct Frame<'a> {
    pub assets: &'a mut Assets,
    pub mode: gpu::PresentMode,
    pub events: EventDispatcher<AppEvent>,
}

pub struct WindowState {
    pub context: WindowContext,
    pub world: World,
    pub clock: Clock,
    pub pacer: FramePacer,
}

impl Deref for WindowState {
    type Target = WindowContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for WindowState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl WindowState {
    pub fn new(context: WindowContext, world: World) -> Self {
        Self {
            context,
            world,
            clock: Clock::default(),
            pacer: FramePacer::default(),
        }
    }

    fn time(&self, mode: gpu::PresentMode, events: EventDispatcher<AppEvent>) -> Time {
        Time::snapshot(
            self.context.window.id(),
            &self.clock,
            &self.pacer,
            mode,
            events,
        )
    }

    pub fn load(&mut self, frame: Frame<'_>) {
        let mut time = self.time(frame.mode, frame.events);
        self.world
            .load_active(&mut self.context, &mut time, frame.assets);
    }

    pub fn fixed_update(&mut self, frame: Frame<'_>) {
        let mut time = self.time(frame.mode, frame.events);
        self.world
            .fixed_update(&mut self.context, &mut time, frame.assets);
    }

    pub fn update(&mut self, frame: Frame<'_>) {
        let mut time = self.time(frame.mode, frame.events);
        self.world
            .update(&mut self.context, &mut time, frame.assets);
    }

    fn extract(&mut self, assets: &Assets) {
        let viewport = self.context.window.pixel_size();
        let ctx = &mut self.context;

        ctx.render.refresh_materials(assets);
        ctx.render.update_cameras(viewport);
        ctx.packet.clear();

        for layer in Layer::ALL {
            let camera = ctx.render.camera_packet(layer, viewport);
            let depth = ctx.render.view_needs_depth(layer);
            let enabled = ctx.render.view_enabled(layer);

            let view = &mut ctx.packet[layer];

            view.camera = camera;
            view.depth = depth;
            view.enabled = enabled;
        }

        // Meshes are a world-space concept, so they all land on the world view
        let meshes = &mut ctx.packet[Layer::World].meshes;

        ctx.render.extract_meshes(meshes);
    }

    pub fn draw(&mut self, frame: Frame<'_>, imgui: &mut Imgui) {
        let mut time = self.time(frame.mode, frame.events);

        self.extract(frame.assets);
        self.world
            .draw(&mut self.context, &mut time, frame.assets, imgui);
    }

    pub fn handle_window_event(&mut self, event: SdlEvent, platform: &mut PlatformWindow) {
        match event {
            SdlEvent::Window { win_event, .. } => match win_event {
                SdlWindowEvent::Resized(w, h) => {
                    platform.on_resized(w as u32, h as u32);
                }

                SdlWindowEvent::PixelSizeChanged(w, h) => {
                    platform.on_pixel_resized(w as u32, h as u32);
                }

                _ => {}
            },

            SdlEvent::KeyDown {
                scancode: Some(c),
                repeat,
                ..
            } => {
                self.input.k_down.insert(c);

                if !repeat {
                    self.input.k_pressed.insert(c);
                }
            }

            SdlEvent::KeyUp {
                scancode: Some(c), ..
            } => {
                self.input.k_down.remove(&c);
                self.input.k_released.insert(c);
            }

            SdlEvent::MouseButtonDown {
                mouse_btn, clicks, ..
            } => {
                self.input.m_down.insert(mouse_btn);

                if clicks == 1 {
                    self.input.m_pressed.insert(mouse_btn);
                }
            }

            SdlEvent::MouseButtonUp { mouse_btn, .. } => {
                self.input.m_down.remove(&mouse_btn);
                self.input.m_released.insert(mouse_btn);
            }

            SdlEvent::MouseMotion {
                x, y, xrel, yrel, ..
            } => {
                self.input.m_pos.set([x, y]);
                self.input.m_delta.set([xrel, yrel]);
            }

            SdlEvent::MouseWheel { x, y, .. } => {
                self.input.m_wheel.set([x, y]);
            }

            _ => {}
        }
    }

    pub fn flush(&mut self) {
        self.input.flush();
    }
}
