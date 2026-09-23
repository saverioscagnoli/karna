use quickjs::Context;
use quickjs::Error;
use quickjs::Runtime;

#[test]
fn eval_returns_values() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let v = ctx
        .eval(
            "[1, 2, 3].map(x => x * 2).reduce((a, b) => a + b)",
            "<test>",
        )
        .unwrap();
    assert_eq!(v.as_i32(), Some(12));
    assert_eq!(v.as_f64(), Some(12.0));

    let v = ctx.eval("0.5 + 0.25", "<test>").unwrap();
    assert_eq!(v.as_f64(), Some(0.75));
    assert_eq!(v.as_i32(), None);

    let v = ctx.eval("'kar' + 'na'", "<test>").unwrap();
    assert!(v.is_string());
    assert_eq!(v.to_string().unwrap(), "karna");

    let v = ctx.eval("1 < 2", "<test>").unwrap();
    assert_eq!(v.as_bool(), Some(true));

    assert!(ctx.eval("undefined", "<test>").unwrap().is_undefined());
}

#[test]
fn exceptions_become_errors() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let Err(Error::Exception(e)) = ctx.eval(
        "function f() { throw new TypeError('boom') }\nf()",
        "boom.js",
    ) else {
        panic!("expected an exception");
    };
    assert_eq!(e.message, "TypeError: boom");
    assert!(e.stack.unwrap().contains("boom.js"));

    let Err(Error::Exception(e)) = ctx.eval("throw null", "<test>") else {
        panic!("expected an exception");
    };
    assert_eq!(e.message, "null");
    assert!(e.stack.is_none());

    let Err(Error::Exception(e)) = ctx.eval("let x = ;", "<test>") else {
        panic!("expected a syntax error");
    };
    assert!(e.message.starts_with("SyntaxError"));

    assert!(matches!(ctx.eval("1\0", "<test>"), Err(Error::InteriorNul)));

    assert_eq!(ctx.eval("40 + 2", "<test>").unwrap().as_i32(), Some(42));
}

#[test]
fn globals_round_trip() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let global = ctx.global();
    global.set("answer", ctx.i32(42)).unwrap();
    global.set("name", ctx.string("karna").unwrap()).unwrap();
    global.set("ratio", ctx.f64(0.5)).unwrap();
    global.set("on", ctx.bool(true)).unwrap();

    let v = ctx
        .eval("`${name}:${answer * ratio}:${on}`", "<test>")
        .unwrap();
    assert_eq!(v.to_string().unwrap(), "karna:21:true");

    ctx.eval("var config = { width: 1280 }", "<test>").unwrap();
    let width = global.get("config").unwrap().get("width").unwrap();
    assert_eq!(width.as_i32(), Some(1280));

    assert!(global.get("missing").unwrap().is_undefined());
}

#[test]
fn clones_share_the_object() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let obj = ctx.eval("({ n: 1 })", "<test>").unwrap();
    let other = obj.clone();
    drop(obj);

    other.set("n", ctx.i32(2)).unwrap();
    assert_eq!(other.get("n").unwrap().as_i32(), Some(2));
}

#[test]
fn memory_limit_is_enforced() {
    let rt = Runtime::new().unwrap();
    rt.set_memory_limit(4 * 1024 * 1024);
    let ctx = Context::new(&rt).unwrap();

    let res = ctx.eval(
        "let a = []; for (;;) a.push(new Array(1024).fill(0));",
        "<test>",
    );
    assert!(matches!(res, Err(Error::Exception(_))));
}
