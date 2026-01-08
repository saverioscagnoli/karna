pub mod input;
pub mod states;
pub mod sysinfo;

mod monitors;
mod scene_changer;
mod time;
mod tween;
mod window;

use crate::{
    Arcs,
    state::{
        input::Input,
        scene_changer::SceneChanger,
        states::{GlobalStates, ScopedStates},
        sysinfo::SystemInfo,
    },
};
use assets::AssetServer;
use globals::profiling::{self, Statistics};
use renderer::{Draw, Renderer, Scene};
use std::sync::Arc;
use winit::{
    event::{DeviceEvent, MouseScrollDelta, WindowEvent},
    keyboard::PhysicalKey,
};

// === RE-EXPORTS ===
pub use crate::state::time::Time;
pub use monitors::{Monitor, Monitors};
pub use window::Window;
pub(crate) use window::WinitWindow;

/// Holds the state of the game loop for a single window
/// for all its life, shares its content to [`Context`]
/// and [`RenderContext`]
///
/// So that the state can be mutable when `Scene::load` and `Scene::update`
/// But not on Scene::render, where a [`SceneView`] will be created, so that
/// scene information can be read, but not written.
pub struct EngineState {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub render: Renderer,
    pub scenes: SceneChanger,
    pub monitors: Monitors,
    pub assets: AssetServer,
    pub states: ScopedStates,
    pub globals: Arc<GlobalStates>,
    pub info: Arc<SystemInfo>,
    pub profiling: Statistics,
}

unsafe impl Send for EngineState {}
unsafe impl Sync for EngineState {}

/// Holds all the references from [`EngineState`],
/// And permits the user to mutate the window state, but only during
/// `Scene::load` and `Scene::update`
pub struct Context<'a> {
    pub window: &'a Window,
    pub time: &'a mut Time,
    pub input: &'a mut Input,
    // (This context will be mutable, so the scene can be changed)
    pub scene: Scene<'a>,
    pub scenes: &'a mut SceneChanger,
    pub monitors: &'a Monitors,
    pub assets: &'a mut AssetServer,
    pub states: &'a mut ScopedStates,
    pub globals: &'a GlobalStates,
    pub info: &'a SystemInfo,
    pub profiling: &'a Statistics,
}

/// Basically equal to [`Context`], but will be immutable,
/// where it will be accompanied by a mutable [`Draw`] handle, for immediate rendering.
pub struct RenderContext<'a> {
    pub window: &'a Window,
    pub time: &'a Time,
    pub input: &'a Input,
    pub monitors: &'a Monitors,
    pub assets: &'a AssetServer,
    pub states: &'a ScopedStates,
    pub globals: &'a GlobalStates,
    pub info: &'a SystemInfo,
    pub profiling: &'a Statistics,
}

impl EngineState {
    pub(crate) fn new(window: Window, arcs: Arcs) -> Self {
        let assets = AssetServer::new();
        let render = Renderer::new(window.inner().clone(), &assets);
        let scenes = SceneChanger::new();
        let monitors = Monitors::new(Arc::clone(window.inner()));
        let states = ScopedStates::new();

        Self {
            window,
            time: Time::default(),
            input: Input::default(),
            render,
            scenes,
            monitors,
            assets,
            states,
            globals: arcs.globals,
            info: arcs.info,
            profiling: profiling::get_stats(),
        }
    }

    #[inline]
    pub(crate) fn as_context(&mut self) -> Context<'_> {
        Context {
            window: &mut self.window,
            time: &mut self.time,
            input: &mut self.input,
            scene: Scene::new(&mut self.render),
            scenes: &mut self.scenes,
            monitors: &self.monitors,
            assets: &mut self.assets,
            states: &mut self.states,
            globals: &self.globals,
            info: &self.info,
            profiling: &self.profiling,
        }
    }

    #[inline]
    pub(crate) fn as_render_context(&mut self) -> (RenderContext<'_>, Draw<'_>) {
        let ctx = RenderContext {
            window: &self.window,
            time: &self.time,
            input: &self.input,
            monitors: &self.monitors,
            assets: &self.assets,
            states: &self.states,
            globals: &self.globals,
            info: &self.info,
            profiling: &self.profiling,
        };

        let draw = Draw::new(&mut self.render, &self.assets);

        (ctx, draw)
    }

    #[inline]
    pub(crate) fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.input.mouse_delta.x += delta.0 as f32;
                self.input.mouse_delta.y += delta.1 as f32;
            }

            _ => {}
        }
    }

    #[inline]
    pub(crate) fn handle_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.render.resize(size.into());
            }

            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(code) => {
                    if event.state.is_pressed() {
                        if !event.repeat {
                            self.input.pressed_keys.insert(code);
                        }
                        self.input.held_keys.insert(code);
                    } else {
                        self.input.held_keys.remove(&code);
                        self.input.released_keys.insert(code);
                    }
                }
                PhysicalKey::Unidentified(_) => {}
            },

            WindowEvent::CursorMoved { position, .. } => {
                self.input.mouse_position.x = position.x as f32;
                self.input.mouse_position.y = position.y as f32;
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() {
                    if !self.input.pressed_mouse.contains(&button) {
                        self.input.pressed_mouse.insert(button);
                    }
                    self.input.held_mouse.insert(button);
                } else {
                    self.input.held_mouse.remove(&button);
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                self.input.wheel_delta = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
            }

            _ => {}
        }
    }
}
