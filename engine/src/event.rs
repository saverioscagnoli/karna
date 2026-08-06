use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use crate::window::WindowId;

pub type SdlEvent = sdl3::event::Event;
pub type SdlWindowEvent = sdl3::event::WindowEvent;

#[derive(Debug, Clone)]
pub enum WindowEvent {
    TitleChangeRequested(Rc<str>),
    SizeChangeRequested(math::Size<u32>),
    FpsTargetChangeRequested(u32),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Window { id: WindowId, event: WindowEvent },
    TpsTargetChangeRequested(u32),
}

impl AppEvent {
    pub fn get_window_id(&self) -> Option<WindowId> {
        match self {
            Self::Window { id, .. } => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct EventDispatcher<E>(Rc<RefCell<Vec<E>>>);

impl<E> Default for EventDispatcher<E> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }
}

impl<E> Clone for EventDispatcher<E> {
    fn clone(&self) -> Self {
        EventDispatcher(Rc::clone(&self.0))
    }
}

impl<E> EventDispatcher<E> {
    pub fn send(&self, event: E) {
        self.0.borrow_mut().push(event);
    }
}

pub struct EventQueue<E> {
    dispatcher: EventDispatcher<E>,
    q: Vec<E>,
}

impl<E> EventQueue<E> {
    pub fn new() -> Self {
        Self {
            dispatcher: EventDispatcher::default(),
            q: Vec::new(),
        }
    }

    pub fn dispatcher(&self) -> EventDispatcher<E> {
        self.dispatcher.clone()
    }

    fn swap(&mut self) {
        self.q.clear();
        mem::swap(&mut self.q, &mut *self.dispatcher.0.borrow_mut());
    }

    pub fn take(&mut self) -> Vec<E> {
        self.swap();
        mem::take(&mut self.q)
    }

    pub fn restore(&mut self, q: Vec<E>) {
        self.q = q;
    }
}
