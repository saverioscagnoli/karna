mod builder;
mod conf;
mod event;
mod log;
mod render;
mod scene;
mod window;

use std::mem;

use gpu::Gpu;
use logging::debug;
use logging::error;
use logging::fatal;
use logging::info;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use utils::FastHashMap;

use crate::conf::config;
use crate::event::AppEvent;
use crate::event::EventQueue;
use crate::window::WindowEntry;
use crate::window::WindowId;
use crate::window::context::UserContext;
use crate::window::platform::PlatformWindow;
use crate::window::state::SceneSlot;
use crate::window::state::WindowState;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::log::init_logging;
pub use crate::render::draw::Draw;
pub use crate::render::scene_ref::SceneRef;
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

    // SDL / SDL wrappers
    gpu: Gpu,
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

        info!("SDL{} Initialized.", conf.sdl_version);

        let video = match sdl.video() {
            Ok(v) => v,
            Err(e) => fatal!("Failed to initialize video subsystem: {}", e),
        };

        debug!("Video subsystem initialized.");

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
            gpu,
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
            scenes,
            active_scenes: b.active_scenes,
        };

        self.windows
            .insert(window.id(), WindowEntry { window, state });
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            error!("Received an event for a closed window.");
            return;
        };

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
            entry.state.load_active();
        }

        debug!("Loaded initially active scenes.");

        self.quit();
    }
}
