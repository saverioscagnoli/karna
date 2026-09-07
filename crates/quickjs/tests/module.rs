//! Tests for the parts of the binding that do not need a running engine: the
//! `karna` module's value types and namespaces.
//!
//! Everything that borrows from the engine — `ctx` and `draw` — needs a window
//! and a GPU device, so it is exercised by `examples/js` instead.

use quickjs::KarnaModule;
use quickjs::MODULE_NAME;
use rquickjs::CatchResultExt;
use rquickjs::Context;
use rquickjs::Module;
use rquickjs::Runtime;
use rquickjs::loader::BuiltinResolver;
use rquickjs::loader::ModuleLoader;

/// Evaluates `source` as a module with `karna` importable, and fails the test
/// with the JS stack if it throws.
fn eval(source: &str) {
    let runtime = Runtime::new().unwrap();

    runtime.set_loader(
        BuiltinResolver::default().with_module(MODULE_NAME),
        ModuleLoader::default().with_module(MODULE_NAME, KarnaModule),
    );

    let context = Context::full(&runtime).unwrap();

    context.with(|ctx| {
        let prelude = "
            function assert(cond, what) {
                if (!cond) throw new Error(`assertion failed: ${what}`);
            }

            function throws(fn, what) {
                try {
                    fn();
                } catch (e) {
                    return;
                }

                throw new Error(`expected a throw: ${what}`);
            }
        ";

        let source = format!("{prelude}\n{source}");

        let res = Module::declare(ctx.clone(), "test", source)
            .and_then(|m| m.eval())
            .and_then(|(_, promise)| promise.finish::<()>());

        if let Err(e) = res.catch(&ctx) {
            panic!("{e}");
        }
    });
}

#[test]
fn vec2_arithmetic() {
    eval(
        r##"
        import { Vec2, vec2 } from "karna";

        assert(vec2(3, 4).length() === 5, "length");
        assert(vec2(1, 2).add(vec2(3, 4)).x === 4, "add");
        assert(vec2(3, 4).sub(vec2(1, 1)).y === 3, "sub");
        assert(vec2(2, 3).mul(vec2(3, 3)).x === 6, "componentwise mul");
        assert(vec2(6, 9).div(vec2(3, 3)).y === 3, "div");
        assert(vec2(1, 2).scale(3).y === 6, "scale");
        assert(vec2(1, 2).neg().x === -1, "neg");
        assert(vec2(1, 0).dot(vec2(0, 1)) === 0, "dot");
        assert(vec2(0, 0).distance(vec2(3, 4)) === 5, "distance");
        assert(vec2(0, 0).lerp(vec2(10, 10), 0.5).x === 5, "lerp");
        assert(vec2(1, 1).eq(vec2(1, 1)), "eq");
        assert(!vec2(1, 1).eq(vec2(1, 2)), "eq is not always true");

        assert(Vec2.zero().x === 0, "Vec2.zero");
        assert(Vec2.one().y === 1, "Vec2.one");
        assert(Vec2.splat(4).x === 4, "Vec2.splat");
        assert(new Vec2(1, 2) instanceof Vec2, "instanceof");
    "##,
    );
}

#[test]
fn vec2_is_mutable_in_place() {
    eval(
        r##"
        import { vec2 } from "karna";

        const v = vec2(0, 0);

        v.x = 5;
        assert(v.x === 5, "field set");

        v.set(1, 2);
        assert(v.x === 1 && v.y === 2, "set");

        // Operators return new values rather than mutating.
        const w = v.add(vec2(1, 1));
        assert(v.x === 1 && w.x === 2, "add does not mutate");
    "##,
    );
}

#[test]
fn size_and_color() {
    eval(
        r##"
        import { Color, Size, color, size } from "karna";

        assert(size(4, 2).area() === 8, "area");
        assert(size(4, 2).aspectRatio() === 2, "aspect ratio");
        assert(Size.square(3).width === 3, "square");
        assert(Size.zero().height === 0, "zero");

        assert(color(1, 0, 0).r === 1, "rgb");
        assert(color(1, 1, 1).a === 1, "alpha defaults to opaque");
        assert(color(1, 1, 1, 0.5).a === 0.5, "explicit alpha");
        assert(Color.rgb(0, 1, 0).g === 1, "Color.rgb");
        assert(Color.RED.r === 1 && Color.RED.g === 0, "named constant");
        assert(Color.hex("#ff0000").r === 1, "hex string");
        assert(Color.hex(0x00ff00).g === 1, "hex number");
        assert(Color.WHITE.withAlpha(0.25).a === 0.25, "withAlpha");
        throws(() => Color.hex("nonsense"), "a bad hex string");
    "##,
    );
}

#[test]
fn namespaces_are_sealed() {
    eval(
        r##"
        import { Button, Cursor, Key, Layer } from "karna";

        assert(Key.W.name() === "W", "key name");
        assert(Key.Space.name() === "Space", "space is modelled");
        assert(Key.W.eq(Key.W), "keys compare equal");
        assert(!Key.W.eq(Key.A), "distinct keys differ");
        assert(String(Key.W) === "Key.W", "toString");

        assert(Button.Left.name() === "Left", "button name");
        assert(Layer.WORLD.name() === "WORLD", "layer name");
        assert(Cursor.POINTER.name() === "POINTER", "cursor name");

        // A typo must fail where it is written, not somewhere downstream.
        throws(() => Key.Escpae, "an unknown key");
        throws(() => Button.Fourth, "an unknown button");
        throws(() => { Key.W = 1; }, "writing to a namespace");
    "##,
    );
}

#[test]
fn console_is_installed_by_the_scene_loader_only() {
    // `console` comes from the scene loader, not the module, so a bare context
    // does not have it. This pins that split down.
    eval(
        r##"
        assert(typeof globalThis.console === "undefined", "no console here");
    "##,
    );
}
