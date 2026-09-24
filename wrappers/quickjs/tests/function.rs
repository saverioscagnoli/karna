use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use quickjs::Context;
use quickjs::Error;
use quickjs::Runtime;

#[test]
fn typed_functions_convert_arguments() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();
    let global = ctx.global();

    global
        .set("add", ctx.function("add", |a: f64, b: f64| a + b).unwrap())
        .unwrap();
    global
        .set(
            "greet",
            ctx.function("greet", |name: String, excl: Option<bool>| {
                format!("hi {name}{}", if excl == Some(true) { "!" } else { "" })
            })
            .unwrap(),
        )
        .unwrap();

    assert_eq!(
        ctx.eval("add(40, 2)", "<test>").unwrap().as_f64(),
        Some(42.0)
    );
    assert_eq!(
        ctx.eval("add(0.5, 0.25)", "<test>").unwrap().as_f64(),
        Some(0.75)
    );
    assert_eq!(ctx.eval("add.length", "<test>").unwrap().as_i32(), Some(2));
    assert_eq!(
        ctx.eval("add.name", "<test>").unwrap().to_string().unwrap(),
        "add"
    );

    let v = ctx.eval("greet('karna') + ' ' + greet('you', true)", "<test>");
    assert_eq!(v.unwrap().to_string().unwrap(), "hi karna hi you!");

    let Err(Error::Exception(e)) = ctx.eval("add(1, 'two')", "<test>") else {
        panic!("expected a type error");
    };
    assert_eq!(
        e.message,
        "TypeError: argument 2: expected number, got string"
    );

    let Err(Error::Exception(e)) = ctx.eval("add(1)", "<test>") else {
        panic!("expected a type error");
    };
    assert_eq!(
        e.message,
        "TypeError: argument 2: expected number, got undefined"
    );
}

#[test]
fn closures_keep_state_and_can_fail() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();

    let f = ctx
        .function("log", move |s: String| sink.borrow_mut().push(s))
        .unwrap();
    ctx.global().set("log", f).unwrap();

    let fail = ctx
        .function("fail", |msg: String| -> Result<(), Error> {
            Err(Error::custom(msg))
        })
        .unwrap();
    ctx.global().set("fail", fail).unwrap();

    ctx.eval("log('a'); log('b')", "<test>").unwrap();
    assert_eq!(*log.borrow(), ["a", "b"]);

    let v = ctx.eval(
        "try { fail('nope') } catch (e) { `${e instanceof Error}:${e.message}` }",
        "<test>",
    );
    assert_eq!(v.unwrap().to_string().unwrap(), "true:nope");
}

#[test]
fn closures_are_dropped_with_the_runtime() {
    struct Flag(Rc<Cell<bool>>);

    impl Drop for Flag {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    let dropped = Rc::new(Cell::new(false));

    {
        let rt = Runtime::new().unwrap();
        let ctx = Context::new(&rt).unwrap();
        let flag = Flag(dropped.clone());

        let f = ctx.function("f", move || flag.0.get()).unwrap();
        ctx.global().set("f", f).unwrap();
        ctx.eval("f()", "<test>").unwrap();

        assert!(!dropped.get());
    }

    assert!(dropped.get());
}

#[test]
fn raw_functions_see_this_and_values() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let f = ctx
        .function_raw("describe", 0, |ctx, this, args| {
            let kinds = args.iter().map(|a| a.type_name()).collect::<Vec<_>>();
            let tag = this.get("tag")?.to_string()?;

            ctx.string(&format!("{tag}:{}", kinds.join(",")))
        })
        .unwrap();
    ctx.global().set("describe", f).unwrap();

    let v = ctx
        .eval(
            "({ tag: 'obj', describe }).describe(1, 'a', null, {}, () => 1)",
            "<test>",
        )
        .unwrap();
    assert_eq!(
        v.to_string().unwrap(),
        "obj:number,string,null,object,function"
    );
}

#[test]
fn rust_calls_js() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    ctx.eval(
        "function update(dt) { this.t = (this.t ?? 0) + dt; return this.t }",
        "<test>",
    )
    .unwrap();

    let update = ctx.global().get("update").unwrap();
    assert!(update.is_function());

    let state = ctx.object().unwrap();
    update.call_with(&state, &[ctx.f64(0.5)]).unwrap();
    let t = update.call_with(&state, &[ctx.f64(0.25)]).unwrap();
    assert_eq!(t.as_f64(), Some(0.75));
    assert_eq!(state.get("t").unwrap().as_f64(), Some(0.75));

    ctx.eval("function boom() { throw new RangeError('x') }", "<test>")
        .unwrap();
    let boom = ctx.global().get("boom").unwrap();
    let Err(Error::Exception(e)) = boom.call(&[]) else {
        panic!("expected an exception");
    };
    assert_eq!(e.message, "RangeError: x");
}

#[test]
fn classes_wrap_rust_values() {
    struct Counter {
        n: Cell<i32>,
        dropped: Rc<Cell<bool>>,
    }

    impl Drop for Counter {
        fn drop(&mut self) {
            self.dropped.set(true);
        }
    }

    let dropped = Rc::new(Cell::new(false));

    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let proto = ctx.class::<Counter>().unwrap();
    let bump = ctx
        .function_raw("bump", 0, |ctx, this, _| {
            let counter = this
                .opaque::<Counter>()
                .ok_or_else(|| Error::Type("not a Counter".into()))?;
            counter.n.set(counter.n.get() + 1);

            Ok(ctx.i32(counter.n.get()))
        })
        .unwrap();
    proto.set("bump", bump).unwrap();

    let counter = ctx
        .instance(Counter {
            n: Cell::new(0),
            dropped: dropped.clone(),
        })
        .unwrap();
    ctx.global().set("counter", counter).unwrap();

    let v = ctx
        .eval("counter.bump(); counter.bump()", "<test>")
        .unwrap();
    assert_eq!(v.as_i32(), Some(2));

    let n = ctx.global().get("counter").unwrap();
    assert_eq!(n.opaque::<Counter>().unwrap().n.get(), 2);
    assert!(n.opaque::<String>().is_none());
    drop(n);

    let Err(Error::Exception(e)) = ctx.eval("counter.bump.call({})", "<test>") else {
        panic!("expected a type error");
    };
    assert_eq!(e.message, "TypeError: not a Counter");

    ctx.eval("counter = undefined", "<test>").unwrap();
    rt.run_gc();
    assert!(dropped.get());
}

#[test]
fn pending_jobs_run_on_demand() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    ctx.eval(
        "var done = false; (async () => { await null; done = true })()",
        "<test>",
    )
    .unwrap();
    assert_eq!(ctx.global().get("done").unwrap().as_bool(), Some(false));

    rt.run_jobs().unwrap();
    assert_eq!(ctx.global().get("done").unwrap().as_bool(), Some(true));

    ctx.eval(
        "Promise.resolve().then(() => { throw new Error('late') })",
        "<test>",
    )
    .unwrap();
    let Err(Error::Exception(e)) = rt.run_jobs() else {
        panic!("expected an unhandled rejection");
    };
    assert_eq!(e.message, "Error: late");
    rt.run_jobs().unwrap();

    ctx.eval(
        "var caught; var p = Promise.reject(new Error('x')); p.catch(e => caught = e.message)",
        "<test>",
    )
    .unwrap();
    rt.run_jobs().unwrap();
    assert_eq!(
        ctx.global().get("caught").unwrap().to_string().unwrap(),
        "x"
    );

    // Left pending at drop: must not leak or trip QuickJS' leak assertion.
    ctx.eval("Promise.reject(1)", "<test>").unwrap();
}
