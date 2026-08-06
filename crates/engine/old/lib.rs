mod builder;
mod clock;
mod conf;
mod event;
mod input;
mod log;
mod render;
mod scene;
mod window;

use std::mem;
use std::time::Instant;

use gpu::Gpu;
use logging::debug;
use logging::error;
use logging::fatal;
use logging::info;
use logging::trace;
use sdl3::GamepadSubsystem;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::joystick::JoystickId;
use utils::FastHashMap;
use utils::SleepTimer;

use crate::clock::Clock;
use crate::conf::config;
use crate::event::AppEvent;
use crate::event::EventQueue;
use crate::event::SdlEvent;
use crate::event::SdlWindowEvent;
use crate::input::Input;
use crate::window::SdlWindow;
use crate::window::WindowEntry;
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
pub use crate::log::init_logging;
pub use crate::render::color::Color;
pub use crate::render::draw::Draw;
pub use crate::render::stage::SceneView;
pub use crate::scene::Scene;
pub use crate::scene::SceneId;
pub use crate::window::WindowHandle;
pub use crate::window::context::DrawContext;
pub use crate::window::context::LoadContext;
pub use crate::window::context::UpdateContext;

pub struct App {
    requested_at_creation: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowEntry>,
    should_quit: bool,
    app_events: EventQueue<AppEvent>,

    clock: Clock,
    input: Input,
    sleeper: SleepTimer,

    // SDL / SDL wrappers
    gpu: Gpu,
    gamepad: GamepadSubsystem,
    video: VideoSubsystem,
    sdl: Sdl,
}

impl App {
    pub(crate) fn new() -> Self {
        let conf = config();

        let sdl = match sdl3::init() {
            Ok(s) => s,
            Err(e) => fatal!("Failed to initialize SDL{}: {}", conf.sdl_version, e),
        };

        info!("SDL v{} Initialized.", conf.sdl_version);

        let video = match sdl.video() {
            Ok(v) => v,
            Err(e) => fatal!("Failed to initialize video subsystem: {}", e),
        };

        debug!("Video subsystem initialized.");

        let gamepad = match sdl.gamepad() {
            Ok(j) => j,
            Err(e) => fatal!("Failed to initialize gamepad subsystem: {}", e),
        };

        debug!("Joystick subsystem initialized.");

        let gpu = match Gpu::init() {
            Ok(g) => g,
            Err(e) => fatal!("Failed to initialize GPU: {}", e),
        };

        gpu.log_info();

        // Load shaders...
        debug!("Loaded built-in shaders.");

        info!("Application initialized. (karna v{})", conf.karna_verson);

        Self {
            requested_at_creation: Vec::new(),
            windows: FastHashMap::default(),
            should_quit: false,
            app_events: EventQueue::new(),
            clock: Clock::default(),
            input: Input::default(),
            sleeper: SleepTimer::calibrated(),
            gpu,
            gamepad,
            video,
            sdl,
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    fn spawn_window(&mut self, b: WindowBuilder) {
        let conf = config();
        let sdl_window = match self
            .video
            .window(&b.title, b.size.width, b.size.height)
            .build()
        {
            Ok(w) => w,
            Err(e) => return error!("Failed to create window: {}", e),
        };

        if let Err(e) = self.gpu.claim_window(&sdl_window) {
            error!("GPU failed to claim the window: {}", e);
            return;
        };

        self.gpu.set_present_mode(&sdl_window, conf.present_mode);

        let mut scenes = FastHashMap::default();

        for (id, builder) in b.scenes {
            scenes.insert(id, SceneSlot::Unloaded(builder));
        }

        let window = PlatformWindow::new(sdl_window);
        let state = WindowState {
            ctx: UserContext {
                window: WindowHandle::new(&window, self.app_events.dispatcher()),
            },
            pacer: FramePacer::new(PaceMode::Fixed),
            draw: Draw::new(),
            scenes,
            active_scenes: b.active_scenes,
            dispatcher: self.app_events.dispatcher(),
        };

        self.windows
            .insert(window.id(), WindowEntry { window, state });
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            error!("Received an event for a closed window.");
            return;
        };

        if self.input.focused == Some(id) {
            self.input.focused = None;
            self.input.keys.clear_all();
            self.input.mouse.clear_all();
        }

        self.gpu.release_window(&entry.window);

        if self.windows.is_empty() {
            self.should_quit = true;
        }

        info!(
            "Closing window (id: {}, title: {})",
            entry.window.id(),
            entry.window.title()
        )
    }

    fn quit(&mut self) {
        debug!("Quit signal received.");
        let ids = self.windows.iter().map(|(id, _)| *id).collect::<Vec<_>>();

        for id in ids {
            self.close_window(id);
        }
    }

    fn handle_sdl_event(&mut self, event: SdlEvent) {
        match event {
            SdlEvent::Quit { .. } => return self.quit(),
            SdlEvent::Window {
                window_id,
                win_event: SdlWindowEvent::CloseRequested,
                ..
            } => return self.close_window(window_id),

            SdlEvent::Window {
                window_id,
                win_event: SdlWindowEvent::FocusGained,
                ..
            } => {
                self.input.focused = Some(window_id);
                debug!("Window '{}' gained focus.", window_id);
            }

            SdlEvent::Window {
                window_id,
                win_event: SdlWindowEvent::FocusLost,
                ..
            } => {
                if self.input.focused == Some(window_id) {
                    self.input.focused = None;
                    self.input.keys.clear_all();
                    self.input.mouse.clear_all();
                    debug!("Window '{}' lost focus.", window_id);
                }
            }

            SdlEvent::KeyDown {
                window_id,
                scancode: Some(k),
                repeat: false,
                ..
            } => {
                if self.input.focused == Some(window_id) {
                    self.input.keys.press(k);
                }
            }

            SdlEvent::KeyUp {
                scancode: Some(k), ..
            } => self.input.keys.release(k),

            SdlEvent::MouseButtonDown {
                window_id,
                mouse_btn,
                ..
            } => {
                if self.input.focused == Some(window_id) {
                    self.input.mouse.press(mouse_btn);
                }
            }

            SdlEvent::MouseButtonUp { mouse_btn, .. } => self.input.mouse.release(mouse_btn),

            SdlEvent::MouseWheel { x, y, .. } => {
                self.input.m_wheel += math::Vector2::new(x, y);
            }

            SdlEvent::ControllerDeviceAdded { which, .. } => {
                let id = JoystickId::new(which);
                match self.gamepad.open(id) {
                    Ok(pad) => {
                        info!(
                            "Gamepad connected: {}",
                            pad.name().unwrap_or(String::from("unnamed gamepad"))
                        );
                        self.input.gamepads.insert(id, pad);
                    }

                    Err(e) => error!("Failed to open gamepad: {}", e),
                }
            }

            SdlEvent::ControllerDeviceRemoved { which, .. } => {
                let id = JoystickId::new(which);
                self.input.gamepads.remove(&id);
            }

            event => {
                if let Some(id) = event.get_window_id()
                    && let Some(entry) = self.windows.get_mut(&id)
                {
                    entry.state.handle_event(event);
                }
            }
        }
    }

    fn handle_app_events(&mut self) {
        let mut batch = self.app_events.take();

        for event in batch.drain(..) {
            trace!("Received app event: {:?}", event);

            let Some(id) = event.get_window_id() else {
                continue;
            };

            let Some(entry) = self.windows.get_mut(&id) else {
                trace!("Dropping event for closed window: {:?}", id);
                continue;
            };

            match event {
                AppEvent::Window { event, .. } => {
                    entry.window.handle_event(event, &mut entry.state.pacer)
                }

                AppEvent::TpsTargetChangeRequested(t) => self.clock.set_target_tps(t),
            }
        }

        self.app_events.restore(batch);
    }

    pub fn run(mut self) {
        for builder in mem::take(&mut self.requested_at_creation) {
            self.spawn_window(builder);
        }

        let mut pump = match self.sdl.event_pump() {
            Ok(p) => p,
            Err(e) => fatal!("Failed to initialize SDL event pump: {}", e),
        };

        debug!("SDL event pump initialized.");

        for entry in self.windows.values_mut() {
            entry.state.load_active(&self.clock);
        }

        while !self.should_quit {
            for event in pump.poll_iter() {
                self.handle_sdl_event(event);
            }

            self.handle_app_events();

            if self.should_quit {
                break;
            }

            let now = Instant::now();
            self.clock.advance(now);

            while self.clock.should_tick() {
                for entry in self.windows.values_mut() {
                    entry.state.update(UpdatePhase::FixedUpdate, &self.clock);
                }

                self.input.roll_tick();
                self.clock.consume();
            }

            for entry in self.windows.values_mut() {
                if !entry.state.pacer.due(now) {
                    continue;
                }

                entry.state.pacer.record(now);
                entry.state.update(UpdatePhase::Update, &self.clock);
                entry.state.draw(&self.clock);

                let _ = self.gpu.clear(
                    &entry.window,
                    entry.state.ctx.window.clear_color().into(),
                    self.gpu.present_mode() == gpu::PresentMode::Vsync,
                );

                entry.state.ctx.window.roll_frame();
            }

            self.input.roll_frame();

            let after_draw = Instant::now();
            let tick = self.clock.next_tick();

            let deadline = self
                .windows
                .values()
                .map(|e| {
                    e.state
                        .pacer
                        .deadline()
                        .unwrap_or(after_draw + e.state.pacer.idle_backoff())
                })
                .fold(tick, Instant::min);

            self.sleeper.sleep_until(deadline);
        }
    }
}
