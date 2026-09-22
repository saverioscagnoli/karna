use core::ffi::CStr;

use quickjs_sys::*;

#[test]
fn reports_version() {
    let v = unsafe { CStr::from_ptr(JS_GetVersion()) };
    let expected = format!("{QJS_VERSION_MAJOR}.{QJS_VERSION_MINOR}.{QJS_VERSION_PATCH}");
    assert_eq!(v.to_str().unwrap(), expected);
    println!("linked against quickjs-ng {expected}");
}

#[test]
fn evaluates_script() {
    unsafe {
        let rt = JS_NewRuntime();
        assert!(!rt.is_null(), "JS_NewRuntime failed");
        let ctx = JS_NewContext(rt);
        assert!(!ctx.is_null(), "JS_NewContext failed");

        let src = c"[1, 2, 3].map(x => x * 2).reduce((a, b) => a + b)";
        let filename = c"<smoke>";
        let v = JS_Eval(
            ctx,
            src.as_ptr(),
            src.count_bytes(),
            filename.as_ptr(),
            JS_EVAL_TYPE_GLOBAL as i32,
        );
        assert!(!JS_IsException(v), "script threw");
        assert_eq!(JS_VALUE_GET_TAG(v), JS_TAG_INT);
        assert_eq!(JS_VALUE_GET_INT(v), 12);
        JS_FreeValue(ctx, v);

        // Goes through the static inline shim in extern.c.
        let s = JS_NewString(ctx, c"karna".as_ptr());
        assert!(JS_IsString(s));
        let back = JS_ToCString(ctx, s);
        assert_eq!(CStr::from_ptr(back), c"karna");
        JS_FreeCString(ctx, back);
        JS_FreeValue(ctx, s);

        assert!(JS_IsUndefined(JS_UNDEFINED));
        assert!(JS_IsException(JS_EXCEPTION));

        JS_FreeContext(ctx);
        JS_FreeRuntime(rt);
    }
}

#[test]
fn surfaces_exceptions() {
    unsafe {
        let rt = JS_NewRuntime();
        let ctx = JS_NewContext(rt);

        let src = c"throw new Error('boom')";
        let v = JS_Eval(
            ctx,
            src.as_ptr(),
            src.count_bytes(),
            c"<smoke>".as_ptr(),
            JS_EVAL_TYPE_GLOBAL as i32,
        );
        assert!(JS_IsException(v));

        let err = JS_GetException(ctx);
        let msg = JS_ToCString(ctx, err);
        assert_eq!(CStr::from_ptr(msg), c"Error: boom");
        JS_FreeCString(ctx, msg);
        JS_FreeValue(ctx, err);

        JS_FreeContext(ctx);
        JS_FreeRuntime(rt);
    }
}
