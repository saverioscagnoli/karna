use core::cell::Cell;
use core::ffi::c_int;
use core::mem::ManuallyDrop;
use core::mem::size_of;

use alloc::boxed::Box;
use alloc::vec::Vec;

use lua_sys::LUA_REGISTRYINDEX;
use lua_sys::lua_State;
use lua_sys::lua_error;
use lua_sys::lua_gettop;
use lua_sys::lua_newuserdatauv;
use lua_sys::lua_pushcclosure;
use lua_sys::lua_pushlstring;
use lua_sys::lua_pushvalue;
use lua_sys::lua_rawgeti;
use lua_sys::lua_setmetatable;
use lua_sys::lua_touserdata;
use lua_sys::lua_upvalueindex;

use crate::Error;
use crate::FromLua;
use crate::IntoLua;
use crate::Lua;
use crate::Value;
use crate::state::protect;

pub type RawFn = dyn for<'l> Fn(&'l Lua, &[Value<'l>]) -> Result<Value<'l>, Error>;

impl Lua {
    pub fn function<Args, F>(&self, f: F) -> Result<Value<'_>, Error>
    where
        F: IntoFunction<Args>,
    {
        self.function_raw(move |lua, args| f.call(lua, args))
    }

    pub fn function_raw<F>(&self, f: F) -> Result<Value<'_>, Error>
    where
        F: for<'l> Fn(&'l Lua, &[Value<'l>]) -> Result<Value<'l>, Error> + 'static,
    {
        let boxed: Box<Box<RawFn>> = Box::new(Box::new(f));
        let ptr = Box::into_raw(boxed);
        let mt = self.state().closure_mt.get();
        let owned = Cell::new(false);
        let owned = &owned;

        let res = unsafe {
            protect(self.as_ptr(), 0, 1, move |L| {
                let slot = lua_newuserdatauv(L, size_of::<*mut Box<RawFn>>(), 0);
                *slot.cast::<*mut Box<RawFn>>() = ptr;
                owned.set(true);

                lua_rawgeti(L, LUA_REGISTRYINDEX, mt as i64);
                lua_setmetatable(L, -2);
                lua_pushcclosure(L, Some(trampoline), 1);
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
}

unsafe extern "C" fn trampoline(L: *mut lua_State) -> c_int {
    match unsafe { invoke(L) } {
        Some(n) => n,
        None => unsafe { lua_error(L) },
    }
}

unsafe fn invoke(L: *mut lua_State) -> Option<c_int> {
    let slot = unsafe { lua_touserdata(L, lua_upvalueindex(1)) }.cast::<*mut Box<RawFn>>();
    let f = unsafe { &**slot };

    let lua = ManuallyDrop::new(unsafe { Lua::from_ptr(L) });
    let n = unsafe { lua_gettop(L) };

    let args = (1..=n)
        .map(|i| unsafe {
            lua_pushvalue(L, i);
            Value::pop(L)
        })
        .collect::<Vec<_>>();

    match f(&lua, &args) {
        Ok(v) => {
            v.push(L);
            Some(1)
        }
        Err(e) => {
            let msg = e.message();
            unsafe { lua_pushlstring(L, msg.as_ptr().cast(), msg.len()) };
            None
        }
    }
}

pub trait IntoFunction<Args>: 'static {
    fn call<'l>(&self, lua: &'l Lua, args: &[Value<'l>]) -> Result<Value<'l>, Error>;
}

macro_rules! impl_into_function {
    ($($arg:ident),*) => {
        impl<F, R, $($arg,)*> IntoFunction<($($arg,)*)> for F
        where
            F: Fn($($arg),*) -> R + 'static,
            R: IntoLua,
            $($arg: FromLua,)*
        {
            #[allow(non_snake_case, unused_variables, unused_mut)]
            fn call<'l>(&self, lua: &'l Lua, args: &[Value<'l>]) -> Result<Value<'l>, Error> {
                let nil = lua.nil();
                let mut args = args.iter().chain(core::iter::repeat(&nil)).enumerate();

                $(
                    let (i, v) = args.next().unwrap();
                    let $arg = $arg::from_lua(v).map_err(|e| match e {
                        Error::Type(msg) => Error::Type(alloc::format!("argument {}: {msg}", i + 1)),
                        e => e,
                    })?;
                )*

                (self)($($arg),*).into_lua(lua)
            }
        }
    };
}

impl_into_function!();
impl_into_function!(A);
impl_into_function!(A, B);
impl_into_function!(A, B, C);
impl_into_function!(A, B, C, D);
impl_into_function!(A, B, C, D, E);
impl_into_function!(A, B, C, D, E, G);
impl_into_function!(A, B, C, D, E, G, H);
impl_into_function!(A, B, C, D, E, G, H, I);
