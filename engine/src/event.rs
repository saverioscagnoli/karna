use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use crate::window::WindowCommandRequest;
use crate::window::time::TimeCommandRequest;

pub type SdlEvent = sdl3::event::Event;
pub type SdlWindowEvent = sdl3::event::WindowEvent;

#[derive(Debug)]
pub enum AppEvent {
    Time(TimeCommandRequest),
    Window(WindowCommandRequest),
}

pub struct EventDispatcher<E>(Rc<RefCell<Vec<E>>>);

impl<E> Clone for EventDispatcher<E> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<E> EventDispatcher<E> {
    pub fn send(&self, event: E) {
        self.0.borrow_mut().push(event)
    }
}

pub struct EventQueue<E> {
    write: EventDispatcher<E>,
    read: Vec<E>,
}

impl<E> EventQueue<E> {
    pub fn new() -> Self {
        Self {
            write: EventDispatcher(Rc::new(RefCell::new(Vec::new()))),
            read: Vec::new(),
        }
    }

    pub fn dispatcher(&self) -> EventDispatcher<E> {
        self.write.clone()
    }

    pub fn take(&mut self) -> Vec<E> {
        self.swap();
        mem::take(&mut self.read)
    }

    pub fn restore(&mut self, buffer: Vec<E>) {
        self.read = buffer;
    }

    pub fn swap(&mut self) {
        self.read.clear();
        mem::swap(&mut self.read, &mut *self.write.0.borrow_mut());
    }
}
