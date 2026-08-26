//! A single-threaded queue for [`super::app`] events.
//!
//! `EventDispatcher` is a cheap clonable handle that scenes and window handles
//! keep; `EventQueue` is drained once per iteration by the main loop. `Rc` and
//! `RefCell` rather than a channel because the whole thing is main-thread only
//! — SDL requires event handling on the thread that initialised video.

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct EventDispatcher<T>(Rc<RefCell<Vec<T>>>);

impl<T> Clone for EventDispatcher<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> EventDispatcher<T> {
    pub fn dispatch(&self, event: T) {
        self.0.borrow_mut().push(event);
    }
}

#[derive(Default)]
pub struct EventQueue<T> {
    dispatcher: EventDispatcher<T>,
}

impl<T> EventQueue<T> {
    pub fn dispatcher(&self) -> EventDispatcher<T> {
        self.dispatcher.clone()
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.dispatcher.0.take()
    }
}
