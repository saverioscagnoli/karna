use core::marker::PhantomData;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::String;

use quickjs_sys::JS_DefinePropertyValueStr;
use quickjs_sys::JS_DupValue;
use quickjs_sys::JS_EVAL_TYPE_GLOBAL;
use quickjs_sys::JS_Eval;
use quickjs_sys::JS_FreeContext;
use quickjs_sys::JS_GetClassProto;
use quickjs_sys::JS_GetException;
use quickjs_sys::JS_GetGlobalObject;
use quickjs_sys::JS_GetRuntime;
use quickjs_sys::JS_IsException;
use quickjs_sys::JS_IsNull;
use quickjs_sys::JS_IsUninitialized;
use quickjs_sys::JS_NULL;
use quickjs_sys::JS_NewBool;
use quickjs_sys::JS_NewCFunctionData;
use quickjs_sys::JS_NewContext;
use quickjs_sys::JS_NewFloat64;
use quickjs_sys::JS_NewInt32;
use quickjs_sys::JS_NewObject;
use quickjs_sys::JS_NewObjectClass;
use quickjs_sys::JS_NewStringLen;
use quickjs_sys::JS_PROP_CONFIGURABLE;
use quickjs_sys::JS_SetClassProto;
use quickjs_sys::JS_SetOpaque;
use quickjs_sys::JS_UNDEFINED;
use quickjs_sys::JSClassID;
use quickjs_sys::JSContext;
use quickjs_sys::JSValue;

use crate::Error;
use crate::Exception;
use crate::IntoFunction;
use crate::Persistent;
use crate::RawFn;
use crate::Runtime;
use crate::Value;
use crate::class;
use crate::function::trampoline;
use crate::runtime::state;

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

    /// Wrap a context owned by someone else. The result must not be dropped
    /// (keep it in a `ManuallyDrop`), or it frees the context.
    pub(crate) unsafe fn from_ptr(raw: NonNull<JSContext>) -> Self {
        Self {
            raw,
            _rt: PhantomData,
        }
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

    pub fn object(&self) -> Result<Value<'_>, Error> {
        unsafe { self.wrap(JS_NewObject(self.as_ptr())) }
    }

    /// A JS function calling `f`, with arguments and the return value
    /// converted through [`FromJs`](crate::FromJs) / [`IntoJs`](crate::IntoJs).
    ///
    /// ```ignore
    /// global.set("add", ctx.function("add", |a: f64, b: f64| a + b)?)?;
    /// ```
    pub fn function<Args, F>(&self, name: &str, f: F) -> Result<Value<'_>, Error>
    where
        F: IntoFunction<Args>,
    {
        self.function_raw(name, F::LENGTH, move |ctx, _this, args| f.call(ctx, args))
    }

    /// A JS function calling `f` with the raw `this` and arguments.
    /// `length` is what the function's `length` property reports.
    ///
    /// Returning `Err` throws it into JS (see [`Error`] for how each variant
    /// maps to a JS error).
    pub fn function_raw<F>(&self, name: &str, length: i32, f: F) -> Result<Value<'_>, Error>
    where
        F: for<'c, 'r> Fn(&'c Context<'r>, &Value<'c>, &[Value<'c>]) -> Result<Value<'c>, Error>
            + 'static,
    {
        let id = unsafe { state(JS_GetRuntime(self.as_ptr())) }
            .closure_class
            .get();

        // The closure lives in an object of the closure class, whose finalizer
        // drops it once the function (the only thing referencing it) is
        // collected.
        let holder = unsafe { self.wrap(JS_NewObjectClass(self.as_ptr(), id)) }?;
        let boxed: Box<Box<RawFn>> = Box::new(Box::new(f));
        unsafe { JS_SetOpaque(holder.as_raw(), Box::into_raw(boxed).cast()) };

        let mut data = [holder.as_raw()];
        let func = unsafe {
            self.wrap(JS_NewCFunctionData(
                self.as_ptr(),
                Some(trampoline),
                length,
                0,
                1,
                data.as_mut_ptr(),
            ))
        }?;

        let name = self.string(name)?;
        let ret = unsafe {
            JS_DefinePropertyValueStr(
                self.as_ptr(),
                func.as_raw(),
                c"name".as_ptr(),
                name.into_raw(),
                JS_PROP_CONFIGURABLE as i32,
            )
        };

        if ret < 0 {
            return Err(self.take_exception());
        }

        Ok(func)
    }

    /// The prototype shared by every JS object wrapping a `T` (see
    /// [`instance`](Self::instance)). Methods set on it are visible on all of
    /// them.
    pub fn class<T: 'static>(&self) -> Result<Value<'_>, Error> {
        let id = self.class_id::<T>()?;
        let proto = unsafe { Value::from_raw(self, JS_GetClassProto(self.as_ptr(), id)) };

        if !unsafe { JS_IsNull(proto.as_raw()) } {
            return Ok(proto);
        }

        let proto = self.object()?;
        unsafe {
            JS_SetClassProto(
                self.as_ptr(),
                id,
                JS_DupValue(self.as_ptr(), proto.as_raw()),
            )
        };

        Ok(proto)
    }

    /// Move `value` into a new JS object. It is dropped when the object is
    /// garbage collected; borrow it back with [`Value::opaque`].
    pub fn instance<T: 'static>(&self, value: T) -> Result<Value<'_>, Error> {
        let id = self.class_id::<T>()?;

        // Make sure the prototype exists, or the object gets a null one.
        drop(self.class::<T>()?);

        let obj = unsafe { self.wrap(JS_NewObjectClass(self.as_ptr(), id)) }?;
        unsafe { JS_SetOpaque(obj.as_raw(), Box::into_raw(Box::new(value)).cast()) };

        Ok(obj)
    }

    /// Keep `value` alive beyond the borrow of this context, e.g. to store a
    /// callback in a struct. Get it back with [`Persistent::get`].
    pub fn persist(&self, value: Value<'_>) -> Persistent<'rt> {
        unsafe { Persistent::new(JS_GetRuntime(self.as_ptr()), value.into_raw()) }
    }

    pub(crate) fn class_id<T: 'static>(&self) -> Result<JSClassID, Error> {
        unsafe { class::class_id::<T>(JS_GetRuntime(self.as_ptr())) }
    }

    /// Wrap a borrowed value, taking a new reference to it.
    pub(crate) fn dup(&self, raw: JSValue) -> Value<'_> {
        unsafe { Value::from_raw(self, JS_DupValue(self.as_ptr(), raw)) }
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

    describe(exc)
}

/// Turn a thrown JS value into an [`Error`].
pub(crate) fn describe(exc: Value<'_>) -> Error {
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
