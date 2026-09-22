use core::fmt;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;
use core::slice;

use alloc::ffi::CString;
use alloc::string::String;

use quickjs_sys::JS_DupValue;
use quickjs_sys::JS_FreeCString;
use quickjs_sys::JS_FreeValue;
use quickjs_sys::JS_GetPropertyStr;
use quickjs_sys::JS_GetRuntime;
use quickjs_sys::JS_IsBool;
use quickjs_sys::JS_IsException;
use quickjs_sys::JS_IsFunction;
use quickjs_sys::JS_IsNull;
use quickjs_sys::JS_IsNumber;
use quickjs_sys::JS_IsObject;
use quickjs_sys::JS_IsString;
use quickjs_sys::JS_IsUndefined;
use quickjs_sys::JS_SetPropertyStr;
use quickjs_sys::JS_TAG_BOOL;
use quickjs_sys::JS_TAG_FLOAT64;
use quickjs_sys::JS_TAG_INT;
use quickjs_sys::JS_ToCStringLen;
use quickjs_sys::JS_VALUE_GET_BOOL;
use quickjs_sys::JS_VALUE_GET_FLOAT64;
use quickjs_sys::JS_VALUE_GET_INT;
use quickjs_sys::JS_VALUE_GET_TAG;
use quickjs_sys::JSContext;
use quickjs_sys::JSValue;

use crate::Context;
use crate::Error;
use crate::context::take_exception;

pub struct Value<'ctx> {
    ctx: NonNull<JSContext>,
    raw: JSValue,
    _ctx: PhantomData<&'ctx Context<'ctx>>,
}

impl<'ctx> Value<'ctx> {
    pub unsafe fn from_raw(ctx: &'ctx Context<'_>, raw: JSValue) -> Self {
        unsafe { Self::from_parts(NonNull::new_unchecked(ctx.as_ptr()), raw) }
    }

    pub(crate) unsafe fn from_parts(ctx: NonNull<JSContext>, raw: JSValue) -> Self {
        Self {
            ctx,
            raw,
            _ctx: PhantomData,
        }
    }

    pub fn as_raw(&self) -> JSValue {
        self.raw
    }

    pub fn into_raw(self) -> JSValue {
        ManuallyDrop::new(self).raw
    }

    pub fn ctx_ptr(&self) -> *mut JSContext {
        self.ctx.as_ptr()
    }

    pub fn tag(&self) -> i32 {
        JS_VALUE_GET_TAG(self.raw)
    }

    pub fn is_undefined(&self) -> bool {
        unsafe { JS_IsUndefined(self.raw) }
    }

    pub fn is_null(&self) -> bool {
        unsafe { JS_IsNull(self.raw) }
    }

    pub fn is_bool(&self) -> bool {
        unsafe { JS_IsBool(self.raw) }
    }

    pub fn is_number(&self) -> bool {
        unsafe { JS_IsNumber(self.raw) }
    }

    pub fn is_string(&self) -> bool {
        unsafe { JS_IsString(self.raw) }
    }

    pub fn is_object(&self) -> bool {
        unsafe { JS_IsObject(self.raw) }
    }

    pub fn is_function(&self) -> bool {
        unsafe { JS_IsFunction(self.ctx_ptr(), self.raw) }
    }

    pub fn is_exception(&self) -> bool {
        unsafe { JS_IsException(self.raw) }
    }

    pub fn as_bool(&self) -> Option<bool> {
        (self.tag() == JS_TAG_BOOL).then(|| unsafe { JS_VALUE_GET_BOOL(self.raw) } != 0)
    }

    pub fn as_i32(&self) -> Option<i32> {
        (self.tag() == JS_TAG_INT).then(|| unsafe { JS_VALUE_GET_INT(self.raw) })
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.tag() {
            JS_TAG_INT => Some(unsafe { JS_VALUE_GET_INT(self.raw) } as f64),
            JS_TAG_FLOAT64 => Some(unsafe { JS_VALUE_GET_FLOAT64(self.raw) }),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Result<String, Error> {
        let mut len = 0;
        let ptr = unsafe { JS_ToCStringLen(self.ctx_ptr(), &mut len, self.raw) };

        if ptr.is_null() {
            return Err(self.context_error());
        }

        let bytes = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len) };
        let s = String::from_utf8_lossy(bytes).into_owned();

        unsafe { JS_FreeCString(self.ctx_ptr(), ptr) };

        Ok(s)
    }

    pub fn get(&self, key: &str) -> Result<Value<'ctx>, Error> {
        let key = CString::new(key).map_err(|_| Error::InteriorNul)?;
        let raw = unsafe { JS_GetPropertyStr(self.ctx_ptr(), self.raw, key.as_ptr()) };

        if unsafe { JS_IsException(raw) } {
            return Err(self.context_error());
        }

        Ok(self.sibling(raw))
    }

    pub fn set(&self, key: &str, value: Value<'_>) -> Result<(), Error> {
        assert_eq!(
            unsafe { JS_GetRuntime(self.ctx_ptr()) },
            unsafe { JS_GetRuntime(value.ctx_ptr()) },
            "values from different quickjs runtimes cannot be mixed",
        );

        let key = CString::new(key).map_err(|_| Error::InteriorNul)?;
        let ret =
            unsafe { JS_SetPropertyStr(self.ctx_ptr(), self.raw, key.as_ptr(), value.into_raw()) };

        if ret < 0 {
            return Err(self.context_error());
        }

        Ok(())
    }

    fn sibling(&self, raw: JSValue) -> Value<'ctx> {
        unsafe { Value::from_parts(self.ctx, raw) }
    }

    fn context_error(&self) -> Error {
        take_exception(self.ctx)
    }
}

impl Clone for Value<'_> {
    fn clone(&self) -> Self {
        self.sibling(unsafe { JS_DupValue(self.ctx_ptr(), self.raw) })
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        unsafe { JS_FreeValue(self.ctx_ptr(), self.raw) };
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_string() {
            Ok(s) => write!(f, "Value({s:?})"),
            Err(_) => write!(f, "Value(<tag {}>)", self.tag()),
        }
    }
}
