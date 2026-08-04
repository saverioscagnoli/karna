mod assets;
mod builder;
mod event;
mod render;
mod scene;
mod sound;
mod window;

use std::mem;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use gpu::Gpu;
use imgui::Imgui;
use logging::debug;
use logging::error;
use logging::fatal;
use logging::info;
use logging::trace;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::gpu::PresentMode;
use utils::FastHashMap;
use utils::SleepTimer;
use utils::WindowId;

use crate::assets::AssetServer;
use crate::event::AppEvent;
use crate::event::EventQueue;
use crate::event::SdlEvent;
use crate::event::SdlWindowEvent;
use crate::render::Renderer;
use crate::render::packet::FramePacket;
use crate::render::retained::RenderWorld;
use crate::scene::World;
use crate::sound::SharedMixer;
use crate::window::audio::AudioHandle;
use crate::window::context::WindowContext;
use crate::window::platform::PlatformWindow;
use crate::window::resources::Resources;
use crate::window::state::Frame;
use crate::window::state::WindowState;
use crate::window::time::TimeCommand;

pub use crate::assets::Assets;
pub use crate::assets::audio::Audio;
pub use crate::assets::audio::AudioLength;
pub use crate::assets::font::Font;
pub use crate::assets::font::Rasterize;
pub use crate::assets::image::Image;
pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::render::MESH_SHADER;
pub use crate::render::camera::Camera;
pub use crate::render::camera::Fit;
pub use crate::render::camera::Projection;
pub use crate::render::color::Color;
pub use crate::render::geometry::Geometry;
pub use crate::render::geometry::cube;
pub use crate::render::immediate::Draw;
pub use crate::render::layer::Layer;
pub use crate::render::material::AlphaMode;
pub use crate::render::material::Material;
pub use crate::render::material::MaterialDesc;
pub use crate::render::material::PassState;
pub use crate::render::mesh::Mesh;
pub use crate::render::retained::SceneRef;
pub use crate::render::vertex::MeshVertex;
pub use crate::scene::Scene;
pub use crate::window::WindowHandle;
pub use crate::window::context::ContextRef;
pub use crate::window::context::DrawContext;
pub use crate::window::input::Input;
pub use crate::window::input::Key;
pub use crate::window::input::MouseButton;
pub use crate::window::time::Time;
pub use imgui::Condition;
pub use imgui::Ui;
pub use imgui::WindowFlags;
pub use imgui::WindowToken;

struct WindowEntry {
    platform: PlatformWindow,
    state: WindowState,
    imgui: Imgui,
}

pub struct App {
    queued_windows: Vec<WindowBuilder>,

    windows: FastHashMap<WindowId, WindowEntry>,
    should_quit: bool,
    sleeper: SleepTimer,
    events: EventQueue<AppEvent>,
    asset_server: AssetServer,
    assets: Assets,
    mixer: Arc<SharedMixer>,
    renderer: Renderer,
    gpu: Gpu,
    video: VideoSubsystem,
    sdl: Sdl,
}

impl App {
    pub(crate) fn new(asset_workers: usize, asset_root: Option<PathBuf>) -> Self {
        let sdl = match sdl3::init() {
            Ok(sdl) => sdl,
            Err(e) => fatal!("Failed to initialize sdl: {}", e),
        };
        debug!("SDL3 initialized.");

        let sleeper = SleepTimer::calibrated();

        let video = match sdl.video() {
            Ok(v) => v,
            Err(e) => fatal!("Failed to initialize video subsystem: {}", e),
        };
        debug!("Video subsystem initialized.");

        let mut gpu = match Gpu::init() {
            Ok(gpu) => gpu,
            Err(e) => fatal!("Failed to initialize gpu: {}", e),
        };

        gpu.log_shader_formats();
        gpu.log_info();

        render::load_builtin_shaders(&mut gpu);
        debug!("Loaded built-in shaders.");

        let mixer = match SharedMixer::open() {
            Ok(m) => Arc::new(m),
            Err(e) => fatal!("Failed to initialize audio: {}", e),
        };

        debug!("Audio mixer initialized.");

        let root = match asset_root {
            Some(r) => r,
            None => assets::resolve_base_path(),
        };

        let (asset_server, assets) = assets::spawn(root, asset_workers, Arc::clone(&mixer));

        info!("Application initialized.");

        Self {
            queued_windows: Vec::new(),
            windows: FastHashMap::default(),
            should_quit: false,
            sleeper,
            events: EventQueue::new(),
            asset_server,
            assets,
            mixer,
            renderer: Renderer::new(),
            gpu,
            video,
            sdl,
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    fn spawn_window(&mut self, b: WindowBuilder) {
        let window = match self
            .video
            .window(&b.title, b.size.width, b.size.height)
            .build()
        {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create window: {}", e);
                return;
            }
        };

        if let Err(e) = self.gpu.claim_window(&window) {
            error!("The GPU failed to claim the window: {}", e);
            return;
        }

        self.gpu.set_present_mode(&window, PresentMode::Immediate);

        let platform = PlatformWindow::new(window);

        debug!(
            "window {:?} logical {:?} pixels {:?} density {}",
            platform.id(),
            platform.size(),
            platform.pixel_size(),
            platform.inner().pixel_density() / platform.inner().display_scale() // SDL_GetWindowPixelDensity / SDL_GetWindowDisplayScale
        );

        let handle = WindowHandle::new(
            platform.id(),
            platform.size(),
            platform.pixel_size(),
            platform.title(),
            platform.is_resizable(),
            self.events.dispatcher(),
        );

        let state = WindowState::new(
            WindowContext {
                window: handle,
                input: Input::default(),
                audio: AudioHandle::new(Arc::clone(&self.mixer)),
                resources: Resources::default(),
                render: RenderWorld::default(),
                packet: FramePacket::default(),
            },
            World::new(b.scenes, b.active_scenes),
        );

        self.windows.insert(
            state.window.id(),
            WindowEntry {
                platform,
                state,
                imgui: Imgui::new(),
            },
        );
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            error!("Received an event for a closed window.");
            return;
        };

        self.gpu.release_window(&entry.platform.inner());
        info!("Closing window '{}'", entry.platform.title());

        if self.windows.is_empty() {
            self.should_quit = true;
        }
    }

    fn quit(&mut self) {
        debug!("Quit signal received.");

        for (_, entry) in self.windows.drain() {
            self.gpu.release_window(&entry.platform.inner());
        }

        self.should_quit = true;
    }

    fn handle_sdl_event(&mut self, event: SdlEvent) {
        match event {
            SdlEvent::Quit { .. } => return self.quit(),

            SdlEvent::Window {
                window_id,
                win_event: SdlWindowEvent::CloseRequested,
                ..
            } => return self.close_window(window_id),

            event => {
                if let Some(id) = event.get_window_id()
                    && let Some(entry) = self.windows.get_mut(&id)
                {
                    entry.imgui.handle_event(&event);

                    // Whatever the ui swallowed must not also reach the game,
                    // or clicking a slider fires the scene's click handler too.
                    let capture = entry.imgui.capture();
                    let for_game = match event {
                        SdlEvent::MouseMotion { .. }
                        | SdlEvent::MouseButtonDown { .. }
                        | SdlEvent::MouseButtonUp { .. }
                        | SdlEvent::MouseWheel { .. } => !capture.mouse,

                        SdlEvent::KeyDown { .. }
                        | SdlEvent::KeyUp { .. }
                        | SdlEvent::TextInput { .. } => !capture.keyboard,

                        _ => true,
                    };

                    if for_game {
                        entry.state.handle_window_event(event, &mut entry.platform);
                    }
                }
            }
        }
    }

    fn drain_app_events(&mut self) {
        let mut batch = self.events.take();

        for event in batch.drain(..) {
            trace!("Received app event: {:?}", event);

            match event {
                AppEvent::Time(req) => {
                    let Some(entry) = self.windows.get_mut(&req.window) else {
                        error!("Received a command for a closed window.");
                        continue;
                    };

                    match req.command {
                        TimeCommand::SetTargetFps(t) => entry.state.pacer.set_fps(t),
                        TimeCommand::SetTargetTps(t) => entry.state.clock.set_tps(t),
                        TimeCommand::SetPresentMode(mode) => {
                            if self.gpu.supports_present_mode(entry.platform.inner(), mode) {
                                self.gpu.set_present_mode(&entry.platform.inner(), mode);
                            }
                        }
                    }
                }

                AppEvent::Window(req) => {
                    let Some(entry) = self.windows.get_mut(&req.window) else {
                        error!("Received a command for a closed window.");
                        continue;
                    };

                    entry.platform.apply(req.command);
                }
            }
        }

        self.events.restore(batch);
    }

    pub fn run(mut self) {
        for b in mem::take(&mut self.queued_windows) {
            self.spawn_window(b);
        }

        let mut pump = match self.sdl.event_pump() {
            Ok(p) => p,
            Err(e) => fatal!("Failed to initialize SDL event pump: {}", e),
        };

        for e in self.windows.values_mut() {
            e.state.load(Frame {
                assets: &mut self.assets,
                mode: self.gpu.present_mode(),
                events: self.events.dispatcher(),
            });
        }

        while !self.should_quit {
            for event in pump.poll_iter() {
                self.handle_sdl_event(event);
            }

            self.drain_app_events();
            self.assets.poll(&self.gpu);

            let now = Instant::now();

            for e in self.windows.values_mut() {
                e.state.window.sync(&e.platform);
                e.state.clock.advance(now);

                while e.state.clock.should_tick() {
                    e.state.fixed_update(Frame {
                        assets: &mut self.assets,
                        mode: self.gpu.present_mode(),
                        events: self.events.dispatcher(),
                    });
                    e.state.clock.consume(now);
                }

                if !e.state.pacer.due(now) {
                    continue;
                }

                e.state.update(Frame {
                    assets: &mut self.assets,
                    mode: self.gpu.present_mode(),
                    events: self.events.dispatcher(),
                });

                e.state.draw(
                    Frame {
                        assets: &mut self.assets,
                        mode: self.gpu.present_mode(),
                        events: self.events.dispatcher(),
                    },
                    &mut e.imgui,
                );

                let draw_data = e.imgui.end_frame();

                self.renderer.present(
                    &self.gpu,
                    &e.platform,
                    &self.assets,
                    &e.state.context.packet,
                    &mut e.state.context.render,
                    Some((&mut e.imgui.renderer, draw_data)),
                );

                e.state.pacer.record(now);
                e.state.pacer.schedule(now);

                e.state.flush();
            }

            let deadline = self
                .windows
                .values()
                .map(|e| e.state.pacer.next_frame.min(e.state.clock.next_tick()))
                .min();

            if self.gpu.present_mode() == PresentMode::Vsync {
                if deadline.is_none() {
                    self.should_quit = true;
                }
            } else {
                match deadline {
                    Some(d) => self.sleeper.sleep_until(d),
                    None => self.should_quit = true,
                };
            }
        }

        info!("All windows were closed. Exiting.");
        self.asset_server.shutdown(self.assets);
    }
}
