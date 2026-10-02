use core::any::TypeId;
use core::cell::Cell;
use core::cell::RefCell;
use core::ffi::c_int;
use core::ptr;
use core::ptr::NonNull;
use core::slice;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::ffi::CString;
use alloc::format;
use alloc::string::String;

use lua_sys::LUA_ERRMEM;
use lua_sys::LUA_NOREF;
use lua_sys::LUA_OK;
use lua_sys::LUA_REGISTRYINDEX;
use lua_sys::LUA_TSTRING;
use lua_sys::lua_State;
use lua_sys::lua_checkstack;
use lua_sys::lua_close;
use lua_sys::lua_createtable;
use lua_sys::lua_getextraspace;
use lua_sys::lua_gettop;
use lua_sys::lua_insert;
use lua_sys::lua_pcall;
use lua_sys::lua_pop;
use lua_sys::lua_pushboolean;
use lua_sys::lua_pushcfunction;
use lua_sys::lua_pushglobaltable;
use lua_sys::lua_pushinteger;
use lua_sys::lua_pushlightuserdata;
use lua_sys::lua_pushlstring;
use lua_sys::lua_pushnumber;
use lua_sys::lua_remove;
use lua_sys::lua_setfield;
use lua_sys::lua_tolstring;
use lua_sys::lua_touserdata;
use lua_sys::lua_type;
use lua_sys::luaL_loadbufferx;
use lua_sys::luaL_newstate;
use lua_sys::luaL_openlibs;
use lua_sys::luaL_ref;
use lua_sys::luaL_tolstring;
use lua_sys::luaL_traceback;

use crate::Error;
use crate::Exception;
use crate::Persistent;
use crate::Value;
use crate::function::RawFn;
use crate::userdata::gc;

pub(crate) struct State {
    pub main: NonNull<lua_State>,
    pub closure_mt: Cell<c_int>,
    pub classes: RefCell<BTreeMap<TypeId, c_int>>,
}

pub struct Lua {
    raw: NonNull<lua_State>,
}

impl Lua {
    pub fn new() -> Result<Self, Error> {
        let raw = NonNull::new(unsafe { luaL_newstate() }).ok_or(Error::OutOfMemory)?;

        let state = Box::new(State {
            main: raw,
            closure_mt: Cell::new(LUA_NOREF),
            classes: RefCell::new(BTreeMap::new()),
        });

        unsafe { *lua_getextraspace(raw.as_ptr()).cast::<*mut State>() = Box::into_raw(state) };

        let lua = Self { raw };
        let L = lua.as_ptr();

        unsafe {
            protect(L, 0, 1, |L| {
                luaL_openlibs(L);
                lua_createtable(L, 0, 2);
                lua_pushcfunction(L, Some(gc::<Box<RawFn>>));
                lua_setfield(L, -2, c"__gc".as_ptr());
                lua_pushboolean(L, 0);
                lua_setfield(L, -2, c"__metatable".as_ptr());
                1
            })?;

            lua.state().closure_mt.set(luaL_ref(L, LUA_REGISTRYINDEX));
        }

        Ok(lua)
    }

    pub(crate) unsafe fn from_ptr(L: *mut lua_State) -> Self {
        Self {
            raw: unsafe { state(L) }.main,
        }
    }

    pub fn load(&self, src: &str, name: &str) -> Result<Value<'_>, Error> {
        let name = CString::new(format!("={name}")).map_err(|_| Error::InteriorNul)?;
        let L = self.as_ptr();

        let status = unsafe {
            luaL_loadbufferx(
                L,
                src.as_ptr().cast(),
                src.len(),
                name.as_ptr(),
                c"t".as_ptr(),
            )
        };

        if status != LUA_OK {
            return Err(unsafe { pop_error(L, status) });
        }

        Ok(unsafe { Value::pop(L) })
    }

    pub fn eval(&self, src: &str, name: &str) -> Result<Value<'_>, Error> {
        self.load(src, name)?.call(&[])
    }

    pub fn globals(&self) -> Value<'_> {
        unsafe {
            lua_pushglobaltable(self.as_ptr());
            Value::pop(self.as_ptr())
        }
    }

    pub fn nil(&self) -> Value<'_> {
        unsafe { Value::nil(self.raw) }
    }

    pub fn bool(&self, v: bool) -> Value<'_> {
        unsafe {
            lua_pushboolean(self.as_ptr(), v as c_int);
            Value::pop(self.as_ptr())
        }
    }

    pub fn integer(&self, v: i64) -> Value<'_> {
        unsafe {
            lua_pushinteger(self.as_ptr(), v);
            Value::pop(self.as_ptr())
        }
    }

    pub fn number(&self, v: f64) -> Value<'_> {
        unsafe {
            lua_pushnumber(self.as_ptr(), v);
            Value::pop(self.as_ptr())
        }
    }

    pub fn string(&self, v: &str) -> Result<Value<'_>, Error> {
        unsafe {
            protect(self.as_ptr(), 0, 1, move |L| {
                lua_pushlstring(L, v.as_ptr().cast(), v.len());
                1
            })?;

            Ok(Value::pop(self.as_ptr()))
        }
    }

    pub fn table(&self) -> Result<Value<'_>, Error> {
        unsafe {
            protect(self.as_ptr(), 0, 1, |L| {
                lua_createtable(L, 0, 0);
                1
            })?;

            Ok(Value::pop(self.as_ptr()))
        }
    }

    pub fn persist(&self, value: Value<'_>) -> Persistent<'_> {
        unsafe { Persistent::new(self.raw, value.into_ref()) }
    }

    pub fn as_ptr(&self) -> *mut lua_State {
        self.raw.as_ptr()
    }

    pub(crate) fn state(&self) -> &State {
        unsafe { state(self.as_ptr()) }
    }
}

impl Drop for Lua {
    fn drop(&mut self) {
        let state = unsafe { *lua_getextraspace(self.as_ptr()).cast::<*mut State>() };

        unsafe { lua_close(self.as_ptr()) };
        drop(unsafe { Box::from_raw(state) });
    }
}

pub(crate) unsafe fn state<'a>(L: *mut lua_State) -> &'a State {
    unsafe { &**lua_getextraspace(L).cast::<*mut State>() }
}

pub(crate) unsafe fn protect<F>(
    L: *mut lua_State,
    nargs: c_int,
    nresults: c_int,
    f: F,
) -> Result<(), Error>
where
    F: Fn(*mut lua_State) -> c_int + Copy,
{
    unsafe extern "C" fn run<F>(L: *mut lua_State) -> c_int
    where
        F: Fn(*mut lua_State) -> c_int + Copy,
    {
        let f = unsafe { *lua_touserdata(L, -1).cast::<F>() };
        unsafe { lua_pop(L, 1) };

        f(L)
    }

    unsafe {
        if lua_checkstack(L, 3) == 0 {
            return Err(Error::OutOfMemory);
        }

        lua_pushcfunction(L, Some(run::<F>));
        lua_insert(L, -(nargs + 1));
        lua_pushlightuserdata(L, ptr::from_ref(&f).cast_mut().cast());

        pcall(L, nargs + 1, nresults)
    }
}

pub(crate) unsafe fn pcall(L: *mut lua_State, nargs: c_int, nresults: c_int) -> Result<(), Error> {
    unsafe {
        let base = lua_gettop(L) - nargs;

        lua_pushcfunction(L, Some(handler));
        lua_insert(L, base);

        let status = lua_pcall(L, nargs, nresults, base);
        lua_remove(L, base);

        if status == LUA_OK {
            Ok(())
        } else {
            Err(pop_error(L, status))
        }
    }
}

unsafe extern "C" fn handler(L: *mut lua_State) -> c_int {
    unsafe {
        let msg = luaL_tolstring(L, 1, ptr::null_mut());
        luaL_traceback(L, L, msg, 1);
    }

    1
}

pub(crate) unsafe fn pop_error(L: *mut lua_State, status: c_int) -> Error {
    if status == LUA_ERRMEM {
        unsafe { lua_pop(L, 1) };
        return Error::OutOfMemory;
    }

    let text = unsafe { read_string(L, -1) }.unwrap_or_else(|| String::from("unknown error"));
    unsafe { lua_pop(L, 1) };

    let (message, stack) = match text.find("\nstack traceback:") {
        Some(i) => (String::from(&text[..i]), Some(String::from(&text[i + 1..]))),
        None => (text, None),
    };

    Error::Exception(Exception { message, stack })
}

pub(crate) unsafe fn read_string(L: *mut lua_State, idx: c_int) -> Option<String> {
    if unsafe { lua_type(L, idx) } != LUA_TSTRING {
        return None;
    }

    let mut len = 0;
    let ptr = unsafe { lua_tolstring(L, idx, &mut len) };
    let bytes = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), len) };

    Some(String::from_utf8_lossy(bytes).into_owned())
}
