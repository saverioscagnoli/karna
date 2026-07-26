mod assets;
mod builder;
mod logs;
mod render;
mod scene;
mod simulation;
mod window;

use std::mem;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};

use gpu::Gpu;
use logging::{debug, error, info};
use sdl3::sys::video::SDL_SetWindowResizable;
use sdl3::video::WindowPos;
use utils::{FastHashMap, Lazy, WindowId};

use crate::render::Renderer;
use crate::render::packet::FramePacket;
use crate::simulation::SimulationRunner;
use crate::window::{FrameSubmission, SdlEvent, SdlWindowEvent, WindowRequest, WindowSlot};
use crate::window::{WindowAction, WindowHandle};

pub use crate::builder::WindowBuilder;
pub use crate::logs::init_logging;
pub use crate::render::color::Color;
pub use crate::render::immediate::Draw;
pub use crate::render::retained::SceneRef;
pub use crate::scene::Scene;
pub use crate::window::context::ContextRef;
pub use crate::window::input;

// Drop order matters
pub struct App {
    /// The window queued at creation
    queued_windows: Vec<WindowBuilder>,

    windows: FastHashMap<WindowId, WindowSlot>,
    sims: Option<JoinHandle<()>>,
    action_sender: Sender<WindowRequest>,
    action_receiver: Receiver<WindowRequest>,
    frame_sender: Sender<FrameSubmission>,
    frame_receiver: Receiver<FrameSubmission>,
    event_sender: Lazy<Sender<SdlEvent>>,
    gpu: Gpu,
    video: sdl3::VideoSubsystem,
    sdl: sdl3::Sdl,
}

impl App {
    pub(crate) fn new() -> Self {
        let sdl = sdl3::init().expect("Failed to init sdl3");
        let video = sdl.video().expect("Failed to init video subsystem");
        let mut gpu = Gpu::init();

        let (action_tx, action_rx) = channel::<WindowRequest>();
        let (frame_tx, frame_rx) = channel::<FrameSubmission>();

        gpu.log_shader_formats();
        gpu.log_info();

        render::load_builtin_shaders(&mut gpu);

        Self {
            queued_windows: Vec::new(),
            windows: FastHashMap::default(),
            sims: None,
            action_sender: action_tx,
            action_receiver: action_rx,
            frame_sender: frame_tx,
            frame_receiver: frame_rx,
            event_sender: Lazy::unset(),
            gpu,
            video,
            sdl,
        }
    }

    fn spawn_sims(&mut self) {
        let (event_tx, event_rx) = channel::<SdlEvent>();
        let action_tx = self.action_sender.clone();
        let frame_tx = self.frame_sender.clone();
        let mut builders = mem::take(&mut self.queued_windows);
        let mut ids = Vec::new();

        self.event_sender.set(event_tx);

        for b in &builders {
            let window = self
                .video
                .window(&b.title, b.size.width, b.size.height)
                .build()
                .expect("Failed to create window");

            self.gpu
                .claim_window(&window)
                .expect("Failed to claim window");

            let id = window.id();
            ids.push(id);

            let renderer = Renderer::new(&self.gpu, self.gpu.swapchain_format(&window));

            self.windows.insert(
                id,
                WindowSlot {
                    inner: window,
                    renderer,
                    packet: FramePacket::default(),
                },
            );
        }

        self.sims = Some(thread::spawn(move || {
            let mut runner = SimulationRunner::new(event_rx);

            for (b, id) in builders.drain(..).zip(ids) {
                runner.insert_window(
                    WindowHandle {
                        action_sender: action_tx.clone(),
                        frame_sender: frame_tx.clone(),
                        id,
                        position: math::Vector2::zero(),
                        resizable: false,
                        cached_size: b.size,
                        cached_title: b.title.clone().into(),
                    },
                    b,
                );
            }

            runner.run();
        }));
    }

    fn close_window(&mut self, id: WindowId) {
        if let Some(slot) = self.windows.remove(&id) {
            info!("Closing window '{}'", id);
            self.gpu.release_window(&slot.inner);
        }
    }

    pub fn run(mut self) {
        self.spawn_sims();

        let mut pump = self.sdl.event_pump().expect("Failed to get event pump");

        'main: loop {
            for event in pump.poll_iter() {
                match event {
                    SdlEvent::Quit { .. } => break 'main,
                    SdlEvent::Window {
                        timestamp,
                        window_id,
                        win_event,
                    } => {
                        let forwarded = SdlEvent::Window {
                            timestamp,
                            window_id,
                            win_event,
                        };

                        if let Err(e) = self.event_sender.send(forwarded) {
                            error!("Failed to send window event: {}", e);
                        }

                        if matches!(win_event, SdlWindowEvent::CloseRequested) {
                            self.close_window(window_id);

                            if self.windows.is_empty() {
                                break 'main;
                            }
                        }
                    }

                    event => {
                        if let Err(e) = self.event_sender.send(event) {
                            error!("Failed to send window event: {}", e);
                        }
                    }
                }
            }

            while let Ok(request) = self.action_receiver.try_recv() {
                let Some(slot) = self.windows.get_mut(&request.window_id) else {
                    continue;
                };

                match request.action {
                    WindowAction::Center => {
                        slot.inner
                            .set_position(WindowPos::Centered, WindowPos::Centered);
                        debug!("Centering window");
                    }

                    WindowAction::SetPosition(pos) => {
                        slot.inner.set_position(
                            WindowPos::Positioned(pos.x),
                            WindowPos::Positioned(pos.y),
                        );

                        debug!("Setting window position to ({}, {})", pos.x, pos.y);
                    }

                    WindowAction::SetResizable(r) => unsafe {
                        // idk why sdl3 bindings dont expose this
                        SDL_SetWindowResizable(slot.inner.raw(), r);
                        debug!("Setting window resizable: {}", r);
                    },

                    WindowAction::SetSize(size) => {
                        let _ = slot.inner.set_size(size.width, size.height);
                        debug!("Setting window size to {:?}", size);
                    }

                    WindowAction::SetTitle(title) => {
                        let _ = slot.inner.set_title(&title);
                        debug!("Setting window title to '{}'", title);
                    }
                }
            }

            while let Ok(submission) = self.frame_receiver.try_recv() {
                if let Some(slot) = self.windows.get_mut(&submission.window_id) {
                    slot.packet = submission.packet;
                }
            }

            for slot in self.windows.values_mut() {
                slot.renderer.present(&self.gpu, &slot.inner, &slot.packet);
            }
        }

        info!("All windows were closed. Exiting.");

        if let Some(t) = self.sims.take() {
            let _ = t.join();
        }
    }
}
