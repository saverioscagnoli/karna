use core::any::TypeId;
use core::cell::Cell;
use core::cell::RefCell;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use quickjs_sys::JS_DupValueRT;
use quickjs_sys::JS_ExecutePendingJob;
use quickjs_sys::JS_FreeRuntime;
use quickjs_sys::JS_FreeValueRT;
use quickjs_sys::JS_GetRuntime;
use quickjs_sys::JS_GetRuntimeOpaque;
use quickjs_sys::JS_NewRuntime;
use quickjs_sys::JS_RunGC;
use quickjs_sys::JS_SetHostPromiseRejectionTracker;
use quickjs_sys::JS_SetMaxStackSize;
use quickjs_sys::JS_SetMemoryLimit;
use quickjs_sys::JS_SetRuntimeOpaque;
use quickjs_sys::JS_VALUE_GET_PTR;
use quickjs_sys::JSClassID;
use quickjs_sys::JSContext;
use quickjs_sys::JSRuntime;
use quickjs_sys::JSValue;

use crate::Error;
use crate::Value;
use crate::class;
use crate::context::describe;
use crate::context::take_exception;
use crate::module::ModuleLoader;

/// Per-runtime bookkeeping, reachable from any context through the runtime's
/// opaque pointer.
pub(crate) struct RuntimeState {
    /// Class backing the objects that own boxed Rust closures.
    pub closure_class: Cell<JSClassID>,
    /// Classes registered for Rust types (see [`Context::class`]).
    ///
    /// [`Context::class`]: crate::Context::class
    pub classes: RefCell<BTreeMap<TypeId, JSClassID>>,
    /// Promises rejected with no handler attached (yet); reported by
    /// [`Runtime::run_jobs`].
    rejections: RefCell<Vec<Rejection>>,
    /// Source for `import`s (see [`Runtime::set_module_loader`]).
    pub loader: RefCell<Option<Box<ModuleLoader>>>,
}

/// Both values are owned references.
struct Rejection {
    ctx: NonNull<JSContext>,
    promise: JSValue,
    reason: JSValue,
}

pub struct Runtime {
    raw: NonNull<JSRuntime>,
}

impl Runtime {
    pub fn new() -> Result<Self, Error> {
        let raw = NonNull::new(unsafe { JS_NewRuntime() }).ok_or(Error::OutOfMemory)?;

        let state = Box::new(RuntimeState {
            closure_class: Cell::new(0),
            classes: RefCell::new(BTreeMap::new()),
            rejections: RefCell::new(Vec::new()),
            loader: RefCell::new(None),
        });

        unsafe {
            JS_SetRuntimeOpaque(raw.as_ptr(), Box::into_raw(state).cast());
            JS_SetHostPromiseRejectionTracker(
                raw.as_ptr(),
                Some(track_rejection),
                core::ptr::null_mut(),
            );
        }

        let rt = Self { raw };
        let id = unsafe { class::register_closure_class(rt.as_ptr()) }?;
        rt.state().closure_class.set(id);

        Ok(rt)
    }

    pub fn set_memory_limit(&self, bytes: usize) {
        unsafe { JS_SetMemoryLimit(self.as_ptr(), bytes) };
    }

    pub fn set_max_stack_size(&self, bytes: usize) {
        unsafe { JS_SetMaxStackSize(self.as_ptr(), bytes) };
    }

    pub fn run_gc(&self) {
        unsafe { JS_RunGC(self.as_ptr()) };
    }

    /// Run queued jobs (promise reactions, `await` continuations) until none
    /// are left, stopping at the first one that throws.
    ///
    /// Promises that ended up rejected with no handler are reported here too,
    /// as the error of the first one.
    pub fn run_jobs(&self) -> Result<(), Error> {
        unsafe { run_jobs(self.as_ptr()) }
    }

    pub fn as_ptr(&self) -> *mut JSRuntime {
        self.raw.as_ptr()
    }

    fn state(&self) -> &RuntimeState {
        unsafe { state(self.as_ptr()) }
    }
}

/// See [`Runtime::run_jobs`].
///
/// # Safety
/// `rt` must be a runtime created by [`Runtime::new`] that is still alive.
pub(crate) unsafe fn run_jobs(rt: *mut JSRuntime) -> Result<(), Error> {
    loop {
        let mut ctx = core::ptr::null_mut();

        match unsafe { JS_ExecutePendingJob(rt, &mut ctx) } {
            0 => return unsafe { take_rejection(rt) },
            n if n < 0 => {
                return Err(match NonNull::new(ctx) {
                    Some(ctx) => take_exception(ctx),
                    None => Error::OutOfMemory,
                });
            }
            _ => {}
        }
    }
}

unsafe fn take_rejection(rt: *mut JSRuntime) -> Result<(), Error> {
    let rejections = unsafe { state(rt) }.rejections.take();
    let mut first = None;

    for Rejection {
        ctx,
        promise,
        reason,
    } in rejections
    {
        unsafe { JS_FreeValueRT(rt, promise) };
        let reason = unsafe { Value::from_parts(ctx, reason) };

        if first.is_none() {
            first = Some(describe(reason));
        }
    }

    first.map_or(Ok(()), Err)
}

/// # Safety
/// `rt` must be a runtime created by [`Runtime::new`] that is still alive.
pub(crate) unsafe fn state<'a>(rt: *mut JSRuntime) -> &'a RuntimeState {
    unsafe { &*JS_GetRuntimeOpaque(rt).cast::<RuntimeState>() }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let state = unsafe { JS_GetRuntimeOpaque(self.as_ptr()) }.cast::<RuntimeState>();

        for r in unsafe { &*state }.rejections.take() {
            unsafe {
                JS_FreeValueRT(self.as_ptr(), r.promise);
                JS_FreeValueRT(self.as_ptr(), r.reason);
            }
        }

        // Freeing the runtime runs the class finalizers, which drop the Rust
        // values they own; the state has to outlive that.
        unsafe { JS_FreeRuntime(self.as_ptr()) };
        drop(unsafe { Box::from_raw(state) });
    }
}

unsafe extern "C" fn track_rejection(
    ctx: *mut JSContext,
    promise: JSValue,
    reason: JSValue,
    is_handled: bool,
    _opaque: *mut core::ffi::c_void,
) {
    let rt = unsafe { JS_GetRuntime(ctx) };
    let mut rejections = unsafe { state(rt) }.rejections.borrow_mut();

    if is_handled {
        // A handler was attached after the fact; it is no longer unhandled.
        let ptr = unsafe { JS_VALUE_GET_PTR(promise) };
        let found = rejections
            .iter()
            .position(|r| unsafe { JS_VALUE_GET_PTR(r.promise) } == ptr);

        if let Some(i) = found {
            let r = rejections.remove(i);
            unsafe {
                JS_FreeValueRT(rt, r.promise);
                JS_FreeValueRT(rt, r.reason);
            }
        }
    } else {
        rejections.push(Rejection {
            ctx: unsafe { NonNull::new_unchecked(ctx) },
            promise: unsafe { JS_DupValueRT(rt, promise) },
            reason: unsafe { JS_DupValueRT(rt, reason) },
        });
    }
}
