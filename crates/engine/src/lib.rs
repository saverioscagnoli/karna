#![feature(mpmc_channel)]
#![warn(clippy::use_self)]

mod assets;
mod builder;
mod clock;
mod config;
mod cursor;
mod err;
mod events;
mod gpu;
mod input;
mod path;
mod render;
mod scene;
mod sdl;
mod window;

use std::mem;
use std::path::PathBuf;
use std::time::Instant;

use logging::debug;
use logging::error;
use logging::fatal;
use logging::info;
use logging::trace;
use logging::warn;
use math as m;
use sdl3::SDL_INIT_VIDEO;
use sdl3::SDL_Init;
use utils::FastHashMap;
use utils::SleepTimer;

use crate::assets::AssetServer;
use crate::assets::AssetWorkers;
use crate::clock::Clock;
use crate::config::config;
use crate::cursor::CursorRegistry;
use crate::err::sdl_last_error;
use crate::events::KeyEvent;
use crate::events::MouseEvent;
use crate::events::SDLEvent;
use crate::events::SDLWindowEvent;
use crate::events::UserEvent;
use crate::events::WindowId;
use crate::events::queue::EventQueue;
use crate::events::user::UserWindowEvent;
use crate::gpu::Device;
use crate::input::InputScope;
use crate::render::Renderer;
use crate::sdl::SDLGuard;
use crate::window::ForContext;
use crate::window::ForContextMut;
use crate::window::FramePacer;
use crate::window::PaceMode;
use crate::window::SceneSlot;
use crate::window::UpdatePhase;
use crate::window::UserContext;
use crate::window::Window;
use crate::window::WindowEntry;
use crate::window::WindowHandle;
use crate::window::WindowState;

pub use crate::assets::Image;
pub use crate::assets::ImageView;
pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::cursor::Cursor;
pub use crate::cursor::SystemCursor;
pub use crate::events::MouseButton;
pub use crate::input::Input;
pub use crate::input::Key;
pub use crate::render::Camera;
pub use crate::render::Color;
pub use crate::render::Draw;
pub use crate::render::Layer;
pub use crate::render::Projection;
pub use crate::scene::Scene;
pub use crate::scene::SceneId;
pub use crate::window::DrawContext;
pub use crate::window::LoadContext;
pub use crate::window::Time;
pub use crate::window::UpdateContext;

pub struct App {
    requested_windows: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowEntry>,
    should_quit: bool,
    clock: Clock,
    sleeper: SleepTimer,
    input: Input,
    asset_workers: AssetWorkers,
    asset_server: AssetServer,
    cursors: CursorRegistry,

    event_queue: EventQueue<UserEvent>,

    gpu: Device,
    _sdl: SDLGuard,
}

impl App {
    fn new(root: PathBuf) -> Self {
        if sdl::init_check() {
            fatal!("App::new was called more than one time.");
        }

        if !unsafe { SDL_Init(SDL_INIT_VIDEO) } {
            fatal!("Error initalizing SDL: {}", sdl_last_error());
        }

        debug!("SDL v{} initialized.", sdl3::compiled_version_string());

        let config = config();
        let gpu = Device::init(config.gpu.debug);

        let (asset_workers, asset_server) = assets::spawn(root, config.asset.worker_threads);

        info!("Application initalized.");

        Self {
            requested_windows: Vec::new(),
            windows: FastHashMap::default(),
            should_quit: false,
            clock: Clock::default(),
            sleeper: SleepTimer::new(),
            input: Input::default(),
            asset_workers,
            asset_server,
            cursors: CursorRegistry::default(),
            event_queue: EventQueue::default(),
            gpu,
            _sdl: SDLGuard::new(),
        }
    }

    fn spawn_window(&mut self, builder: WindowBuilder) {
        let config = config();

        #[rustfmt::skip]
        let WindowBuilder { title, size, resizable, scene_builders, scenes_active } = builder;
        let window = Window::new(&title, size, resizable, self.gpu.clone());
        let state = WindowState {
            ctx: UserContext {
                window: WindowHandle {
                    id: window.id(),
                    title: window.title().into(),
                    size: window.size(),
                    resizable: window.is_resizable(),
                    mouse_position: m::Vector2::zero(),
                    mouse_delta: m::Vector::zero(),
                    clear_color: config.window.clear_color,
                    dispatcher: self.event_queue.dispatcher(),
                },
                time: Time::new(window.id(), self.event_queue.dispatcher()),
            },
            pacer: FramePacer::new(PaceMode::Fixed),
            #[rustfmt::skip]
            scenes: scene_builders
                .into_iter()
                .map(|(k, b)| (k, SceneSlot{ builder: b, scene: None }))
                .collect(),
            scenes_active,
            renderer: Renderer::new(self.gpu.clone(), window.size()),
        };

        self.windows
            .insert(window.id(), WindowEntry { window, state });
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            error!("Received close request for dropped {}", id);
            return;
        };

        let WindowEntry { window, .. } = entry;

        if self.input.focused == Some(window.id()) {
            self.input.focused = None;
            self.input.keys.clear_all();
            self.input.mouse.clear_all();
        }

        if self.windows.is_empty() {
            self.should_quit = true
        }

        info!("Closing '{}' (\"{}\")", window.id(), window.title());
    }

    fn quit(&mut self) {
        debug!("Quit signal received");

        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.close_window(id);
        }
    }

    pub fn run(mut self) {
        for builder in mem::take(&mut self.requested_windows) {
            self.spawn_window(builder);
        }

        for entry in self.windows.values_mut() {
            entry.state.sync_time(&self.clock);
            entry.state.load_active_scenes(&mut ForContextMut {
                input: &self.input,
                assets: &mut self.asset_server,
            });
        }

        while !self.should_quit {
            for event in events::poll() {
                trace!("Received SDL event: {:?}", event);

                match event {
                    SDLEvent::Quit => self.quit(),
                    SDLEvent::Key { window, kevent } => {
                        #[rustfmt::skip]
                        let KeyEvent { pressed, repeat, scancode, .. } = kevent;
                        let Some(key) = Key::from_scancode(scancode.raw()) else {
                            continue;
                        };

                        // First scope -> key down
                        // Second scope -> key up
                        if pressed {
                            if !repeat && self.input.focused == Some(window) {
                                self.input.keys.press(key);
                            }
                        } else {
                            self.input.keys.release(key);
                        }
                    }
                    SDLEvent::Mouse { window, mevent } => match mevent {
                        MouseEvent::Motion { x, y, dx, dy } => {
                            let Some(entry) = self.windows.get_mut(&window) else {
                                continue;
                            };

                            entry.state.ctx.window.mouse_position.set([x, y]);
                            entry.state.ctx.window.mouse_delta += m::Vector2::new(dx, dy);
                        }
                        #[rustfmt::skip]
                        MouseEvent::Button { button, pressed, .. } => {
                            // First scope -> mouse down
                            // Second scope -> mouse up
                            if pressed {
                                if self.input.focused == Some(window) {
                                    self.input.mouse.press(button);
                                }
                            } else {
                                self.input.mouse.release(button);
                            }
                        }
                        MouseEvent::Wheel { x, y, .. } => {
                            self.input.m_wheel += m::Vector2::new(x, y);
                        }
                    },

                    SDLEvent::Window { window, wevent } => match wevent {
                        SDLWindowEvent::CloseRequested => self.close_window(window),
                        SDLWindowEvent::FocusGained => {
                            self.input.focused = Some(window);
                            debug!("{:?} gained focus.", window)
                        }
                        SDLWindowEvent::FocusLost => {
                            if self.input.focused == Some(window) {
                                self.input.focused = None;
                                self.input.keys.clear_all();
                                self.input.mouse.clear_all();
                                debug!("{:?} lost focus.", window);
                            }
                        }

                        _ => {}
                    },
                    _ => {}
                }
            }

            for event in self.event_queue.drain() {
                trace!("Received user event: {:?}", event);

                match event {
                    UserEvent::ChangeTargetTps(t) => self.clock.set_target_tps(t),
                    UserEvent::ChangeCursor(kind) => self.cursors.pending = Some(kind),
                    UserEvent::Window { id, wevent } => {
                        let Some(entry) = self.windows.get_mut(&id) else {
                            warn!("Received user event for dropped {:?}: {:?}", id, wevent);
                            continue;
                        };

                        let WindowEntry { window, state } = entry;
                        let WindowState { pacer, .. } = state;
                        let FramePacer { counter, .. } = pacer;

                        match wevent {
                            UserWindowEvent::ChangeTitle(t) => window.set_title(t),
                            UserWindowEvent::ChangeSize(s) => window.set_size(s),
                            UserWindowEvent::ChangeResizable(r) => window.set_resizable(r),
                            UserWindowEvent::ChangeTargetFps(t) => pacer.set_target_fps(t),
                            UserWindowEvent::ChangeFpsCalcStrategy(s) => counter.set_strategy(s),

                            event => {
                                let mut fctx = ForContextMut {
                                    input: &self.input,
                                    assets: &mut self.asset_server,
                                };

                                match event {
                                    UserWindowEvent::LoadScene(s) => {
                                        entry.state.load_scene(s, &mut fctx)
                                    }
                                    UserWindowEvent::UnloadScene(s) => {
                                        entry.state.unload_scene(s, &mut fctx)
                                    }
                                    UserWindowEvent::ActivateScene(s) => {
                                        entry.state.activate_scene(s, &mut fctx)
                                    }
                                    UserWindowEvent::DeactivateScene(s) => {
                                        entry.state.deactivate_scene(s)
                                    }

                                    _ => unreachable!(),
                                }
                            }
                        }
                    }
                }
            }

            self.asset_server.poll(&self.gpu);
            self.cursors.poll(&self.asset_server);

            if self.should_quit {
                break;
            }

            let now = Instant::now();

            self.clock.advance(now);

            while self.clock.should_tick() {
                self.input.change_scope(InputScope::Tick);

                for entry in self.windows.values_mut() {
                    entry.state.sync_time(&self.clock);
                    entry.state.update_active_scenes(
                        UpdatePhase::Fixed,
                        ForContextMut {
                            input: &self.input,
                            assets: &mut self.asset_server,
                        },
                    );
                }

                self.input.roll_tick();
                self.clock.consume();
            }

            self.input.change_scope(InputScope::Frame);

            let mut rendered = false;
            let now = Instant::now();

            for entry in self.windows.values_mut() {
                if !entry.state.pacer.due(now) {
                    continue;
                }

                rendered = true;

                entry.state.pacer.record(now);
                entry.state.sync_time(&self.clock);

                entry.state.update_active_scenes(
                    UpdatePhase::Unrestrained,
                    ForContextMut {
                        input: &self.input,
                        assets: &mut self.asset_server,
                    },
                );

                entry.state.draw_active_scenes(ForContext {
                    input: &self.input,
                    assets: &self.asset_server,
                });

                entry.state.render(&entry.window, &self.asset_server);
                entry.state.sync_window(&entry.window);
            }

            if rendered {
                self.input.roll_frame();
            }

            let after_draw = Instant::now();
            let tick = self.clock.next_tick();

            let deadline = self
                .windows
                .values()
                .map(|entry| {
                    entry
                        .state
                        .pacer
                        .deadline()
                        .unwrap_or(after_draw + entry.state.pacer.idle_backoff())
                })
                .fold(tick, Instant::min);

            self.sleeper.sleep_until(deadline);
        }

        info!("Lifecycle loop over. Exiting.");

        self.cursors.shutdown();
        self.asset_workers.shutdown(self.asset_server);
    }
}
