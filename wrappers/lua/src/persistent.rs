use core::ffi::c_int;
use core::marker::PhantomData;
use core::ptr::NonNull;

use lua_sys::LUA_REGISTRYINDEX;
use lua_sys::lua_State;
use lua_sys::lua_rawgeti;
use lua_sys::luaL_unref;

use crate::Lua;
use crate::Value;

pub struct Persistent<'lua> {
    L: NonNull<lua_State>,
    r: c_int,
    _lua: PhantomData<&'lua Lua>,
}

impl<'lua> Persistent<'lua> {
    pub(crate) unsafe fn new(L: NonNull<lua_State>, r: c_int) -> Self {
        Self {
            L,
            r,
            _lua: PhantomData,
        }
    }

    pub fn get<'l>(&self, lua: &'l Lua) -> Value<'l> {
        assert_eq!(
            lua.as_ptr(),
            self.L.as_ptr(),
            "values from different lua states cannot be mixed",
        );

        unsafe {
            lua_rawgeti(lua.as_ptr(), LUA_REGISTRYINDEX, self.r as i64);
            Value::pop(lua.as_ptr())
        }
    }
}

impl Drop for Persistent<'_> {
    fn drop(&mut self) {
        unsafe { luaL_unref(self.L.as_ptr(), LUA_REGISTRYINDEX, self.r) };
    }
}
