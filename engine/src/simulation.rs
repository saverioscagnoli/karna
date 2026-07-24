use std::clone;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use logging::debug;
use logging::info;
use sdl3::sys::keyboard;
use sdl3::sys::keycode::SDL_KMOD_LALT;
use utils::FastHashMap;
use utils::WindowId;

use crate::render::view::View;
use crate::scene::SceneRegistry;
use crate::window::SdlEvent;
use crate::window::SdlWindow;
use crate::window::SdlWindowEvent;
use crate::window::WindowAction;
use crate::window::WindowHandle;
use crate::window::context::WindowContext;

type SimulationId = usize;

pub struct Simulation {
    views: Vec<View>,
    context: WindowContext,
    scenes: SceneRegistry,
}

impl Deref for Simulation {
    type Target = WindowContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for Simulation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

pub struct SimulationRunner {
    simulations: FastHashMap<WindowId, Simulation>,
    event_receiver: Receiver<SdlEvent>,
}

impl SimulationRunner {
    pub fn new(e: Receiver<SdlEvent>) -> Self {
        Self {
            simulations: FastHashMap::default(),
            event_receiver: e,
        }
    }

    pub fn insert_window(&mut self, window: WindowHandle) {
        let window_id = window.id();
        let simulation = Simulation {
            views: Vec::new(),
            context: WindowContext { window },
            scenes: SceneRegistry::default(),
        };

        self.simulations.insert(window_id, simulation);
    }

    fn handle_event(&mut self, event: SdlEvent) {
        match event {
            SdlEvent::Window {
                window_id,
                win_event,
                ..
            } => match win_event {
                SdlWindowEvent::CloseRequested => {
                    self.simulations.remove(&window_id);
                }

                _ => {}
            },

            _ => {}
        }
    }

    pub fn run(mut self) {
        'sim: loop {
            loop {
                match self.event_receiver.try_recv() {
                    Ok(event) => self.handle_event(event),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        break 'sim;
                    }
                }
            }

            if self.simulations.is_empty() {
                break 'sim;
            }

            std::thread::sleep(Duration::from_millis(16));
        }
    }
}
