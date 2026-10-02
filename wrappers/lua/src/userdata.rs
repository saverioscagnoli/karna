use core::any::TypeId;
use core::any::type_name;
use core::cell::Cell;
use core::ffi::c_int;
use core::mem::size_of;
use core::ptr;

use alloc::boxed::Box;

use lua_sys::LUA_REGISTRYINDEX;
use lua_sys::lua_State;
use lua_sys::lua_createtable;
use lua_sys::lua_getfield;
use lua_sys::lua_newuserdatauv;
use lua_sys::lua_pushboolean;
use lua_sys::lua_pushcfunction;
use lua_sys::lua_pushlstring;
use lua_sys::lua_pushvalue;
use lua_sys::lua_rawgeti;
use lua_sys::lua_setfield;
use lua_sys::lua_setmetatable;
use lua_sys::lua_touserdata;
use lua_sys::luaL_ref;

use crate::Error;
use crate::Lua;
use crate::Value;
use crate::state::protect;

impl Lua {
    pub fn class<T: 'static>(&self) -> Result<Value<'_>, Error> {
        let mt = self.metatable::<T>()?;
        let L = self.as_ptr();

        unsafe {
            lua_rawgeti(L, LUA_REGISTRYINDEX, mt as i64);
            lua_getfield(L, -1, c"__index".as_ptr());
            let methods = Value::pop(L);
            lua_sys::lua_pop(L, 1);

            Ok(methods)
        }
    }

    pub fn instance<T: 'static>(&self, value: T) -> Result<Value<'_>, Error> {
        let mt = self.metatable::<T>()?;
        let ptr = Box::into_raw(Box::new(value));
        let owned = Cell::new(false);
        let owned = &owned;

        let res = unsafe {
            protect(self.as_ptr(), 0, 1, move |L| {
                let slot = lua_newuserdatauv(L, size_of::<*mut T>(), 0);
                *slot.cast::<*mut T>() = ptr;
                owned.set(true);

                lua_rawgeti(L, LUA_REGISTRYINDEX, mt as i64);
                lua_setmetatable(L, -2);
                1
            })
        };

        if let Err(e) = res {
            if !owned.get() {
                drop(unsafe { Box::from_raw(ptr) });
            }

            return Err(e);
        }

        Ok(unsafe { Value::pop(self.as_ptr()) })
    }

    fn metatable<T: 'static>(&self) -> Result<c_int, Error> {
        let classes = &self.state().classes;

        if let Some(&mt) = classes.borrow().get(&TypeId::of::<T>()) {
            return Ok(mt);
        }

        let name = type_name::<T>();
        let name = name.rsplit("::").next().unwrap_or(name);
        let L = self.as_ptr();

        unsafe {
            protect(L, 0, 1, move |L| {
                lua_createtable(L, 0, 4);
                lua_pushcfunction(L, Some(gc::<T>));
                lua_setfield(L, -2, c"__gc".as_ptr());
                lua_pushboolean(L, 0);
                lua_setfield(L, -2, c"__metatable".as_ptr());
                lua_pushlstring(L, name.as_ptr().cast(), name.len());
                lua_setfield(L, -2, c"__name".as_ptr());
                lua_createtable(L, 0, 0);
                lua_pushvalue(L, -1);
                lua_setfield(L, -3, c"__index".as_ptr());
                lua_sys::lua_pop(L, 1);
                1
            })?;
        }

        let mt = unsafe { luaL_ref(L, LUA_REGISTRYINDEX) };
        classes.borrow_mut().insert(TypeId::of::<T>(), mt);

        Ok(mt)
    }
}

pub(crate) unsafe extern "C" fn gc<T>(L: *mut lua_State) -> c_int {
    let slot = unsafe { lua_touserdata(L, 1) }.cast::<*mut T>();

    if !slot.is_null() {
        let ptr = unsafe { slot.replace(ptr::null_mut()) };

        if !ptr.is_null() {
            drop(unsafe { Box::from_raw(ptr) });
        }
    }

    0
}
