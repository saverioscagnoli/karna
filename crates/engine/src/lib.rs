mod builder;
mod clock;
mod config;
mod err;
mod events;
mod path;
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

use crate::clock::Clock;
use crate::config::config;
use crate::err::sdl_last_error;
use crate::events::SDLEvent;
use crate::events::SDLWindowEvent;
use crate::events::UserEvent;
use crate::events::WindowId;
use crate::events::queue::EventQueue;
use crate::events::user::UserWindowEvent;
use crate::sdl::SDLGuard;
use crate::window::FramePacer;
use crate::window::PaceMode;
use crate::window::SceneSlot;
use crate::window::Time;
use crate::window::UpdatePhase;
use crate::window::UserContext;
use crate::window::Window;
use crate::window::WindowEntry;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::scene::Scene;
pub use crate::scene::SceneId;
pub use crate::window::DrawContext;
pub use crate::window::LoadContext;
pub use crate::window::UpdateContext;
use crate::window::WindowHandle;
use crate::window::WindowState;

pub struct App {
    requested_windows: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowEntry>,
    should_quit: bool,
    clock: Clock,
    sleeper: SleepTimer,
    event_queue: EventQueue<UserEvent>,

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

        info!("Application initalized.");

        Self {
            requested_windows: Vec::new(),
            windows: FastHashMap::default(),
            should_quit: false,
            clock: Clock::default(),
            sleeper: SleepTimer::new(),
            event_queue: EventQueue::default(),
            _sdl: SDLGuard::new(),
        }
    }

    fn spawn_window(&mut self, builder: WindowBuilder) {
        #[rustfmt::skip]
        let WindowBuilder { title, size, resizable, scene_builders, scenes_active } = builder;
        let window = Window::new(&title, size, resizable);
        let state = WindowState {
            ctx: UserContext {
                window: WindowHandle {
                    id: window.id(),
                    title: window.title().into(),
                    size: window.size(),
                    resizable: window.is_resizable(),
                    mouse_position: m::Vector2::zero(),
                    mouse_delta: m::Vector::zero(),
                    dispatcher: self.event_queue.dispatcher(),
                },
                time: Time::new(window.id(), self.event_queue.dispatcher()),
            },
            pacer: FramePacer::new(PaceMode::Fixed),
            scenes: scene_builders
                .into_iter()
                .map(|(k, b)| (k, SceneSlot::Unloaded(b)))
                .collect(),
            scenes_active,
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
            entry.state.load_active_scenes();
        }

        while !self.should_quit {
            for event in events::poll() {
                trace!("Received SDL event: {:?}", event);

                match event {
                    SDLEvent::Quit => self.quit(),
                    SDLEvent::Window { window, wevent } => match wevent {
                        SDLWindowEvent::CloseRequested => self.close_window(window),
                        _ => {}
                    },
                    _ => {}
                }
            }

            for event in self.event_queue.drain() {
                match event {
                    UserEvent::ChangeTargetTps(t) => self.clock.set_target_tps(t),
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
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }

            let now = Instant::now();

            self.clock.advance(now);

            while self.clock.should_tick() {
                for entry in self.windows.values_mut() {
                    entry.state.sync_time(&self.clock);
                    entry.state.update_active_scenes(UpdatePhase::Fixed);
                }

                self.clock.consume();
            }

            for entry in self.windows.values_mut() {
                if !entry.state.pacer.due(now) {
                    continue;
                }

                entry.state.pacer.record(now);
                entry.state.sync_time(&self.clock);
                entry.state.update_active_scenes(UpdatePhase::Unrestrained);
                entry.state.draw_active_scenes();
                entry.state.sync_window(&entry.window);
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
    }
}
