//! The native `karna` module, reached from a script with
//! `import { Vec2, Key } from "karna"`.
//!
//! Only value types and namespaces live here. Everything that touches the
//! engine arrives as an argument to a scene callback instead, because it is
//! only valid for the length of that call — see [`crate::slot`].

use engine::Color;
use rquickjs::Class;
use rquickjs::Ctx;
use rquickjs::Error;
use rquickjs::FromJs;
use rquickjs::Function;
use rquickjs::IntoJs;
use rquickjs::Object;
use rquickjs::Result;
use rquickjs::function::Opt;
use rquickjs::module::Declarations;
use rquickjs::module::Exports;
use rquickjs::module::ModuleDef;

use crate::enums::button_namespace;
use crate::enums::cursor_namespace;
use crate::enums::key_namespace;
use crate::enums::layer_namespace;
use crate::value::JsColor;
use crate::value::JsSize;
use crate::value::JsVec2;

/// The module's name as scripts spell it in an `import`.
pub const NAME: &str = "karna";

/// Only constructible types are exported as classes. `Text`, `Image` and
/// `Font` come out of the engine and have no constructor to hand a script.
const EXPORTS: &[&str] = &[
    "Vec2", "Size", "Color", "Key", "Button", "Layer", "Cursor", "vec2", "size", "color",
];

const COLORS: &[(&str, Color)] = &[
    ("RED", Color::RED),
    ("GREEN", Color::GREEN),
    ("BLUE", Color::BLUE),
    ("WHITE", Color::WHITE),
    ("BLACK", Color::BLACK),
    ("YELLOW", Color::YELLOW),
    ("CYAN", Color::CYAN),
    ("MAGENTA", Color::MAGENTA),
    ("GRAY", Color::GRAY),
    ("ORANGE", Color::ORANGE),
    ("PURPLE", Color::PURPLE),
    ("BROWN", Color::BROWN),
    ("PINK", Color::PINK),
];

pub struct KarnaModule;

impl ModuleDef for KarnaModule {
    fn declare(decl: &Declarations<'_>) -> Result<()> {
        for name in EXPORTS {
            decl.declare(*name)?;
        }

        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        let color = constructor::<JsColor>(ctx, "Color")?;

        // The named constants hang off the constructor, so `Color.RED` reads
        // like the Rust `Color::RED` it mirrors.
        for (name, value) in COLORS {
            color.set(*name, JsColor::from(*value))?;
        }

        exports.export("Vec2", constructor::<JsVec2>(ctx, "Vec2")?)?;
        exports.export("Size", constructor::<JsSize>(ctx, "Size")?)?;
        exports.export("Color", color)?;

        exports.export("Key", key_namespace(ctx)?)?;
        exports.export("Button", button_namespace(ctx)?)?;
        exports.export("Layer", layer_namespace(ctx)?)?;
        exports.export("Cursor", cursor_namespace(ctx)?)?;

        // Shorthand for the constructors, which are used often enough in
        // gameplay code that `new` everywhere is noise.
        exports.export(
            "vec2",
            Function::new(ctx.clone(), |x: f32, y: f32| JsVec2::new(x, y))?,
        )?;

        exports.export(
            "size",
            Function::new(ctx.clone(), |w: f32, h: f32| JsSize::new(w, h))?,
        )?;

        exports.export(
            "color",
            Function::new(ctx.clone(), |r: f32, g: f32, b: f32, a: Opt<f32>| {
                JsColor::new(r, g, b, a)
            })?,
        )?;

        Ok(())
    }
}

/// The class's constructor object, as a plain [`Object`] so properties can be
/// hung off it.
fn constructor<'js, C>(ctx: &Ctx<'js>, name: &str) -> Result<Object<'js>>
where
    C: rquickjs::class::JsClass<'js>,
{
    let ctor = Class::<C>::create_constructor(ctx)?
        .ok_or_else(|| Error::new_loading_message(NAME, format!("{name} has no constructor")))?;

    Object::from_js(ctx, ctor.into_js(ctx)?)
}
