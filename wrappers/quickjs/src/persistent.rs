use core::marker::PhantomData;

use quickjs_sys::JS_DupValue;
use quickjs_sys::JS_FreeValueRT;
use quickjs_sys::JS_GetRuntime;
use quickjs_sys::JSRuntime;
use quickjs_sys::JSValue;

use crate::Context;
use crate::Runtime;
use crate::Value;

/// An owned reference to a JS value that is only tied to its runtime, not to
/// a context borrow. Made by [`Context::persist`].
pub struct Persistent<'rt> {
    rt: *mut JSRuntime,
    raw: JSValue,
    _rt: PhantomData<&'rt Runtime>,
}

impl<'rt> Persistent<'rt> {
    /// # Safety
    /// `raw` must be an owned reference to a value of runtime `rt`.
    pub(crate) unsafe fn new(rt: *mut JSRuntime, raw: JSValue) -> Self {
        Self {
            rt,
            raw,
            _rt: PhantomData,
        }
    }

    /// A new reference to the value.
    pub fn get<'c>(&self, ctx: &'c Context<'rt>) -> Value<'c> {
        assert_eq!(
            unsafe { JS_GetRuntime(ctx.as_ptr()) },
            self.rt,
            "values from different quickjs runtimes cannot be mixed",
        );

        unsafe { Value::from_raw(ctx, JS_DupValue(ctx.as_ptr(), self.raw)) }
    }
}

impl Drop for Persistent<'_> {
    fn drop(&mut self) {
        unsafe { JS_FreeValueRT(self.rt, self.raw) };
    }
}
