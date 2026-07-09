use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use winit::event::WindowEvent;

use crate::time::Time;
use crate::window::Window;

pub struct WindowState {
    should_exit: bool,
    window: Window,
    time: Time,

    // Channels
    event_rx: Receiver<WindowEvent>,
    draw_tx: Sender<()>,
}

impl WindowState {
    pub fn new(window: Window, event_rx: Receiver<WindowEvent>, draw_tx: Sender<()>) -> Self {
        Self {
            should_exit: false,
            window,
            time: Time::new(),
            event_rx,
            draw_tx,
        }
    }

    pub fn start(mut self) {
        while !self.should_exit {
            for event in self.event_rx.try_iter() {
                match event {
                    WindowEvent::CloseRequested => {
                        self.should_exit = true;
                    }

                    _ => {}
                }
            }

            println!("dt {}", self.time.delta());

            self.time.wait_for_next_frame(false);
        }
    }
}
