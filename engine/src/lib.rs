pub mod assets;
mod builder;
mod event;
mod logs;
mod scene;
mod window;

pub mod render;

use std::mem;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;

use gpu::Gpu;
use imgui::SharedImgui;
use logging::debug;
use logging::error;
use logging::info;
use logging::warn;
use parking_lot::Mutex;
use sdl3::clipboard::ClipboardUtil;
use sdl3::event::Event;
use sdl3::event::WindowEvent;
use utils::FastHashMap;

use crate::assets::AssetServer;
use crate::render::Camera;
use crate::render::FramePacket;
use crate::render::Projection;
use crate::render::Renderer;
use crate::window::FrameSlot;
use crate::window::MainThreadRequest;
use crate::window::PresentPending;
use crate::window::Window;
use crate::window::WindowHandle;
use crate::window::WindowRequest;
use crate::window::context::WindowContext;
use crate::window::input::Input;
use crate::window::scene::Cameras;
use crate::window::state::WindowState;
use crate::window::time::Time;

pub use imgui;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::logs::init_logging;
pub use crate::render::Draw;
pub use crate::scene::Scene;
pub use crate::scene::SceneManager;
pub use crate::window::context::Context;
pub use crate::window::input;
pub use crate::window::scene::SceneView;

const IMMEDIATE_SHADER: usize = 0;
const IMGUI_SHADER: usize = 1;
pub(crate) const MESH_SHADER: usize = 2;

/// Everything that should be shared across windows,
/// e.g. the asset servers, should live in the app and be cloned around
/// (possibly with arc, so that the same instance is shared)
#[derive(Clone)]
struct AppOwned {
    /// The asset server uses arc internally, safe to clone
    assets: AssetServer,
    /// ?
    clipboard: Arc<ClipboardUtil>,
    /// Uses Arc internally, safe to clone
    imgui: SharedImgui,
}

pub struct App {
    // Field order matters: window handles own GPU resources (renderers),
    // so they must drop before the GPU context.
    windows: FastHashMap<u32, WindowHandle>,
    gpu: Gpu,
    events: sdl3::EventSubsystem,
    video: sdl3::VideoSubsystem,
    sdl: sdl3::Sdl,
    queued_windows: Vec<WindowBuilder>,
    owned: AppOwned,
}

impl App {
    fn new() -> Self {
        let sdl = sdl3::init().expect("Failed to init sdl3");
        let video = sdl.video().expect("Failed to init video subsystem");
        let events = sdl.event().expect("Failed to init event subsystem");

        events.register_custom_event::<WindowRequest>().unwrap();

        let mut gpu = Gpu::new();

        gpu.load_shader(
            IMMEDIATE_SHADER,
            gpu::ShaderDesc {
                vertex_spirv: include_bytes!(concat!(env!("OUT_DIR"), "/immediate.vert.spv")),
                fragment_spirv: include_bytes!(concat!(env!("OUT_DIR"), "/immediate.frag.spv")),
                vertex_uniform_buffers: 1,
                fragment_samplers: 1,
                fragment_uniform_buffers: 0,
            },
        );

        gpu.load_shader(
            IMGUI_SHADER,
            gpu::ShaderDesc {
                vertex_spirv: include_bytes!(concat!(env!("OUT_DIR"), "/imgui.vert.spv")),
                fragment_spirv: include_bytes!(concat!(env!("OUT_DIR"), "/imgui.frag.spv")),
                vertex_uniform_buffers: 1,
                fragment_samplers: 1,
                fragment_uniform_buffers: 0,
            },
        );

        gpu.load_shader(
            MESH_SHADER,
            gpu::ShaderDesc {
                vertex_spirv: include_bytes!(concat!(env!("OUT_DIR"), "/mesh.vert.spv")),
                fragment_spirv: include_bytes!(concat!(env!("OUT_DIR"), "/mesh.frag.spv")),
                vertex_uniform_buffers: 2,
                fragment_samplers: 1,
                fragment_uniform_buffers: 2,
            },
        );

        debug!("GPU initialized.");
        gpu.log_info();

        info!("Video driver: {:?}", video.current_video_driver());

        let assets = AssetServer::new();
        debug!("Asset Server initialized.");

        let clipboard = Arc::new(video.clipboard());
        debug!("Clipboard initialized.");

        let imgui = SharedImgui::new();
        debug!("Imgui context manager initialized.");

        Self {
            windows: FastHashMap::default(),
            gpu,
            events,
            video,
            sdl,
            queued_windows: Vec::new(),
            owned: AppOwned {
                assets,
                clipboard,
                imgui,
            },
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    pub(crate) fn queue_window(&mut self, b: WindowBuilder) {
        self.queued_windows.push(b);
    }

    fn spawn_window(&mut self, b: WindowBuilder) {
        let sdl_builder = self.video.window(&b.title, b.size.width, b.size.height);
        let window = b.build_sdl(sdl_builder);

        // Swapchain lives on the main thread
        self.gpu.claim_window(&window);

        let format = self.gpu.swapchain_format(&window);
        let renderer = Renderer::new(&self.gpu, format, self.owned.assets.reader());
        debug!("Renderer initialized.");

        let window_id = window.id();
        let window_title = window.title().to_owned();
        let window_size = window.size().into();

        self.owned
            .imgui
            .register_window(window_id, window_size, self.owned.clipboard.clone());

        // Channels
        let (event_tx, event_rx) = mpsc::channel::<Event>();
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
        let (recycle_tx, recycle_rx) = mpsc::channel::<render::RenderLayers>();
        let frame_slot: FrameSlot = Arc::new(Mutex::new(None));
        let thread_slot = frame_slot.clone();
        let present_pending: PresentPending = Arc::new(AtomicBool::new(false));
        let thread_pending = present_pending.clone();
        let proxy = self.events.event_sender();
        let assets = self.owned.assets.clone();
        let imgui = self.owned.imgui.clone();

        let thread = thread::spawn(move || {
            WindowState {
                should_exit: false,
                context: WindowContext {
                    window: Window::new(
                        window_id,
                        window_title,
                        window_size,
                        proxy,
                        thread_pending,
                    ),
                    input: Input::new(),
                    time: Time::new(),
                    scenes: SceneManager::new(),
                    assets,
                    cameras: Cameras {
                        world: Camera::new(Projection::standard_2d(window_size)),
                        ui: Camera::new(Projection::standard_2d(window_size)),
                        debug: Camera::new(Projection::standard_2d(window_size)),
                    },
                    layers: Default::default(),
                    imgui,
                },
                scenes: b.scenes,
                active_scenes: b.initial_active,
                events: event_rx,
                shutdown: shutdown_rx,
                packet: FramePacket::default(),
                frame_slot: thread_slot,
                recycled: recycle_rx,
                pending_input_timestamp: None,
            }
            .run_loop();
        });

        self.windows.insert(
            window_id,
            WindowHandle {
                window,
                renderer,
                frame_slot,
                present_pending,
                recycle: recycle_tx,
                event_sender: event_tx,
                shutdown: shutdown_tx,
                thread,
            },
        );
    }

    fn close_window(&mut self, window_id: u32) {
        let Some(handle) = self.windows.remove(&window_id) else {
            warn!("Trying to close a window that doesnt exist anymore");
            return;
        };

        let _ = handle.shutdown.send(());
        let _ = handle.thread.join();

        self.owned.imgui.unregister_window(window_id);
    }

    pub fn run(mut self) {
        if self.queued_windows.is_empty() {
            warn!("No window was requested. Exiting.");
            return;
        }

        for b in mem::take(&mut self.queued_windows) {
            self.spawn_window(b);
        }

        let mut pump = self.sdl.event_pump().expect("Failed to get event pump");

        'main: loop {
            let event = pump.wait_event();

            if let Some(req) = event.as_user_event_type::<WindowRequest>() {
                match req.request {
                    MainThreadRequest::SetWindowTitle(t) => {
                        if let Some(handle) = self.windows.get_mut(&req.window_id) {
                            let _ = handle.window.set_title(&t);
                            debug!("Title set to '{}' for window {}", t, req.window_id);
                        }
                    }

                    MainThreadRequest::SetWindowSize(size) => {
                        if let Some(handle) = self.windows.get_mut(&req.window_id) {
                            let _ = handle.window.set_size(size.width, size.height);
                            debug!("Size set to {:?} for window {}", size, req.window_id);
                        }
                    }

                    MainThreadRequest::Present => {
                        if let Some(handle) = self.windows.get_mut(&req.window_id) {
                            // Clear before taking the slot: a frame finished
                            // after this point must queue a fresh Present.
                            handle.present_pending.store(false, Ordering::Release);

                            let frame = handle.frame_slot.lock().take();

                            if let Some(frame) = frame {
                                handle.renderer.present(&self.gpu, &handle.window, &frame);
                                let _ = handle.recycle.send(frame.layers);
                            }
                        }
                    }
                }

                continue 'main;
            }

            match event {
                Event::Quit { .. } => break 'main,
                Event::Window {
                    timestamp,
                    window_id: w_id,
                    win_event: w_event,
                } => match w_event {
                    WindowEvent::CloseRequested => {
                        self.close_window(w_id);
                        info!("Closing window '{}'", w_id);

                        if self.windows.is_empty() {
                            info!("All windows were closed. Exiting.");
                            break 'main;
                        }
                    }

                    event => {
                        let Some(handle) = self.windows.get(&w_id) else {
                            warn!("Received event for a window that doesnt exist anymore");
                            return;
                        };

                        if let Err(e) = handle.event_sender.send(Event::Window {
                            timestamp,
                            window_id: w_id,
                            win_event: event,
                        }) {
                            error!("Failed to send event to window: {}", e);
                            return;
                        }
                    }
                },

                Event::KeyDown { window_id, .. }
                | Event::KeyUp { window_id, .. }
                | Event::MouseMotion { window_id, .. }
                | Event::MouseButtonDown { window_id, .. }
                | Event::MouseButtonUp { window_id, .. }
                | Event::MouseWheel { window_id, .. }
                | Event::TextInput { window_id, .. } => {
                    if let Some(handle) = self.windows.get(&window_id) {
                        let _ = handle.event_sender.send(event);
                    }
                }

                _ => {}
            }
        }
    }
}
