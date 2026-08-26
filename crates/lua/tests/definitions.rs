//! Guards `types/karna.lua` against drift.
//!
//! The definition file is a second source of truth: nothing forces it to match
//! the bindings, so adding a method in Rust and forgetting the stub would
//! silently ship a wrong API to every user's editor. This test parses both
//! sides and fails if they disagree.
//!
//! Classes in the `.lua` file opt in by tagging themselves `@rust <Type>`.
//! Metamethods (operators) are excluded — they are documented with `@operator`,
//! which has no one-to-one mapping to a name.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

const SOURCES: &[(&str, &str)] = &[
    ("context.rs", include_str!("../src/context.rs")),
    ("refs.rs", include_str!("../src/refs.rs")),
    ("value.rs", include_str!("../src/value.rs")),
    ("enums.rs", include_str!("../src/enums.rs")),
];

const DEFS: &str = include_str!("../types/karna.lua");

/// Everything bound in Rust, keyed by the Rust type name.
fn bound() -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut current: Option<String> = None;

    for (_file, src) in SOURCES {
        let lines: Vec<&str> = src.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            if t.starts_with("//") {
                continue; // doc comments mention these forms verbatim
            }

            if let Some(ty) = between(t, "register_userdata_type::<", ">")
                .or_else(|| after(t, "impl UserData for ").map(|s| s.trim_end_matches(" {").into()))
            {
                // Skip macro metavariables (`impl UserData for $name`) — the
                // concrete types come from the macro's invocation instead.
                if ty.starts_with('$') {
                    current = None;
                    continue;
                }

                current = Some(ty);
                out.entry(current.clone().unwrap()).or_default();
                continue;
            }

            // `enums.rs` declares its types through the `opaque!` macro:
            //     LuaKey(Key) => "key", |k| format!("{k:?}"),
            // These carry only metamethods, so they register with no methods.
            if t.contains("=> \"")
                && let Some((head, _)) = t.split_once('(')
                && head.chars().all(|c| c.is_alphanumeric() || c == '_')
                && !head.is_empty()
            {
                out.entry(head.to_string()).or_default();
                continue;
            }

            let Some(ty) = current.as_ref() else { continue };

            for marker in [
                "add_method(",
                "add_method_mut(",
                "add_field_method_get(",
                "add_field_method_set(",
            ] {
                let Some(rest) = after(t, marker) else { continue };

                // rustfmt wraps long registrations, putting the name on the
                // following line: `add_method_mut(\n    "set_custom_cursor",`
                let name = quoted(&rest)
                    .or_else(|| lines.get(i + 1).and_then(|next| quoted(next.trim())));

                if let Some(name) = name {
                    out.entry(ty.clone()).or_default().insert(name);
                }
            }
        }
    }

    out
}

/// Everything documented in the `.lua` stub, keyed by the same Rust type name.
fn documented() -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut locals: BTreeMap<String, String> = BTreeMap::new();

    for line in DEFS.lines() {
        let t = line.trim();

        if t.starts_with("---@class") {
            // Classes with no `@rust` tag (Context, Scene) are pure Lua shapes
            // with no bound counterpart; clear `current` so their `@field`
            // lines are not attributed to the previous class.
            current = after(t, "@rust ").map(|rust| {
                let rust = rust.split_whitespace().next().unwrap_or("").to_string();
                out.entry(rust.clone()).or_default();
                rust
            });
            continue;
        }

        // `---@field x number` inside the current class block
        if t.starts_with("---@field")
            && let Some(rest) = after(t, "---@field ")
            && let Some(ty) = current.as_ref()
            && let Some(name) = rest.split_whitespace().next()
        {
            out.entry(ty.clone()).or_default().insert(name.into());
            continue;
        }

        // `local Vec2 = {}` binds that variable to the class just declared
        if t.starts_with("local ")
            && t.ends_with("= {}")
            && let Some(ty) = current.take()
            && let Some(var) = t.strip_prefix("local ").and_then(|s| s.split_whitespace().next())
        {
            locals.insert(var.into(), ty);
            continue;
        }

        // `function Vec2:length() end`
        if let Some(rest) = after(t, "function ")
            && let Some((var, tail)) = rest.split_once(':')
            && let Some(name) = tail.split('(').next()
            && let Some(ty) = locals.get(var)
        {
            out.entry(ty.clone()).or_default().insert(name.into());
        }
    }

    out
}

fn after(s: &str, pat: &str) -> Option<String> {
    s.find(pat).map(|i| s[i + pat.len()..].to_string())
}

fn between(s: &str, open: &str, close: &str) -> Option<String> {
    let rest = after(s, open)?;
    let end = rest.find(close)?;
    Some(rest[..end].to_string())
}

fn quoted(s: &str) -> Option<String> {
    let s = s.trim_start();
    let s = s.strip_prefix('"')?;
    let end = s.find('"')?;
    Some(s[..end].to_string())
}

#[test]
fn definitions_match_bindings() {
    let bound = bound();
    let documented = documented();
    let mut problems = Vec::new();

    for (ty, methods) in &bound {
        let Some(doc) = documented.get(ty) else {
            problems.push(format!(
                "type `{ty}` is bound but has no `---@class ... @rust {ty}` in types/karna.lua"
            ));
            continue;
        };

        for m in methods.difference(doc) {
            problems.push(format!("`{ty}:{m}` is bound in Rust but not documented"));
        }

        for m in doc.difference(methods) {
            problems.push(format!("`{ty}:{m}` is documented but not bound in Rust"));
        }
    }

    for ty in documented.keys() {
        if !bound.contains_key(ty) {
            problems.push(format!("types/karna.lua documents `@rust {ty}`, which is not bound"));
        }
    }

    assert!(
        problems.is_empty(),
        "types/karna.lua is out of sync with the bindings:\n  {}",
        problems.join("\n  ")
    );
}
