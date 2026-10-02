#![allow(non_snake_case)]

use core::ffi::CStr;
use core::ffi::c_int;

use lua_sys::*;

#[test]
fn reports_version() {
    unsafe {
        let L = luaL_newstate();
        assert!(!L.is_null(), "luaL_newstate failed");
        assert_eq!(lua_version(L), LUA_VERSION_NUM as lua_Number);
        luaL_checkversion(L);
        lua_close(L);
    }
}

#[test]
fn evaluates_script() {
    unsafe {
        let L = luaL_newstate();
        luaL_openlibs(L);

        let status = luaL_dostring(L, c"return string.rep('ab', 3), #{1, 2, 3} * 4".as_ptr());
        assert_eq!(status, LUA_OK, "script failed");
        assert_eq!(lua_gettop(L), 2);

        assert!(lua_isinteger(L, -1) != 0);
        assert_eq!(lua_tointeger(L, -1), 12);
        assert_eq!(CStr::from_ptr(lua_tostring(L, -2)), c"ababab");
        lua_pop(L, 2);
        assert_eq!(lua_gettop(L), 0);

        lua_close(L);
    }
}

#[test]
fn calls_rust_functions() {
    unsafe extern "C" fn add(L: *mut lua_State) -> c_int {
        unsafe {
            let a = luaL_checkinteger(L, 1);
            let b = luaL_checkinteger(L, 2);
            lua_pushinteger(L, a + b + lua_tointeger(L, lua_upvalueindex(1)));
        }
        1
    }

    unsafe {
        let L = luaL_newstate();

        lua_pushinteger(L, 100);
        lua_pushcclosure(L, Some(add), 1);
        lua_setglobal(L, c"add".as_ptr());

        assert_eq!(luaL_dostring(L, c"return add(2, 3)".as_ptr()), LUA_OK);
        assert_eq!(lua_tointeger(L, -1), 105);

        lua_pushglobaltable(L);
        assert!(lua_istable(L, -1));
        lua_getfield(L, -1, c"add".as_ptr());
        assert!(lua_isfunction(L, -1));

        lua_close(L);
    }
}

#[test]
fn surfaces_errors() {
    unsafe {
        let L = luaL_newstate();
        luaL_openlibs(L);

        let status = luaL_dostring(L, c"error('boom', 0)".as_ptr());
        assert_eq!(status, LUA_ERRRUN);
        assert_eq!(CStr::from_ptr(lua_tostring(L, -1)), c"boom");
        lua_pop(L, 1);

        let status = luaL_dostring(L, c"this is not lua".as_ptr());
        assert_eq!(status, LUA_ERRSYNTAX);

        lua_close(L);
    }
}

#[test]
fn builds_strings_in_a_buffer() {
    unsafe {
        let L = luaL_newstate();

        let mut b = core::mem::MaybeUninit::<luaL_Buffer>::uninit();
        let b = b.as_mut_ptr();
        luaL_buffinit(L, b);
        for c in b"karna" {
            luaL_addchar(b, *c as _);
        }
        luaL_addstring(b, c"-lua".as_ptr());
        luaL_pushresult(b);

        assert_eq!(CStr::from_ptr(lua_tostring(L, -1)), c"karna-lua");

        lua_close(L);
    }
}
