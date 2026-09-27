#![no_std]

pub mod assets;
pub mod builder;
pub mod commands;
pub mod context;
pub mod input;
pub mod render;
pub mod scene;
pub mod storage;
pub mod text;
pub mod time;
pub mod window;
pub mod window_state;

use core::mem;

use nostd::alloc::vec::Vec;
use nostd::collections::HashMap;
use nostd::log;
use nostd::mem::SdlAllocator;
use nostd::path::PathBuf;
use nostd::time::Instant;
use nostd::time::sleep_precise_until;
use sdl3::SdlGuard;
use sdl3::events::Key;
use sdl3::events::KeyEvent;
use sdl3::events::MouseEvent;
use sdl3::events::SdlEvent;
use sdl3::events::SdlWindowEvent;
use sdl3::events::TextEvent;
use sdl3::gpu::Device;
use sdl3::shadercross::ShaderCross;
use sdl3::window::WindowId;
use sdl3::window::WindowState as SdlWindowState;
use traccia::debug;
use traccia::error;
use traccia::info;
use traccia::trace;
use traccia::warn;

use crate::assets::AssetServer;
use crate::assets::AssetThreadPool;
use crate::builder::WindowBuilder;
use crate::commands::AppOutboxes;
use crate::commands::SceneCommand;
use crate::commands::TimeCommand;
use crate::commands::WindowCommand;
use crate::input::Input;
use crate::input::InputScope;
use crate::scene::SceneId;
use crate::storage::SharedStore;
use crate::time::Clock;
use crate::time::FramePacer;
use crate::time::PaceMode;
use crate::window::SdlWindow;
use crate::window_state::SceneSlot;
use crate::window_state::UpdatePhase;
use crate::window_state::WindowState;

#[global_allocator]
static ALLOC: SdlAllocator = SdlAllocator;

struct WindowEntry {
    state: WindowState,
    pacer: FramePacer,
    sdl_window: SdlWindow,
}

pub struct App {
    requested_windows: Vec<WindowBuilder>,
    windows: HashMap<WindowId, WindowEntry>,
    clock: Clock,
    input: Input,
    assets: AssetServer,
    assets_pool: AssetThreadPool,
    store: SharedStore,
    should_quit: bool,
    outboxes: AppOutboxes,

    shadercross: ShaderCross,
    device: Device,
    _sdl: SdlGuard,
}

impl App {
    pub fn new(root: PathBuf, workers: usize) -> Self {
        let Ok(_sdl) = SdlGuard::init() else {
            panic!("sdl failed to init");
        };

        log::capture();
        info!("SDL v{} initialized.", sdl3::linked_version());

        let Ok(device) = sdl3::gpu::Device::init() else {
            panic!("failed to init gpu device");
        };

        let Ok(shadercross) = ShaderCross::init() else {
            panic!("failed to init shadercross");
        };

        let (pool, assets) = assets::spawn(root, workers, &device);

        let outboxes = AppOutboxes::new();

        Self {
            requested_windows: Vec::new(),
            windows: HashMap::default(),
            should_quit: false,
            clock: Clock::default(),
            input: Input::default(),
            assets,
            store: SharedStore::default(),
            assets_pool: pool,
            outboxes,
            shadercross,
            device,
            _sdl,
        }
    }

    fn spawn_window(&mut self, mut b: WindowBuilder) {
        let mut sdl_window = self
            .device
            .create_window(b.title.clone(), b.size, b.flags)
            .expect("Failed to create window");

        if b.opacity != 1.0 {
            sdl_window.set_opacity(b.opacity);
        }

        if let Some(mode) = b.fullscreen {
            sdl_window.set_fullscreen(mode);
        }

        let scenes = mem::take(&mut b.scene_builders)
            .into_iter()
            .map(|(k, v)| (k, SceneSlot::new(k, v, sdl_window.pixel_size())))
            .collect::<HashMap<SceneId, SceneSlot>>();

        let state = WindowState::init(
            &self.device,
            &self.shadercross,
            &mut sdl_window,
            scenes,
            b.active_scenes,
        );

        self.windows.insert(
            sdl_window.id(),
            WindowEntry {
                state,
                pacer: FramePacer::new(PaceMode::Fixed),
                sdl_window,
            },
        );
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            return;
        };

        if self.input.focused == Some(entry.sdl_window.id()) {
            self.input.focused = None;
            self.input.keys.clear_all();
            self.input.mouse.clear_all();
        }

        if self.windows.is_empty() {
            self.should_quit = true;
        }

        info!("Closing window: {} ('{}')", id, entry.sdl_window.title());
    }

    fn quit(&mut self) {
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.close_window(id);
        }
    }

    fn drain_sdl_events(&mut self) {
        for event in sdl3::events::poll() {
            trace!("SDL event: {:?}", event);

            match event {
                SdlEvent::Quit => self.quit(),
                SdlEvent::Key { window, kevent } => {
                    #[rustfmt::skip]
                    let KeyEvent { pressed, repeat,  scancode, .. } = kevent;
                    let Some(key) = Key::from_scancode(scancode.raw()) else {
                        debug!("Cannot convert unknown scancode to key: {:?}", scancode);
                        continue;
                    };

                    if pressed {
                        if !repeat && self.input.focused == Some(window) {
                            self.input.keys.press(key);
                        }
                    } else {
                        self.input.keys.release(key);
                    }
                }
                SdlEvent::Text { window, tevent } => {
                    if self.input.focused != Some(window) {
                        continue;
                    }

                    match tevent {
                        TextEvent::Input { text } => {
                            self.input.text.push_str(&text);
                            self.input.preedit.clear();
                            self.input.preedit_cursor = -1;
                        }
                        TextEvent::Editing { text, cursor, .. } => {
                            self.input.preedit = text;
                            self.input.preedit_cursor = cursor;
                        }
                        _ => {}
                    }
                }
                SdlEvent::Mouse { window, mevent } => match mevent {
                    MouseEvent::Motion { x, y, dx, dy } => {
                        let Some(entry) = self.windows.get_mut(&window) else {
                            warn!("Received SDL mouse event for dropped window: {}", window);
                            continue;
                        };

                        let pos = math::vec2!(x, y);
                        let d = math::vec2!(dx, dy);
                        entry.state.ctx.window_data.update_input(pos, d);
                    }
                    #[rustfmt::skip]
                    MouseEvent::Button { button, pressed, .. } => {
                        if pressed {
                            if self.input.focused == Some(window) {
                                self.input.mouse.press(button);
                            }
                        } else {
                            self.input.mouse.release(button);
                        }
                    },
                    MouseEvent::Wheel { x, y, .. } => {
                        self.input.m_wheel += math::vec2!(x, y);
                    }
                    _ => {}
                },
                SdlEvent::Window { window, wevent } => {
                    let Some(entry) = self.windows.get_mut(&window) else {
                        warn!("Received SDL event for dropped window: {}", window);
                        continue;
                    };

                    match wevent {
                        SdlWindowEvent::CloseRequested => self.close_window(window),
                        SdlWindowEvent::FocusGained => {
                            self.input.focused = Some(window);
                            debug!(
                                "window {} ('{}') gained focus.",
                                window,
                                entry.sdl_window.title()
                            );
                        }
                        SdlWindowEvent::FocusLost => {
                            if self.input.focused == Some(window) {
                                self.input.focused = None;
                                self.input.keys.clear_all();
                                self.input.mouse.clear_all();
                                debug!(
                                    "window {} ('{}') lost focus.",
                                    window,
                                    entry.sdl_window.title()
                                );
                            }
                        }
                        _ => trace!("Unhandled SDL window event: {:?}", wevent),
                    }
                }
                _ => trace!("Unhandled SDL event: {:?}", event),
            }
        }
    }

    fn drain_app_events(&mut self) {
        #[rustfmt::skip]
        let Self { windows, clock, input, assets, store, outboxes, .. } = self;

        for (id, command) in outboxes.window.drain() {
            use WindowCommand::*;
            let Some(entry) = windows.get_mut(&id) else {
                warn!("Received a command for a closed window: {}", id);
                continue;
            };

            match command {
                SetTitle(t) => entry.sdl_window.set_title(t),
                SetSize(s) => entry.sdl_window.set_size(s),
                SetOpacity(v) => entry.sdl_window.set_opacity(v),
                SetPresentMode(m) => _ = entry.sdl_window.set_present_mode(m),
                SetState(state) => {
                    use SdlWindowState::*;

                    match state {
                        Normal => entry.sdl_window.set_windowed(),
                        Minimized => entry.sdl_window.minimize(),
                        Maximized => entry.sdl_window.maximize(),
                        Fullscreen => unreachable!(),
                    }
                }
                SetFullscreenMode(m) => _ = entry.sdl_window.set_fullscreen(m),
                SetHidden(h) => entry.sdl_window.set_hidden(h),
                SetResizable(r) => entry.sdl_window.set_resizable(r),
                SetDecorated(d) => entry.sdl_window.set_decorated(d),
                SetAlwaysOnTop(o) => entry.sdl_window.set_always_on_top(o),
                SetFocusable(f) => entry.sdl_window.set_focusable(f),
                SetMouseGrabbed(m) => entry.sdl_window.set_mouse_grabbed(m),
                SetKeyboardGrabbed(k) => entry.sdl_window.set_keyboard_grabbed(k),
                SetRelativeMouse(r) => entry.sdl_window.set_relative_mouse(r),
                Restore => entry.sdl_window.restore(),
            }
        }

        for command in outboxes.time.drain() {
            use TimeCommand::*;

            match command {
                SetTargetFPS(wid, t) => {
                    if let Some(entry) = windows.get_mut(&wid) {
                        entry.pacer.set_target_fps(t);
                    }
                }
                SetFPSCalculationStrategy(wid, strat) => {
                    if let Some(entry) = windows.get_mut(&wid) {
                        entry.pacer.counter.set_strategy(strat);
                    }
                }
                SetTargetTPS(t) => clock.set_target_tps(t),
            }
        }

        let mut pending = outboxes.scene.take();

        for (wid, command) in pending.drain(..) {
            use SceneCommand::*;

            let Some(entry) = windows.get_mut(&wid) else {
                warn!("Received a scene command from a closed window: {}", wid);
                continue;
            };

            match command {
                Load(scene) => entry
                    .state
                    .load_scene(scene, outboxes, input, assets, store),

                Activate(scene) => entry
                    .state
                    .activate_scene(scene, outboxes, input, assets, store),

                Deactivate(scene) => entry.state.deactivate_scene(scene),

                Unload(scene) => entry
                    .state
                    .unload_scene(scene, outboxes, input, assets, store),
            }
        }

        outboxes.scene.restore(pending);
    }

    pub fn run(mut self) {
        for builder in mem::take(&mut self.requested_windows) {
            self.spawn_window(builder);
        }

        for entry in self.windows.values_mut() {
            entry.state.sync_time(&self.clock, &entry.pacer);
            entry.state.load_active_scenes(
                &mut self.outboxes,
                &self.input,
                &mut self.assets,
                &mut self.store,
            );
        }

        while !self.should_quit {
            self.drain_sdl_events();
            self.drain_app_events();
            self.assets.poll();

            if self.should_quit {
                break;
            }

            let now = Instant::now();
            self.clock.advance(now);

            while self.clock.should_tick() {
                self.input.change_scope(InputScope::Tick);

                for entry in self.windows.values_mut() {
                    entry.state.sync_time(&self.clock, &entry.pacer);
                    entry.state.update_active_scenes(
                        UpdatePhase::Fixed,
                        &mut self.outboxes,
                        &self.input,
                        &mut self.assets,
                        &mut self.store,
                    );
                }

                self.input.roll_tick();
                self.clock.consume();
            }

            self.input.change_scope(InputScope::Frame);

            let mut rendered = false;
            let now_after_tick = Instant::now();

            for entry in self.windows.values_mut() {
                if !entry.pacer.due(now) {
                    continue;
                }

                rendered = true;

                entry.pacer.record(now_after_tick);
                entry.state.sync_time(&self.clock, &entry.pacer);

                entry.state.update_active_scenes(
                    UpdatePhase::Unrestrained,
                    &mut self.outboxes,
                    &self.input,
                    &mut self.assets,
                    &mut self.store,
                );

                entry
                    .state
                    .draw_active_scenes(&self.input, &mut self.assets, &self.store);

                let atlas = self.assets.atlas();

                if let Err(e) = entry.state.renderer.flush(
                    &entry.sdl_window,
                    atlas.texture(),
                    entry.state.ctx.window_data.clear_color(),
                ) {
                    error!("Failed to render frame: {}", e);
                }

                entry.state.sync_window(&entry.sdl_window);
            }

            if rendered {
                self.input.roll_frame();
            }

            let now_after_render = Instant::now();
            let tick = self.clock.next_tick();

            let deadline = self
                .windows
                .values()
                .map(|e| {
                    e.pacer
                        .deadline()
                        .unwrap_or(now_after_render + e.pacer.idle_backoff())
                })
                .fold(tick, Instant::min);

            sleep_precise_until(deadline);
        }

        info!("App lifecycle ended, exiting.");
        self.assets_pool.shutdown(self.assets);
    }
}
