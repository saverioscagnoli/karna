mod builder;
mod clock;
mod config;
mod err;
mod events;
mod gpu;
mod input;
mod render;
mod scene;
mod window;

use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Instant;

use logging::debug;
use logging::error;
use logging::info;
use logging::trace;
use sdl3::SDL_Event;
use sdl3::SDL_EventType;
use sdl3::SDL_INIT_VIDEO;
use sdl3::SDL_Init;
use sdl3::SDL_PollEvent;
use sdl3::SDL_Quit;
use utils::FastHashMap;
use utils::SleepTimer;

use crate::clock::Clock;
use crate::config::config;
use crate::err::SDL_LastError;
use crate::events::AppEvent;
use crate::events::EventQueue;
use crate::events::WindowEvent;
use crate::gpu::Gpu;
use crate::input::Input;
use crate::input::edges::InputScope;
use crate::window::WindowHandle;
use crate::window::WindowId;
use crate::window::context::UserContext;
use crate::window::pacer::FramePacer;
use crate::window::pacer::PaceMode;
use crate::window::platform::PlatformWindow;
use crate::window::state::SceneSlot;
use crate::window::state::UpdatePhase;
use crate::window::state::WindowState;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::gpu::buffer::Buffer;
pub use crate::gpu::buffer::BufferError;
pub use crate::gpu::buffer::BufferUsage;
pub use crate::input::keys::Key;
pub use crate::input::mouse::MouseButton;
pub use crate::render::draw::Draw;
pub use crate::render::stage::SceneView;
pub use crate::scene::Scene;
pub use crate::scene::SceneId;
pub use crate::window::context::DrawContext;
pub use crate::window::context::LoadContext;
pub use crate::window::context::UpdateContext;
pub use crate::window::pacer::FpsCountStrategy;
pub use crate::window::time::Time;

static SDL_ACTIVE: AtomicBool = AtomicBool::new(false);

struct WindowEntry {
    platform: PlatformWindow,
    state: WindowState,
}

struct SdlGuard(PhantomData<*const ()>);

impl Drop for SdlGuard {
    fn drop(&mut self) {
        unsafe { SDL_Quit() };
        SDL_ACTIVE.store(false, Ordering::Release);
    }
}

pub struct App {
    requested_at_creation: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowEntry>,
    should_quit: bool,

    app_events: EventQueue<AppEvent>,
    input: Input,
    clock: Clock,
    sleeper: SleepTimer,

    gpu: Rc<Gpu>,

    _sdl: SdlGuard,
}

impl App {
    pub(crate) fn new() -> Result<Self, String> {
        if SDL_ACTIVE.swap(true, Ordering::AcqRel) {
            return Err("SDL is already initialized.".into());
        }

        let success = unsafe { SDL_Init(SDL_INIT_VIDEO) };

        if !success {
            SDL_ACTIVE.store(false, Ordering::Release);
            return Err(SDL_LastError());
        }

        debug!("SDL v{} initialized.", sdl3::compiled_version_string());

        let gpu = Gpu::init();

        info!("Application initialized.");

        Ok(Self {
            requested_at_creation: Vec::new(),
            windows: FastHashMap::default(),
            should_quit: false,
            app_events: EventQueue::new(),
            input: Input::default(),
            clock: Clock::default(),
            sleeper: SleepTimer::new(),
            gpu: Rc::new(gpu),
            _sdl: SdlGuard(PhantomData),
        })
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    fn spawn_window(&mut self, builder: WindowBuilder) {
        #[rustfmt::skip]
        let WindowBuilder { title, size, scenes, active_scenes } = builder;

        let config = config();
        let window = PlatformWindow::new(&title, size, self.gpu.clone());

        self.gpu.set_present_mode(&window, config.present_mode);

        let scenes = scenes
            .into_iter()
            .map(|(id, scene)| (id, SceneSlot::Unloaded(scene)))
            .collect::<FastHashMap<_, _>>();

        let state = WindowState {
            ctx: UserContext {
                window_handle: WindowHandle::new(&window, self.app_events.dispatcher()),
            },
            draw: Draw::new(),
            pacer: FramePacer::new(PaceMode::Fixed),
            time: Time::new(window.id(), self.app_events.dispatcher()),
            scenes,
            active_scenes,
        };

        self.windows.insert(
            window.id(),
            WindowEntry {
                platform: window,
                state,
            },
        );
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            error!("Received close request for a closed window.");
            return;
        };

        if self.input.focused == Some(id) {
            self.input.focused = None;
            self.input.keys.clear_all();
            self.input.mouse.clear_all();
        }

        if self.windows.is_empty() {
            self.should_quit = true;
        }

        info!(
            "Closing window (id: {}, title: {})",
            entry.platform.id(),
            entry.platform.title()
        );
    }

    fn quit(&mut self) {
        debug!("Quit signal received");
        let ids = self.windows.keys().copied().collect::<Vec<_>>();

        for id in ids {
            self.close_window(id);
        }
    }

    fn handle_sdl_event(&mut self, event: SDL_Event) {
        trace!("Received SDL event: {}", unsafe { event.type_ });

        match SDL_EventType(unsafe { event.type_ }) {
            SDL_EventType::SDL_EVENT_QUIT => self.quit(),

            SDL_EventType::SDL_EVENT_WINDOW_CLOSE_REQUESTED => {
                let window_id = unsafe { event.window.windowID };
                self.close_window(window_id);
            }

            SDL_EventType::SDL_EVENT_WINDOW_FOCUS_GAINED => {
                let window_id = unsafe { event.window.windowID };
                self.input.focused = Some(window_id);
                debug!("Window '{}' gained focus.", window_id);
            }

            SDL_EventType::SDL_EVENT_WINDOW_FOCUS_LOST => {
                let window_id = unsafe { event.window.windowID };

                if self.input.focused == Some(window_id) {
                    self.input.focused = None;
                    self.input.keys.clear_all();
                    self.input.mouse.clear_all();
                    debug!("Window '{}' lost focus.", window_id);
                }
            }

            SDL_EventType::SDL_EVENT_KEY_DOWN => {
                if !unsafe { event.key.repeat } {
                    let window_id = unsafe { event.key.windowID };
                    let scancode = unsafe { event.key.scancode };

                    if self.input.focused == Some(window_id)
                        && let Some(k) = Key::from_scancode(scancode)
                    {
                        self.input.keys.press(k);
                    }
                }
            }

            SDL_EventType::SDL_EVENT_KEY_UP => {
                let scancode = unsafe { event.key.scancode };

                if let Some(k) = Key::from_scancode(scancode) {
                    self.input.keys.release(k);
                }
            }

            SDL_EventType::SDL_EVENT_MOUSE_BUTTON_DOWN => {
                let window_id = unsafe { event.button.windowID };
                let btn = unsafe { event.button.button };

                if self.input.focused == Some(window_id)
                    && let Some(btn) = MouseButton::from_index(btn)
                {
                    self.input.mouse.press(btn);
                }
            }

            SDL_EventType::SDL_EVENT_MOUSE_BUTTON_UP => {
                let btn = unsafe { event.button.button };

                if let Some(btn) = MouseButton::from_index(btn) {
                    self.input.mouse.release(btn);
                }
            }

            SDL_EventType::SDL_EVENT_MOUSE_WHEEL => {
                let x = unsafe { event.wheel.x };
                let y = unsafe { event.wheel.y };

                self.input.m_wheel += math::Vector2::new(x, y);
            }

            _ => {
                let window = unsafe { event.window };

                if let Some(entry) = self.windows.get_mut(&window.windowID) {
                    entry.state.handle_window_event(&event);
                }
            }
        }
    }

    fn drain_app_events(&mut self) {
        let mut buffer = self.app_events.take();

        for event in buffer.drain(..) {
            trace!("Received app event: {:?}", event);

            match event {
                AppEvent::Window {
                    id,
                    event: win_event,
                } => {
                    let Some(entry) = self.windows.get_mut(&id) else {
                        trace!("Dropping event for closed window: {:?}", id);
                        continue;
                    };

                    match win_event {
                        WindowEvent::TitleChangeRequested(t) => {
                            entry.platform.set_title(&t);
                        }

                        WindowEvent::SizeChangeRequested(s) => {
                            entry.platform.set_size(s);
                        }

                        WindowEvent::FpsTargetChangeRequested(t) => {
                            entry.state.pacer.set_target_fps(t);
                        }

                        WindowEvent::FpsCountStrategyChangeRequested(s) => {
                            entry.state.pacer.counter.set_strategy(s);
                        }
                    }
                }

                AppEvent::TpsTargetChangeRequested(t) => {
                    self.clock.set_target_tps(t);
                }
            }
        }

        self.app_events.restore(buffer);
    }

    pub fn run(mut self) {
        for builder in mem::take(&mut self.requested_at_creation) {
            self.spawn_window(builder);
        }

        for entry in self.windows.values_mut() {
            entry.state.sync_time(&self.clock);
            entry.state.load_active(&self.input);
        }

        let mut event: MaybeUninit<SDL_Event> = MaybeUninit::uninit();

        while !self.should_quit {
            while unsafe { SDL_PollEvent(event.as_mut_ptr()) } {
                let event = unsafe { event.assume_init() };
                self.handle_sdl_event(event);
            }

            self.drain_app_events();

            if self.should_quit {
                break;
            }

            let now = Instant::now();
            self.clock.advance(now);

            while self.clock.should_tick() {
                self.input.change_scope(InputScope::Tick);

                for entry in self.windows.values_mut() {
                    entry.state.sync_time(&self.clock);
                    entry.state.update(UpdatePhase::Fixed, &self.input);
                }

                self.input.roll_tick();
                self.clock.consume();
            }

            self.input.change_scope(InputScope::Frame);
            let mut rendered = false;

            for entry in self.windows.values_mut() {
                if !entry.state.pacer.due(now) {
                    continue;
                }

                rendered = true;
                entry.state.pacer.record(now);
                entry.state.sync_time(&self.clock);

                entry.state.update(UpdatePhase::Unrestrained, &self.input);
                entry.state.draw(&self.input);

                self.gpu
                    .clear(&entry.platform, entry.state.ctx.window_handle.clear_color);

                entry.state.sync_window(&entry.platform);
            }

            if rendered {
                self.input.roll_frame();
            }

            let after_fraw = Instant::now();
            let tick = self.clock.next_tick();

            let deadline = self
                .windows
                .values()
                .map(|entry| {
                    entry
                        .state
                        .pacer
                        .deadline()
                        .unwrap_or(after_fraw + entry.state.pacer.idle_backoff())
                })
                .fold(tick, Instant::min);

            self.sleeper.sleep_until(deadline);
        }
    }
}
