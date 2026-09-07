//! `console` for scripts, routed into the engine's logger.
//!
//! Scripts reach for `console.log` on reflex, so it exists, but it goes through
//! `logging` like everything else rather than to stdout — the log line carries
//! the script's path, so it is clear which scene printed it.

use logging::debug;
use logging::error;
use logging::info;
use logging::trace;
use logging::warn;
use rquickjs::Coerced;
use rquickjs::Ctx;
use rquickjs::FromJs;
use rquickjs::Function;
use rquickjs::Object;
use rquickjs::Result;
use rquickjs::Value;
use rquickjs::function::Rest;

/// Installs `console` on the globals of `ctx`, tagged with `source`.
pub fn install(ctx: &Ctx<'_>, source: &str) -> Result<()> {
    let console = Object::new(ctx.clone())?;

    macro_rules! level {
        ($name:literal, $log:ident) => {{
            let source = source.to_string();

            console.set(
                $name,
                Function::new(
                    ctx.clone(),
                    tie(move |ctx, args| $log!("[{}] {}", source, join(&ctx, args))),
                )?,
            )?;
        }};
    }

    level!("log", info);
    level!("info", info);
    level!("warn", warn);
    level!("error", error);
    level!("debug", debug);
    level!("trace", trace);

    ctx.globals().set("console", console)
}

/// Ties the two elided lifetimes of a console callback together.
///
/// A closure written inline gets an independent lifetime per argument, which
/// leaves `Ctx` and the values it is used with unrelated. Passing it through a
/// higher-ranked bound first is the usual way to force the two to agree.
fn tie<F>(f: F) -> F
where
    F: for<'js> Fn(Ctx<'js>, Rest<Value<'js>>) + 'static,
{
    f
}

/// Space-joins the arguments the way `console.log` does, falling back to a type
/// name for anything that will not stringify.
fn join<'js>(ctx: &Ctx<'js>, args: Rest<Value<'js>>) -> String {
    args.0
        .iter()
        .map(|v| stringify(ctx, v))
        .collect::<Vec<_>>()
        .join(" ")
}

fn stringify<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> String {
    // `JSON.stringify` gives readable output for plain objects and arrays,
    // where the default `[object Object]` gives none. It returns undefined for
    // functions and cyclic values, so fall back to coercion.
    if value.is_object()
        && !value.is_function()
        && let Ok(Some(json)) = ctx.json_stringify(value.clone())
        && let Ok(s) = json.to_string()
    {
        return s;
    }

    match Coerced::<String>::from_js(ctx, value.clone()) {
        Ok(s) => s.0,
        Err(_) => value.type_name().into(),
    }
}
