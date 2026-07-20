pub mod context;
pub mod input;
pub mod scene;
pub mod state;
pub mod time;

use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use logging::error;
use sdl3::event::Event;
use sdl3::event::EventSender;

pub use sdl3::video::Window as SdlWindow;

#[derive(Debug)]
pub enum MainThreadRequest {
    SetWindowTitle(String),
    SetWindowSize(math::Size<u32>),
}

pub struct WindowRequest {
    pub window_id: u32,
    pub request: MainThreadRequest,
}

pub struct WindowHandle {
    pub window: SdlWindow,
    pub event_sender: Sender<Event>,
    pub shutdown: Sender<()>,
    pub thread: JoinHandle<()>,
}

pub struct Window {
    id: u32,
    title: String,
    size: math::Size<u32>,
    proxy: EventSender,
}

impl Window {
    pub(crate) fn new(id: u32, title: String, size: math::Size<u32>, proxy: EventSender) -> Self {
        Self {
            id,
            title,
            size,
            proxy,
        }
    }

    fn push(&self, req: MainThreadRequest) {
        if let Err(e) = self.proxy.push_custom_event(WindowRequest {
            window_id: self.id,
            request: req,
        }) {
            error!("Failed to send window request: {}", e);
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let title: &str = title.as_ref();

        self.title = title.to_owned();
        self.push(MainThreadRequest::SetWindowTitle(title.to_owned()));
    }

    pub fn size(&self) -> &math::Size<u32> {
        &self.size
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size: math::Size<u32> = size.into();

        self.size = size;
        self.push(MainThreadRequest::SetWindowSize(size));
    }

    pub fn set_width(&mut self, width: u32) {
        self.size.width = width;
        self.push(MainThreadRequest::SetWindowSize(self.size))
    }

    pub fn set_height(&mut self, height: u32) {
        self.size.height = height;
        self.push(MainThreadRequest::SetWindowSize(self.size))
    }
}
