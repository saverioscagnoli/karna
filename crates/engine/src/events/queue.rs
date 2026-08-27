use std::cell::RefCell;
use std::rc::Rc;

pub struct EventDispatcher<T>(Rc<RefCell<Vec<T>>>);

impl<T> Default for EventDispatcher<T> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }
}

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

pub struct EventQueue<T> {
    dispatcher: EventDispatcher<T>,
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self {
            dispatcher: EventDispatcher::default(),
        }
    }
}

impl<T> EventQueue<T> {
    pub fn dispatcher(&self) -> EventDispatcher<T> {
        self.dispatcher.clone()
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.dispatcher.0.take()
    }
}
