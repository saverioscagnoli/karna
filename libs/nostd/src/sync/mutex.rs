use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ops::DerefMut;
use core::ptr;
use core::time::Duration;

use sdl3_sys::SDL_CreateSemaphore;
use sdl3_sys::SDL_DestroySemaphore;
use sdl3_sys::SDL_Semaphore;
use sdl3_sys::SDL_SignalSemaphore;
use sdl3_sys::SDL_TryWaitSemaphore;
use sdl3_sys::SDL_WaitSemaphore;
use sdl3_sys::SDL_WaitSemaphoreTimeout;

pub struct Mutex<T: ?Sized> {
    sem: *mut SDL_Semaphore,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized> Send for Mutex<T> {}
unsafe impl<T: ?Sized> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self::try_new(value).unwrap_or_else(|_| panic!("Failed to create mutex"))
    }

    pub fn try_new(value: T) -> Result<Self, T> {
        let sem = unsafe { SDL_CreateSemaphore(1) };

        if sem.is_null() {
            return Err(value);
        }

        Ok(Self {
            sem,
            data: UnsafeCell::new(value),
        })
    }

    pub fn into_inner(self) -> T {
        let this = ManuallyDrop::new(self);
        let value = unsafe { ptr::read(this.data.get()) };

        unsafe { SDL_DestroySemaphore(this.sem) };

        value
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<'_, T> {
        unsafe { SDL_WaitSemaphore(self.sem) };

        MutexGuard {
            lock: self,
            _not_send: PhantomData,
        }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if unsafe { SDL_TryWaitSemaphore(self.sem) } {
            Some(MutexGuard {
                lock: self,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn lock_timeout(&self, timeout: Duration) -> Option<MutexGuard<'_, T>> {
        if unsafe { SDL_WaitSemaphoreTimeout(self.sem, timeout.as_millis() as i32) } {
            Some(MutexGuard {
                lock: self,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    unsafe fn unlock(&self) {
        unsafe { SDL_SignalSemaphore(self.sem) };
    }
}

impl<T: ?Sized> Drop for Mutex<T> {
    fn drop(&mut self) {
        unsafe { SDL_DestroySemaphore(self.sem) };
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never block inside Debug — a locked mutex would hang the formatter.
        match self.try_lock() {
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
            None => f
                .debug_struct("Mutex")
                .field("data", &format_args!("<locked>"))
                .finish(),
        }
    }
}

#[must_use = "the lock is released as soon as the guard is dropped"]
pub struct MutexGuard<'a, T: ?Sized> {
    lock: &'a Mutex<T>,
    _not_send: PhantomData<*const ()>,
}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Safety: the guard's existence is the proof that we hold the lock.
        unsafe { self.lock.unlock() };
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}
