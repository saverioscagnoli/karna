use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::Receiver;

use sdl3::sys::keyboard;
use utils::FastHashMap;
use utils::WindowId;

use crate::render::view::View;
use crate::window::SdlEvent;
use crate::window::context::WindowContext;

type SimulationId = usize;

pub struct Simulation {
    views: Vec<View>,
    context: WindowContext,
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
    simulation: Vec<Simulation>,
    window_sim_route: FastHashMap<WindowId, SimulationId>,
}
