use core::fmt;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use sdl3_sys::SDL_CreateSemaphore;
use sdl3_sys::SDL_DestroySemaphore;
use sdl3_sys::SDL_Semaphore;
use sdl3_sys::SDL_SignalSemaphore;
use sdl3_sys::SDL_TryWaitSemaphore;
use sdl3_sys::SDL_WaitSemaphore;
use sdl3_sys::SDL_WaitSemaphoreTimeout;

use crate::sync::Mutex;

struct Shared<T> {
    queue: Mutex<VecDeque<T>>,
    capacity: usize,
    slots: *mut SDL_Semaphore,
    items: *mut SDL_Semaphore,
    closed: AtomicBool,
    senders: AtomicUsize,
    receivers: AtomicUsize,
}

unsafe impl<T: Send> Send for Shared<T> {}
unsafe impl<T: Send> Sync for Shared<T> {}

impl<T> Shared<T> {
    #[inline]
    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }

    fn close(&self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            unsafe {
                SDL_SignalSemaphore(self.items);
                SDL_SignalSemaphore(self.slots);
            }
        }
    }
}

impl<T> Drop for Shared<T> {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroySemaphore(self.slots);
            SDL_DestroySemaphore(self.items);
        }
    }
}

pub fn channel<T>(capacity: usize) -> Result<(Sender<T>, Receiver<T>), CreateError> {
    assert!(capacity > 0, "channel capacity must be non-zero");
    let capacity_u32 = u32::try_from(capacity).map_err(|_| CreateError)?;

    let slots = unsafe { SDL_CreateSemaphore(capacity_u32) };

    if slots.is_null() {
        return Err(CreateError);
    }

    let items = unsafe { SDL_CreateSemaphore(0) };

    if items.is_null() {
        unsafe { SDL_DestroySemaphore(slots) };
        return Err(CreateError);
    }

    let queue = match Mutex::try_new(VecDeque::with_capacity(capacity)) {
        Ok(queue) => queue,
        Err(_) => {
            unsafe {
                SDL_DestroySemaphore(slots);
                SDL_DestroySemaphore(items);
            }
            return Err(CreateError);
        }
    };

    let shared = Arc::new(Shared {
        queue,
        capacity,
        slots,
        items,
        closed: AtomicBool::new(false),
        senders: AtomicUsize::new(1),
        receivers: AtomicUsize::new(1),
    });

    Ok((
        Sender {
            shared: Arc::clone(&shared),
        },
        Receiver { shared },
    ))
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if self.shared.is_closed() {
            return Err(SendError(value));
        }

        unsafe { SDL_WaitSemaphore(self.shared.slots) };

        self.finish_send(value)
    }

    pub fn send_timeout(&self, value: T, timeout_ms: i32) -> Result<(), SendTimeoutError<T>> {
        if self.shared.is_closed() {
            return Err(SendTimeoutError::Closed(value));
        }

        if !unsafe { SDL_WaitSemaphoreTimeout(self.shared.slots, timeout_ms) } {
            return Err(SendTimeoutError::Timeout(value));
        }

        self.finish_send(value)
            .map_err(|SendError(v)| SendTimeoutError::Closed(v))
    }

    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        if self.shared.is_closed() {
            return Err(TrySendError::Closed(value));
        }

        if !unsafe { SDL_TryWaitSemaphore(self.shared.slots) } {
            return Err(TrySendError::Full(value));
        }

        self.finish_send(value)
            .map_err(|SendError(v)| TrySendError::Closed(v))
    }

    fn finish_send(&self, value: T) -> Result<(), SendError<T>> {
        if self.shared.is_closed() {
            unsafe { SDL_SignalSemaphore(self.shared.slots) };
            return Err(SendError(value));
        }

        {
            let mut queue = self.shared.queue.lock();
            debug_assert!(queue.len() < self.shared.capacity, "slot accounting broken");
            queue.push_back(value);
        }

        unsafe { SDL_SignalSemaphore(self.shared.items) };
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.shared.is_closed()
    }

    pub fn len(&self) -> usize {
        self.shared.queue.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.shared.queue.lock().is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.shared.capacity
    }

    pub fn close(&self) {
        self.shared.close();
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.shared.senders.fetch_add(1, Ordering::Relaxed);
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.shared.senders.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.shared.close();
        }
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender")
            .field("capacity", &self.capacity())
            .field("closed", &self.is_closed())
            .finish()
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        unsafe { SDL_WaitSemaphore(self.shared.items) };
        self.finish_recv().ok_or(RecvError)
    }

    pub fn recv_timeout(&self, timeout_ms: i32) -> Result<T, RecvTimeoutError> {
        if !unsafe { SDL_WaitSemaphoreTimeout(self.shared.items, timeout_ms) } {
            return Err(RecvTimeoutError::Timeout);
        }

        self.finish_recv().ok_or(RecvTimeoutError::Closed)
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        if !unsafe { SDL_TryWaitSemaphore(self.shared.items) } {
            return Err(if self.shared.is_closed() {
                TryRecvError::Closed
            } else {
                TryRecvError::Empty
            });
        }

        self.finish_recv().ok_or(TryRecvError::Closed)
    }

    fn finish_recv(&self) -> Option<T> {
        let popped = self.shared.queue.lock().pop_front();

        match popped {
            Some(value) => {
                unsafe { SDL_SignalSemaphore(self.shared.slots) };
                Some(value)
            }
            None => {
                debug_assert!(self.shared.is_closed());
                unsafe { SDL_SignalSemaphore(self.shared.items) };
                None
            }
        }
    }

    pub fn is_closed(&self) -> bool {
        self.shared.is_closed()
    }

    pub fn len(&self) -> usize {
        self.shared.queue.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.shared.queue.lock().is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.shared.capacity
    }

    pub fn close(&self) {
        self.shared.close();
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.shared.receivers.fetch_add(1, Ordering::Relaxed);
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        if self.shared.receivers.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.shared.close();
        }
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver")
            .field("capacity", &self.capacity())
            .field("closed", &self.is_closed())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateError;

impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to create channel primitives")
    }
}

pub struct SendError<T>(pub T);

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SendError(..)")
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("channel is closed")
    }
}

pub enum TrySendError<T> {
    Full(T),
    Closed(T),
}

impl<T> TrySendError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::Full(v) | Self::Closed(v) => v,
        }
    }
}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(_) => f.write_str("Full(..)"),
            Self::Closed(_) => f.write_str("Closed(..)"),
        }
    }
}

pub enum SendTimeoutError<T> {
    Timeout(T),
    Closed(T),
}

impl<T> SendTimeoutError<T> {
    pub fn into_inner(self) -> T {
        match self {
            Self::Timeout(v) | Self::Closed(v) => v,
        }
    }
}

impl<T> fmt::Debug for SendTimeoutError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(_) => f.write_str("Timeout(..)"),
            Self::Closed(_) => f.write_str("Closed(..)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecvError;

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("channel is closed")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvTimeoutError {
    Timeout,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    Empty,
    Closed,
}
