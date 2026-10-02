use std::cell::Cell;
use std::rc::Rc;

use lua::Error;
use lua::Lua;

#[test]
fn evaluates_chunks() {
    let lua = Lua::new().unwrap();

    assert_eq!(lua.eval("return 40 + 2", "t").unwrap().as_i64(), Some(42));
    assert_eq!(lua.eval("return 0.5", "t").unwrap().as_f64(), Some(0.5));
    assert_eq!(
        lua.eval("return ('ab'):rep(2)", "t").unwrap().to_string().unwrap(),
        "abab"
    );
    assert!(lua.eval("local x = 1", "t").unwrap().is_nil());

    let results = lua.load("return 1, 'two', true", "t").unwrap().call_multi(&[]).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].as_i64(), Some(1));
    assert_eq!(results[1].to_string().unwrap(), "two");
    assert_eq!(results[2].as_bool(), Some(true));
}

#[test]
fn reads_and_writes_tables() {
    let lua = Lua::new().unwrap();
    let globals = lua.globals();

    let t = lua.table().unwrap();
    t.set("name", lua.string("karna").unwrap()).unwrap();
    t.seti(1, lua.integer(10)).unwrap();
    t.seti(2, lua.integer(20)).unwrap();
    globals.set("t", t).unwrap();

    assert_eq!(
        lua.eval("return t.name .. #t", "t").unwrap().to_string().unwrap(),
        "karna2"
    );

    let t = globals.get("t").unwrap();
    assert!(t.is_table());
    assert_eq!(t.geti(2).unwrap().as_i64(), Some(20));
    assert_eq!(t.len().unwrap(), 2);
    assert!(t.get("missing").unwrap().is_nil());
}

#[test]
fn typed_functions_convert_arguments() {
    let lua = Lua::new().unwrap();
    let globals = lua.globals();

    globals.set_fn("add", |a: f64, b: f64| a + b).unwrap();
    globals
        .set_fn("greet", |name: String, excl: Option<bool>| {
            format!("hi {name}{}", if excl == Some(true) { "!" } else { "" })
        })
        .unwrap();

    assert_eq!(lua.eval("return add(40, 2)", "t").unwrap().as_f64(), Some(42.0));
    assert_eq!(
        lua.eval("return greet('karna') .. ' ' .. greet('you', true)", "t")
            .unwrap()
            .to_string()
            .unwrap(),
        "hi karna hi you!"
    );

    let Err(Error::Exception(e)) = lua.eval("return add(1, 'two')", "t") else {
        panic!("expected an error");
    };
    assert_eq!(e.message, "argument 2: expected number, got string");
    assert!(e.stack.unwrap().starts_with("stack traceback:"));

    let Err(Error::Exception(e)) = lua.eval("return add(1)", "t") else {
        panic!("expected an error");
    };
    assert_eq!(e.message, "argument 2: expected number, got nil");
}

#[test]
fn errors_are_caught() {
    let lua = Lua::new().unwrap();

    let Err(Error::Exception(e)) = lua.eval("error('boom')", "chunk") else {
        panic!("expected an error");
    };
    assert_eq!(e.message, "chunk:1: boom");

    let Err(Error::Exception(e)) = lua.eval("this is not lua", "chunk") else {
        panic!("expected a syntax error");
    };
    assert!(e.message.starts_with("chunk:1:"));
    assert!(e.stack.is_none());

    let Err(Error::Exception(e)) = lua.globals().get("nope").unwrap().get("field") else {
        panic!("expected an index error");
    };
    assert!(e.message.contains("attempt to index a nil value"));

    let Err(Error::Exception(e)) = lua.eval("error({})", "chunk") else {
        panic!("expected an error");
    };
    assert!(e.message.starts_with("table: "));

    lua.globals()
        .set_fn("fail", || -> Result<(), Error> { Err(Error::custom("from rust")) })
        .unwrap();

    let v = lua
        .eval("local ok, msg = pcall(fail); return tostring(ok) .. ' ' .. msg", "t")
        .unwrap();
    assert_eq!(v.to_string().unwrap(), "false from rust");
}

#[test]
fn closures_are_dropped_with_the_state() {
    let dropped = Rc::new(Cell::new(false));

    struct Flag(Rc<Cell<bool>>);

    impl Drop for Flag {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    {
        let lua = Lua::new().unwrap();
        let flag = Flag(dropped.clone());
        lua.globals()
            .set_fn("f", move || flag.0.get())
            .unwrap();
        assert_eq!(lua.eval("return f()", "t").unwrap().as_bool(), Some(false));
        assert!(!dropped.get());
    }

    assert!(dropped.get());
}

#[test]
fn userdata_instances() {
    struct Counter(Cell<i64>);

    let lua = Lua::new().unwrap();

    lua.class::<Counter>()
        .unwrap()
        .set(
            "bump",
            lua.function_raw(|lua, args| {
                let counter = args[0]
                    .opaque::<Counter>()
                    .ok_or_else(|| Error::Type("expected a Counter".into()))?;
                counter.0.set(counter.0.get() + 1);
                Ok(lua.integer(counter.0.get()))
            })
            .unwrap(),
        )
        .unwrap();

    let c = lua.instance(Counter(Cell::new(0))).unwrap();
    lua.globals().set("c", c.clone()).unwrap();

    assert_eq!(lua.eval("c:bump(); return c:bump()", "t").unwrap().as_i64(), Some(2));
    assert_eq!(c.opaque::<Counter>().unwrap().0.get(), 2);
    assert!(lua.eval("return getmetatable(c)", "t").unwrap().as_bool() == Some(false));
    assert!(lua.integer(1).opaque::<Counter>().is_none());

    let Err(Error::Exception(e)) = lua.eval("return c.bump({})", "t") else {
        panic!("expected an error");
    };
    assert_eq!(e.message, "expected a Counter");
}

#[test]
fn persistent_values_outlive_borrows() {
    let lua = Lua::new().unwrap();
    let f = lua.persist(lua.eval("return function(x) return x * 2 end", "t").unwrap());

    let v = f.get(&lua).call(&[lua.integer(21)]).unwrap();
    assert_eq!(v.as_i64(), Some(42));
}

#[test]
fn methods_are_called_with_self() {
    let lua = Lua::new().unwrap();
    let scene = lua
        .eval("return { n = 1, update = function(self, d) self.n = self.n + d end }", "t")
        .unwrap();

    let update = scene.get("update").unwrap();
    update.call_with(&scene, &[lua.integer(4)]).unwrap();

    assert_eq!(scene.get("n").unwrap().as_i64(), Some(5));
}

#[test]
fn modules_load_through_the_loader() {
    let lua = Lua::new().unwrap();

    lua.set_module_loader(|name| match name {
        "util" => Ok("return { double = function(x) return x * 2 end }".into()),
        _ => Err(Error::custom("not found")),
    })
    .unwrap();

    assert_eq!(
        lua.eval("return require('util').double(21)", "t").unwrap().as_i64(),
        Some(42)
    );

    let Err(Error::Exception(e)) = lua.eval("return require('nope')", "t") else {
        panic!("expected an error");
    };
    assert!(e.message.contains("not found"));
}

#[test]
fn callbacks_work_inside_coroutines() {
    let lua = Lua::new().unwrap();
    lua.globals().set_fn("twice", |x: i64| x * 2).unwrap();

    let v = lua
        .eval(
            "local co = coroutine.wrap(function(a) local b = coroutine.yield(twice(a)); return twice(b) end)
             return co(1) + co(10)",
            "t",
        )
        .unwrap();

    assert_eq!(v.as_i64(), Some(22));
}
