use std::collections::HashMap;

use quickjs::Context;
use quickjs::Error;
use quickjs::Runtime;

fn files(entries: &[(&str, &str)]) -> impl Fn(&str) -> Result<String, Error> + 'static {
    let files = entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>();

    move |name| {
        files
            .get(name)
            .cloned()
            .ok_or_else(|| Error::custom("no such file"))
    }
}

#[test]
fn default_export_class_can_be_constructed() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let ns = ctx
        .eval_module(
            "export default class Scene {
                n = 1;
                bump(by) { this.n += by; return this.n }
            }
            export const version = 3;",
            "scenes/main.js",
        )
        .unwrap();

    assert_eq!(ns.get("version").unwrap().as_i32(), Some(3));

    let class = ns.get("default").unwrap();
    assert!(class.is_function());

    let scene = class.construct(&[]).unwrap();
    let bump = scene.get("bump").unwrap();
    let n = bump.call_with(&scene, &[ctx.i32(41)]).unwrap();
    assert_eq!(n.as_i32(), Some(42));
}

#[test]
fn imports_resolve_relative_to_the_importer() {
    let rt = Runtime::new().unwrap();
    rt.set_module_loader(files(&[
        (
            "scenes/player.js",
            "import { speed } from '../lib/consts.js'; export class Player { speed = speed }",
        ),
        ("lib/consts.js", "export const speed = 300;"),
    ]));
    let ctx = Context::new(&rt).unwrap();

    let ns = ctx
        .eval_module(
            "import { Player } from './player.js'; export default new Player().speed;",
            "scenes/main.js",
        )
        .unwrap();
    assert_eq!(ns.get("default").unwrap().as_i32(), Some(300));

    let Err(Error::Exception(e)) = ctx.eval_module("import './missing.js'", "scenes/other.js")
    else {
        panic!("expected a load error");
    };
    assert!(
        e.message
            .contains("could not load module 'scenes/missing.js': no such file"),
        "{}",
        e.message
    );
}

#[test]
fn module_errors_surface() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let Err(Error::Exception(e)) =
        ctx.eval_module("export default 1; throw new Error('top')", "m.js")
    else {
        panic!("expected the module to throw");
    };
    assert_eq!(e.message, "Error: top");

    let Err(Error::Exception(e)) = ctx.eval_module(
        "await Promise.resolve(); throw new TypeError('after await')",
        "tla.js",
    ) else {
        panic!("expected the module to throw");
    };
    assert_eq!(e.message, "TypeError: after await");

    let Err(Error::Exception(e)) = ctx.eval_module("export default {", "bad.js") else {
        panic!("expected a syntax error");
    };
    assert!(e.message.starts_with("SyntaxError"));

    // A plain `import` with no loader set fails cleanly.
    let Err(Error::Exception(e)) = ctx.eval_module("import './x.js'", "n.js") else {
        panic!("expected a load error");
    };
    assert!(e.message.contains("could not load module"), "{}", e.message);

    // Still usable afterwards.
    let ns = ctx.eval_module("export default 7", "ok.js").unwrap();
    assert_eq!(ns.get("default").unwrap().as_i32(), Some(7));
}

#[test]
fn persistent_values_outlive_borrows() {
    let rt = Runtime::new().unwrap();
    let ctx = Context::new(&rt).unwrap();

    let kept = {
        let obj = ctx.eval("({ hp: 10 })", "<test>").unwrap();
        ctx.persist(obj)
    };

    rt.run_gc();
    assert_eq!(kept.get(&ctx).get("hp").unwrap().as_i32(), Some(10));
}
