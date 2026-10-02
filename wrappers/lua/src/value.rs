use core::any::TypeId;
use core::ffi::CStr;
use core::ffi::c_int;
use core::fmt;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr;
use core::ptr::NonNull;

use alloc::string::String;
use alloc::vec::Vec;

use lua_sys::LUA_MULTRET;
use lua_sys::LUA_REFNIL;
use lua_sys::LUA_REGISTRYINDEX;
use lua_sys::LUA_TBOOLEAN;
use lua_sys::LUA_TFUNCTION;
use lua_sys::LUA_TNIL;
use lua_sys::LUA_TNUMBER;
use lua_sys::LUA_TSTRING;
use lua_sys::LUA_TTABLE;
use lua_sys::LUA_TUSERDATA;
use lua_sys::lua_State;
use lua_sys::lua_checkstack;
use lua_sys::lua_geti;
use lua_sys::lua_getmetatable;
use lua_sys::lua_gettable;
use lua_sys::lua_gettop;
use lua_sys::lua_isinteger;
use lua_sys::lua_len;
use lua_sys::lua_pop;
use lua_sys::lua_pushlstring;
use lua_sys::lua_rawequal;
use lua_sys::lua_rawgeti;
use lua_sys::lua_rotate;
use lua_sys::lua_seti;
use lua_sys::lua_settable;
use lua_sys::lua_toboolean;
use lua_sys::lua_tointegerx;
use lua_sys::lua_tonumberx;
use lua_sys::lua_touserdata;
use lua_sys::lua_type;
use lua_sys::lua_typename;
use lua_sys::luaL_ref;
use lua_sys::luaL_tolstring;
use lua_sys::luaL_unref;

use crate::Error;
use crate::IntoFunction;
use crate::Lua;
use crate::state::pcall;
use crate::state::protect;
use crate::state::read_string;
use crate::state::state;

pub struct Value<'lua> {
    L: NonNull<lua_State>,
    r: c_int,
    _lua: PhantomData<&'lua Lua>,
}

impl<'lua> Value<'lua> {
    pub(crate) unsafe fn pop(L: *mut lua_State) -> Self {
        let main = unsafe { state(L) }.main;
        let r = unsafe { luaL_ref(L, LUA_REGISTRYINDEX) };

        unsafe { Self::from_ref(main, r) }
    }

    pub(crate) unsafe fn from_ref(L: NonNull<lua_State>, r: c_int) -> Self {
        Self {
            L,
            r,
            _lua: PhantomData,
        }
    }

    pub(crate) unsafe fn nil(L: NonNull<lua_State>) -> Self {
        unsafe { Self::from_ref(L, LUA_REFNIL) }
    }

    pub(crate) fn into_ref(self) -> c_int {
        ManuallyDrop::new(self).r
    }

    pub fn push(&self, L: *mut lua_State) {
        assert_eq!(
            unsafe { state(L) }.main,
            self.L,
            "values from different lua states cannot be mixed",
        );

        unsafe { lua_rawgeti(L, LUA_REGISTRYINDEX, self.r as i64) };
    }

    pub fn state_ptr(&self) -> *mut lua_State {
        self.L.as_ptr()
    }

    fn with<R>(&self, f: impl FnOnce(*mut lua_State) -> R) -> R {
        let L = self.state_ptr();
        self.push(L);
        let r = f(L);
        unsafe { lua_pop(L, 1) };

        r
    }

    pub fn ty(&self) -> c_int {
        self.with(|L| unsafe { lua_type(L, -1) })
    }

    pub fn is_nil(&self) -> bool {
        self.ty() == LUA_TNIL
    }

    pub fn is_bool(&self) -> bool {
        self.ty() == LUA_TBOOLEAN
    }

    pub fn is_number(&self) -> bool {
        self.ty() == LUA_TNUMBER
    }

    pub fn is_integer(&self) -> bool {
        self.with(|L| unsafe { lua_isinteger(L, -1) } != 0)
    }

    pub fn is_string(&self) -> bool {
        self.ty() == LUA_TSTRING
    }

    pub fn is_table(&self) -> bool {
        self.ty() == LUA_TTABLE
    }

    pub fn is_function(&self) -> bool {
        self.ty() == LUA_TFUNCTION
    }

    pub fn is_userdata(&self) -> bool {
        self.ty() == LUA_TUSERDATA
    }

    pub fn type_name(&self) -> &'static str {
        let name = unsafe { CStr::from_ptr(lua_typename(self.state_ptr(), self.ty())) };
        name.to_str().unwrap_or("unknown")
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.with(|L| unsafe {
            (lua_type(L, -1) == LUA_TBOOLEAN).then(|| lua_toboolean(L, -1) != 0)
        })
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.with(|L| unsafe {
            let mut ok = 0;
            let n = lua_tointegerx(L, -1, &mut ok);
            (lua_type(L, -1) == LUA_TNUMBER && ok != 0).then_some(n)
        })
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.with(|L| unsafe {
            (lua_type(L, -1) == LUA_TNUMBER).then(|| lua_tonumberx(L, -1, ptr::null_mut()))
        })
    }

    pub fn to_string(&self) -> Result<String, Error> {
        let L = self.state_ptr();
        self.push(L);

        unsafe {
            protect(L, 1, 1, |L| {
                luaL_tolstring(L, 1, ptr::null_mut());
                1
            })?;

            let s = read_string(L, -1).unwrap_or_default();
            lua_pop(L, 1);

            Ok(s)
        }
    }

    pub fn get(&self, key: &str) -> Result<Value<'lua>, Error> {
        let L = self.state_ptr();
        self.push(L);

        unsafe {
            protect(L, 1, 1, move |L| {
                lua_pushlstring(L, key.as_ptr().cast(), key.len());
                lua_gettable(L, 1);
                1
            })?;

            Ok(Value::pop(L))
        }
    }

    pub fn set(&self, key: &str, value: Value<'_>) -> Result<(), Error> {
        let L = self.state_ptr();
        self.push(L);
        value.push(L);

        unsafe {
            protect(L, 2, 0, move |L| {
                lua_pushlstring(L, key.as_ptr().cast(), key.len());
                lua_rotate(L, 2, 1);
                lua_settable(L, 1);
                0
            })
        }
    }

    pub fn geti(&self, i: i64) -> Result<Value<'lua>, Error> {
        let L = self.state_ptr();
        self.push(L);

        unsafe {
            protect(L, 1, 1, move |L| {
                lua_geti(L, 1, i);
                1
            })?;

            Ok(Value::pop(L))
        }
    }

    pub fn seti(&self, i: i64, value: Value<'_>) -> Result<(), Error> {
        let L = self.state_ptr();
        self.push(L);
        value.push(L);

        unsafe {
            protect(L, 2, 0, move |L| {
                lua_seti(L, 1, i);
                0
            })
        }
    }

    pub fn len(&self) -> Result<i64, Error> {
        let L = self.state_ptr();
        self.push(L);

        unsafe {
            protect(L, 1, 1, |L| {
                lua_len(L, 1);
                1
            })?;
        }

        let len = unsafe { Value::pop(L) };
        len.as_i64()
            .ok_or_else(|| Error::Type(alloc::format!("length is a {}", len.type_name())))
    }

    pub fn set_fn<Args, F>(&self, key: &str, f: F) -> Result<(), Error>
    where
        F: IntoFunction<Args>,
    {
        let lua = ManuallyDrop::new(unsafe { Lua::from_ptr(self.state_ptr()) });
        self.set(key, lua.function(f)?)
    }

    pub fn call(&self, args: &[Value<'_>]) -> Result<Value<'lua>, Error> {
        let mut results = self.call_raw(None, args)?;

        Ok(match results.is_empty() {
            true => unsafe { Value::nil(self.L) },
            false => results.swap_remove(0),
        })
    }

    pub fn call_with(&self, this: &Value<'_>, args: &[Value<'_>]) -> Result<Value<'lua>, Error> {
        let mut results = self.call_raw(Some(this), args)?;

        Ok(match results.is_empty() {
            true => unsafe { Value::nil(self.L) },
            false => results.swap_remove(0),
        })
    }

    pub fn call_multi(&self, args: &[Value<'_>]) -> Result<Vec<Value<'lua>>, Error> {
        self.call_raw(None, args)
    }

    fn call_raw(
        &self,
        this: Option<&Value<'_>>,
        args: &[Value<'_>],
    ) -> Result<Vec<Value<'lua>>, Error> {
        let L = self.state_ptr();
        let nargs = args.len() + this.is_some() as usize;

        if unsafe { lua_checkstack(L, nargs as c_int + 4) } == 0 {
            return Err(Error::OutOfMemory);
        }

        let top = unsafe { lua_gettop(L) };

        self.push(L);
        this.into_iter().chain(args).for_each(|a| a.push(L));

        unsafe { pcall(L, nargs as c_int, LUA_MULTRET) }?;

        let n = unsafe { lua_gettop(L) } - top;
        let mut results = (0..n).map(|_| unsafe { Value::pop(L) }).collect::<Vec<_>>();
        results.reverse();

        Ok(results)
    }

    pub fn opaque<T: 'static>(&self) -> Option<&T> {
        let L = self.state_ptr();
        let mt = *unsafe { state(L) }
            .classes
            .borrow()
            .get(&TypeId::of::<T>())?;

        self.with(|L| unsafe {
            if lua_getmetatable(L, -1) == 0 {
                return None;
            }

            lua_rawgeti(L, LUA_REGISTRYINDEX, mt as i64);
            let same = lua_rawequal(L, -1, -2) != 0;
            lua_pop(L, 2);

            if !same {
                return None;
            }

            let slot = lua_touserdata(L, -1).cast::<*mut T>();
            (*slot).as_ref()
        })
    }
}

impl Clone for Value<'_> {
    fn clone(&self) -> Self {
        let L = self.state_ptr();
        self.push(L);

        unsafe { Value::pop(L) }
    }
}

impl Drop for Value<'_> {
    fn drop(&mut self) {
        unsafe { luaL_unref(self.state_ptr(), LUA_REGISTRYINDEX, self.r) };
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_string() {
            Ok(s) => write!(f, "Value({s:?})"),
            Err(_) => write!(f, "Value(<{}>)", self.type_name()),
        }
    }
}
