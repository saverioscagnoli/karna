//! Lending an engine borrow to JavaScript for the length of one callback.
//!
//! The engine hands each `Scene` method short-lived borrows — `&mut
//! WindowHandle`, `&Input`, `&mut Draw<'w>`. JavaScript objects, on the other
//! hand, live in a garbage-collected heap and cannot be tied to a Rust
//! lifetime: `rquickjs` requires every class to be `'static`.
//!
//! A [`Slot`] bridges the two. The JS-side object holds a clone of the slot and
//! nothing else; the Rust side [`lend`](Slot::lend)s the real pointer just
//! before calling into JS and the returned [`Lease`] clears it on the way out.
//! A script that stashes `ctx.window` in a global and touches it on the next
//! frame therefore gets a clean `TypeError`, not a dangling read.
//!
//! # Invariant
//!
//! [`Slot::borrow`] and [`Slot::borrow_mut`] hand out references whose lifetime
//! Rust cannot check. Every binding must use one for a single engine call and
//! drop it before returning — never store one, never hold two at once, and
//! never call back into JavaScript while one is live.

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

use rquickjs::Ctx;
use rquickjs::Exception;
use rquickjs::Result;

struct Inner<T> {
    ptr: Cell<*mut T>,
    mutable: Cell<bool>,
}

/// A shared, currently-empty-or-not place for a borrow of `T`.
pub struct Slot<T> {
    inner: Rc<Inner<T>>,
    /// What to call this in an error message, e.g. `"assets"`.
    name: &'static str,
}

impl<T> Clone for Slot<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            name: self.name,
        }
    }
}

impl<T> Slot<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            inner: Rc::new(Inner {
                ptr: Cell::new(ptr::null_mut()),
                mutable: Cell::new(false),
            }),
            name,
        }
    }

    /// Lends `value` for reading only, until the returned lease is dropped.
    pub fn lend<'a>(&self, value: &'a T) -> Lease<'a, T> {
        self.install(value as *const T as *mut T, false)
    }

    /// Lends `value` for reading and writing, until the lease is dropped.
    pub fn lend_mut<'a>(&self, value: &'a mut T) -> Lease<'a, T> {
        self.install(value as *mut T, true)
    }

    fn install<'a>(&self, ptr: *mut T, mutable: bool) -> Lease<'a, T> {
        self.inner.ptr.set(ptr);
        self.inner.mutable.set(mutable);

        Lease {
            inner: self.inner.clone(),
            _lend: PhantomData,
        }
    }

    /// The lent value, or a JS exception if nothing is lent right now.
    pub fn borrow(&self, ctx: &Ctx<'_>) -> Result<&T> {
        let ptr = self.inner.ptr.get();

        if ptr.is_null() {
            return Err(self.expired(ctx));
        }

        // SAFETY: non-null means a lease is live, so the pointer still refers
        // to the borrow that created it. See the module invariant.
        Ok(unsafe { &*ptr })
    }

    /// As [`borrow`](Self::borrow), but fails when the value was lent read-only.
    ///
    /// `WindowHandle` is the case that matters: the engine passes it as `&mut`
    /// to `load`/`update` but only as `&` to `draw`, so mutating it from a
    /// script's `draw` is a script bug that should say so.
    // Handing out `&mut` from `&self` is the whole point: the slot is shared
    // with a JS object it cannot borrow-check against, and the lease is what
    // bounds the reference instead. See the module invariant.
    #[allow(clippy::mut_from_ref)]
    pub fn borrow_mut(&self, ctx: &Ctx<'_>) -> Result<&mut T> {
        let ptr = self.inner.ptr.get();

        if ptr.is_null() {
            return Err(self.expired(ctx));
        }

        if !self.inner.mutable.get() {
            return Err(Exception::throw_type(
                ctx,
                &format!("{} is read-only in this callback", self.name),
            ));
        }

        // SAFETY: as `borrow`, plus `mutable` says this came from `lend_mut`,
        // so no shared reference to the same value is outstanding.
        Ok(unsafe { &mut *ptr })
    }

    fn expired(&self, ctx: &Ctx<'_>) -> rquickjs::Error {
        Exception::throw_type(
            ctx,
            &format!(
                "{} is not available here -- it only lives for the callback it \
                 was passed to, and cannot be stored",
                self.name
            ),
        )
    }
}

/// Clears the slot it came from when dropped, including while unwinding.
pub struct Lease<'a, T> {
    inner: Rc<Inner<T>>,
    _lend: PhantomData<&'a mut T>,
}

impl<T> Drop for Lease<'_, T> {
    fn drop(&mut self) {
        self.inner.ptr.set(ptr::null_mut());
        self.inner.mutable.set(false);
    }
}
