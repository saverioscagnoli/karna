use alloc::format;
use alloc::string::String;

use crate::Error;
use crate::Lua;
use crate::Value;

pub trait FromLua: Sized {
    fn from_lua(value: &Value<'_>) -> Result<Self, Error>;
}

pub trait IntoLua {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error>;
}

fn expected(what: &str, value: &Value<'_>) -> Error {
    Error::Type(format!("expected {what}, got {}", value.type_name()))
}

impl FromLua for bool {
    fn from_lua(value: &Value<'_>) -> Result<Self, Error> {
        value.as_bool().ok_or_else(|| expected("boolean", value))
    }
}

impl FromLua for f64 {
    fn from_lua(value: &Value<'_>) -> Result<Self, Error> {
        value.as_f64().ok_or_else(|| expected("number", value))
    }
}

impl FromLua for f32 {
    fn from_lua(value: &Value<'_>) -> Result<Self, Error> {
        f64::from_lua(value).map(|n| n as f32)
    }
}

macro_rules! from_lua_integer {
    ($($t:ty),*) => {$(
        impl FromLua for $t {
            fn from_lua(value: &Value<'_>) -> Result<Self, Error> {
                match value.as_i64() {
                    Some(n) => Ok(n as $t),
                    None => f64::from_lua(value).map(|n| n as $t),
                }
            }
        }
    )*};
}

from_lua_integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl FromLua for String {
    fn from_lua(value: &Value<'_>) -> Result<Self, Error> {
        if !value.is_string() {
            return Err(expected("string", value));
        }

        value.to_string()
    }
}

impl<T: FromLua> FromLua for Option<T> {
    fn from_lua(value: &Value<'_>) -> Result<Self, Error> {
        if value.is_nil() {
            Ok(None)
        } else {
            T::from_lua(value).map(Some)
        }
    }
}

impl IntoLua for () {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        Ok(lua.nil())
    }
}

impl IntoLua for bool {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        Ok(lua.bool(self))
    }
}

macro_rules! into_lua_integer {
    ($($t:ty),*) => {$(
        impl IntoLua for $t {
            fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
                Ok(match i64::try_from(self) {
                    Ok(n) => lua.integer(n),
                    Err(_) => lua.number(self as f64),
                })
            }
        }
    )*};
}

into_lua_integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl IntoLua for f32 {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        Ok(lua.number(self as f64))
    }
}

impl IntoLua for f64 {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        Ok(lua.number(self))
    }
}

impl IntoLua for &str {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        lua.string(self)
    }
}

impl IntoLua for String {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        lua.string(&self)
    }
}

impl<T: IntoLua> IntoLua for Option<T> {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        match self {
            Some(v) => v.into_lua(lua),
            None => Ok(lua.nil()),
        }
    }
}

impl<T: IntoLua> IntoLua for Result<T, Error> {
    fn into_lua<'l>(self, lua: &'l Lua) -> Result<Value<'l>, Error> {
        self?.into_lua(lua)
    }
}
