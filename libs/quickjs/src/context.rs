use core::marker::PhantomData;
use core::ptr::NonNull;

use alloc::ffi::CString;
use alloc::string::String;

use quickjs_sys::JS_EVAL_TYPE_GLOBAL;
use quickjs_sys::JS_Eval;
use quickjs_sys::JS_FreeContext;
use quickjs_sys::JS_GetException;
use quickjs_sys::JS_GetGlobalObject;
use quickjs_sys::JS_IsException;
use quickjs_sys::JS_IsUninitialized;
use quickjs_sys::JS_NULL;
use quickjs_sys::JS_NewBool;
use quickjs_sys::JS_NewContext;
use quickjs_sys::JS_NewFloat64;
use quickjs_sys::JS_NewInt32;
use quickjs_sys::JS_NewStringLen;
use quickjs_sys::JS_UNDEFINED;
use quickjs_sys::JSContext;
use quickjs_sys::JSValue;

use crate::Error;
use crate::Exception;
use crate::Runtime;
use crate::Value;

pub struct Context<'rt> {
    raw: NonNull<JSContext>,
    _rt: PhantomData<&'rt Runtime>,
}

impl<'rt> Context<'rt> {
    pub fn new(rt: &'rt Runtime) -> Result<Self, Error> {
        let raw = NonNull::new(unsafe { JS_NewContext(rt.as_ptr()) }).ok_or(Error::OutOfMemory)?;

        Ok(Self {
            raw,
            _rt: PhantomData,
        })
    }

    pub fn eval(&self, src: &str, filename: &str) -> Result<Value<'_>, Error> {
        self.eval_with(src, filename, JS_EVAL_TYPE_GLOBAL)
    }

    pub fn eval_with(&self, src: &str, filename: &str, flags: u32) -> Result<Value<'_>, Error> {
        let src = CString::new(src).map_err(|_| Error::InteriorNul)?;
        let filename = CString::new(filename).map_err(|_| Error::InteriorNul)?;

        let raw = unsafe {
            JS_Eval(
                self.as_ptr(),
                src.as_ptr(),
                src.as_bytes().len(),
                filename.as_ptr(),
                flags as i32,
            )
        };

        unsafe { self.wrap(raw) }
    }

    pub fn global(&self) -> Value<'_> {
        unsafe { Value::from_raw(self, JS_GetGlobalObject(self.as_ptr())) }
    }

    pub fn undefined(&self) -> Value<'_> {
        unsafe { Value::from_raw(self, JS_UNDEFINED) }
    }

    pub fn null(&self) -> Value<'_> {
        unsafe { Value::from_raw(self, JS_NULL) }
    }

    pub fn bool(&self, v: bool) -> Value<'_> {
        unsafe { Value::from_raw(self, JS_NewBool(self.as_ptr(), v)) }
    }

    pub fn i32(&self, v: i32) -> Value<'_> {
        unsafe { Value::from_raw(self, JS_NewInt32(self.as_ptr(), v)) }
    }

    pub fn f64(&self, v: f64) -> Value<'_> {
        unsafe { Value::from_raw(self, JS_NewFloat64(self.as_ptr(), v)) }
    }

    pub fn string(&self, v: &str) -> Result<Value<'_>, Error> {
        let raw = unsafe { JS_NewStringLen(self.as_ptr(), v.as_ptr().cast(), v.len()) };

        unsafe { self.wrap(raw) }
    }

    pub unsafe fn wrap(&self, raw: JSValue) -> Result<Value<'_>, Error> {
        if unsafe { JS_IsException(raw) } {
            Err(self.take_exception())
        } else {
            Ok(unsafe { Value::from_raw(self, raw) })
        }
    }

    pub fn take_exception(&self) -> Error {
        take_exception(self.raw)
    }

    pub fn as_ptr(&self) -> *mut JSContext {
        self.raw.as_ptr()
    }
}

pub(crate) fn take_exception(ctx: NonNull<JSContext>) -> Error {
    let exc = unsafe { Value::from_parts(ctx, JS_GetException(ctx.as_ptr())) };

    if unsafe { JS_IsUninitialized(exc.as_raw()) } {
        return Error::Exception(Exception {
            message: String::from("unknown error"),
            stack: None,
        });
    }

    let message = exc
        .to_string()
        .unwrap_or_else(|_| String::from("<unprintable exception>"));

    let stack = exc
        .is_object()
        .then(|| exc.get("stack").ok())
        .flatten()
        .filter(|s| s.is_string())
        .and_then(|s| s.to_string().ok());

    Error::Exception(Exception { message, stack })
}

impl Drop for Context<'_> {
    fn drop(&mut self) {
        unsafe { JS_FreeContext(self.as_ptr()) };
    }
}
