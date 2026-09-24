use core::ffi::c_int;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::vec::Vec;

use quickjs_sys::JS_GetOpaque;
use quickjs_sys::JS_GetRuntime;
use quickjs_sys::JS_ThrowOutOfMemory;
use quickjs_sys::JS_ThrowPlainError;
use quickjs_sys::JS_ThrowTypeError;
use quickjs_sys::JSContext;
use quickjs_sys::JSValue;

use crate::Context;
use crate::Error;
use crate::FromJs;
use crate::IntoJs;
use crate::Value;
use crate::runtime::state;

/// A Rust closure callable from JS: `(ctx, this, args) -> result`.
pub type RawFn =
    dyn for<'c, 'rt> Fn(&'c Context<'rt>, &Value<'c>, &[Value<'c>]) -> Result<Value<'c>, Error>;

/// Called by QuickJS for every function made with [`Context::function_raw`].
/// `data[0]` is the object owning the boxed closure.
pub(crate) unsafe extern "C" fn trampoline(
    ctx: *mut JSContext,
    this: JSValue,
    argc: c_int,
    argv: *mut JSValue,
    _magic: c_int,
    data: *mut JSValue,
) -> JSValue {
    let raw = unsafe { NonNull::new_unchecked(ctx) };

    // Borrowed, not owned: dropping it would free the context.
    let ctx = ManuallyDrop::new(unsafe { Context::from_ptr(raw) });

    let id = unsafe { state(JS_GetRuntime(raw.as_ptr())) }
        .closure_class
        .get();
    let f = unsafe { &*JS_GetOpaque(*data, id).cast::<Box<RawFn>>() };

    // `this` and `argv` are borrowed from the caller; the wrappers take their
    // own reference so they can be dropped normally.
    let this = ctx.dup(this);
    let args = (0..argc.max(0) as usize)
        .map(|i| ctx.dup(unsafe { *argv.add(i) }))
        .collect::<Vec<_>>();

    match f(&ctx, &this, &args) {
        Ok(v) => v.into_raw(),
        Err(e) => unsafe { throw(raw.as_ptr(), e) },
    }
}

/// Raise `e` as a JS exception and return the `JS_EXCEPTION` marker.
///
/// # Safety
/// `ctx` must be a live context.
pub(crate) unsafe fn throw(ctx: *mut JSContext, e: Error) -> JSValue {
    let message = |s: &str| CString::new(s.replace('\0', "")).unwrap_or_default();

    unsafe {
        match e {
            Error::OutOfMemory => JS_ThrowOutOfMemory(ctx),
            Error::Type(msg) => JS_ThrowTypeError(ctx, c"%s".as_ptr(), message(&msg).as_ptr()),
            Error::InteriorNul => JS_ThrowTypeError(
                ctx,
                c"%s".as_ptr(),
                c"string contains an interior nul byte".as_ptr(),
            ),
            Error::Exception(e) => {
                JS_ThrowPlainError(ctx, c"%s".as_ptr(), message(&e.message).as_ptr())
            }
            Error::Custom(msg) => JS_ThrowPlainError(ctx, c"%s".as_ptr(), message(&msg).as_ptr()),
        }
    }
}

/// A Rust function whose arguments and return value convert to and from JS
/// automatically. Implemented for closures of up to eight arguments.
///
/// ```ignore
/// ctx.function("add", |a: f64, b: f64| a + b)?;
/// ```
pub trait IntoFunction<Args>: 'static {
    const LENGTH: i32;

    fn call<'c>(&self, ctx: &'c Context<'_>, args: &[Value<'c>]) -> Result<Value<'c>, Error>;
}

macro_rules! impl_into_function {
    ($len:literal $(, $arg:ident)*) => {
        impl<F, R, $($arg,)*> IntoFunction<($($arg,)*)> for F
        where
            F: Fn($($arg),*) -> R + 'static,
            R: IntoJs,
            $($arg: FromJs,)*
        {
            const LENGTH: i32 = $len;

            #[allow(non_snake_case, unused_variables, unused_mut)]
            fn call<'c>(
                &self,
                ctx: &'c Context<'_>,
                args: &[Value<'c>],
            ) -> Result<Value<'c>, Error> {
                let undefined = ctx.undefined();
                let mut args = args.iter().chain(core::iter::repeat(&undefined)).enumerate();

                $(
                    let (i, v) = args.next().unwrap();
                    let $arg = $arg::from_js(v).map_err(|e| match e {
                        Error::Type(msg) => Error::Type(alloc::format!("argument {}: {msg}", i + 1)),
                        e => e,
                    })?;
                )*

                (self)($($arg),*).into_js(ctx)
            }
        }
    };
}

impl_into_function!(0);
impl_into_function!(1, A);
impl_into_function!(2, A, B);
impl_into_function!(3, A, B, C);
impl_into_function!(4, A, B, C, D);
impl_into_function!(5, A, B, C, D, E);
impl_into_function!(6, A, B, C, D, E, G);
impl_into_function!(7, A, B, C, D, E, G, H);
impl_into_function!(8, A, B, C, D, E, G, H, I);
