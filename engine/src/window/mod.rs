pub mod context;
pub mod input;
pub mod scene;
pub mod state;
pub mod time;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use logging::error;
use parking_lot::Mutex;
use sdl3::event::Event;
use sdl3::event::EventSender;

use crate::render::FrameSubmission;
use crate::render::RenderLayers;
use crate::render::Renderer;

pub use sdl3::video::Window as SdlWindow;

/// Latest-wins slot a window's logic thread drops finished frames into.
/// The main thread takes them out when it receives [`MainThreadRequest::FrameReady`].
pub type FrameSlot = Arc<Mutex<Option<FrameSubmission>>>;

/// True while a [`MainThreadRequest::Present`] event for this window is in the
/// SDL event queue. Bounds the queue to one Present per window: the logic
/// thread can outrun the vsync-blocked main thread by orders of magnitude, and
/// unthrottled Present events fill SDL's 65535-event queue in seconds.
pub type PresentPending = Arc<AtomicBool>;

#[derive(Debug)]
pub enum MainThreadRequest {
    SetWindowTitle(String),
    SetWindowSize(math::Size<u32>),
    /// A finished frame is waiting in the window's [`FrameSlot`].
    Present,
}

pub struct WindowRequest {
    pub window_id: u32,
    pub request: MainThreadRequest,
}

pub struct WindowHandle {
    pub window: SdlWindow,
    pub renderer: Renderer,
    pub frame_slot: FrameSlot,
    pub present_pending: PresentPending,
    /// Returns rendered frames' geometry buffers to the logic thread for reuse.
    pub recycle: Sender<RenderLayers>,
    pub event_sender: Sender<Event>,
    pub shutdown: Sender<()>,
    pub thread: JoinHandle<()>,
}

pub struct Window {
    id: u32,
    title: String,
    size: math::Size<u32>,
    proxy: EventSender,
    present_pending: PresentPending,
}

impl Window {
    pub(crate) fn new(
        id: u32,
        title: String,
        size: math::Size<u32>,
        proxy: EventSender,
        present_pending: PresentPending,
    ) -> Self {
        Self {
            id,
            title,
            size,
            proxy,
            present_pending,
        }
    }

    fn push(&self, req: MainThreadRequest) -> bool {
        match self.proxy.push_custom_event(WindowRequest {
            window_id: self.id,
            request: req,
        }) {
            Ok(()) => true,
            Err(e) => {
                error!("Failed to send window request: {}", e);
                false
            }
        }
    }

    /// Wakes the main thread to render the frame in this window's slot.
    /// A no-op if a Present event is already queued — the frame slot is
    /// latest-wins, so the queued event will pick up the newest frame.
    pub(crate) fn present(&self) {
        if !self.present_pending.swap(true, Ordering::AcqRel)
            && !self.push(MainThreadRequest::Present)
        {
            // The event never made it into the queue; clear the flag so the
            // next frame retries instead of never presenting again.
            self.present_pending.store(false, Ordering::Release);
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

    /// Updates the cached size without requesting a resize.
    /// Used when the OS resized the window (e.g. user drag).
    pub(crate) fn sync_size(&mut self, size: math::Size<u32>) {
        self.size = size;
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
        self.push(MainThreadRequest::SetWindowSize(self.size));
    }

    pub fn set_height(&mut self, height: u32) {
        self.size.height = height;
        self.push(MainThreadRequest::SetWindowSize(self.size));
    }
}
