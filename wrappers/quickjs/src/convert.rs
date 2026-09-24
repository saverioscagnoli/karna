use alloc::format;
use alloc::string::String;

use crate::Context;
use crate::Error;
use crate::Value;

/// Conversion from a JS value into an owned Rust value.
///
/// Conversions are strict: a number parameter rejects strings and `undefined`
/// rather than coercing them. Use `Option<T>` for optional parameters.
pub trait FromJs: Sized {
    fn from_js(value: &Value<'_>) -> Result<Self, Error>;
}

/// Conversion from a Rust value into a JS value.
pub trait IntoJs {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error>;
}

fn expected(what: &str, value: &Value<'_>) -> Error {
    Error::Type(format!("expected {what}, got {}", value.type_name()))
}

impl FromJs for bool {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        value.as_bool().ok_or_else(|| expected("boolean", value))
    }
}

impl FromJs for f64 {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        value.as_f64().ok_or_else(|| expected("number", value))
    }
}

macro_rules! from_js_number {
    ($($t:ty),*) => {$(
        impl FromJs for $t {
            fn from_js(value: &Value<'_>) -> Result<Self, Error> {
                f64::from_js(value).map(|n| n as $t)
            }
        }
    )*};
}

from_js_number!(f32, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl FromJs for String {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        if !value.is_string() {
            return Err(expected("string", value));
        }

        value.to_string()
    }
}

impl<T: FromJs> FromJs for Option<T> {
    fn from_js(value: &Value<'_>) -> Result<Self, Error> {
        if value.is_undefined() || value.is_null() {
            Ok(None)
        } else {
            T::from_js(value).map(Some)
        }
    }
}

impl IntoJs for () {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        Ok(ctx.undefined())
    }
}

impl IntoJs for bool {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        Ok(ctx.bool(self))
    }
}

impl IntoJs for i32 {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        Ok(ctx.i32(self))
    }
}

impl IntoJs for f64 {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        Ok(ctx.f64(self))
    }
}

macro_rules! into_js_number {
    ($($t:ty),*) => {$(
        impl IntoJs for $t {
            fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
                // Small integers stay ints on the JS side; everything else
                // becomes a double, like any other JS number.
                Ok(match i32::try_from(self) {
                    Ok(n) => ctx.i32(n),
                    Err(_) => ctx.f64(self as f64),
                })
            }
        }
    )*};
}

into_js_number!(i8, i16, i64, isize, u8, u16, u32, u64, usize);

impl IntoJs for f32 {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        Ok(ctx.f64(self as f64))
    }
}

impl IntoJs for &str {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        ctx.string(self)
    }
}

impl IntoJs for String {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        ctx.string(&self)
    }
}

impl<T: IntoJs> IntoJs for Option<T> {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        match self {
            Some(v) => v.into_js(ctx),
            None => Ok(ctx.undefined()),
        }
    }
}

impl<T: IntoJs> IntoJs for Result<T, Error> {
    fn into_js<'c>(self, ctx: &'c Context<'_>) -> Result<Value<'c>, Error> {
        self?.into_js(ctx)
    }
}
