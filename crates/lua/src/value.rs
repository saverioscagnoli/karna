//! Tier-2 bindings: small `Copy` value types that Lua owns outright.
//!
//! These are newtypes rather than direct registrations of `math::Vector2` and
//! friends, so that the blanket `IntoLua for T: UserData` impl applies and a
//! binding can just return one. `FromLua` is spelled out by [`from_lua_ud!`].

use engine::Color;
use math::Size;
use math::Vector2;
use mlua::Lua;
use mlua::MetaMethod;
use mlua::Result;
use mlua::UserData;
use mlua::UserDataFields;
use mlua::UserDataMethods;
use mlua::Value;

/// Accept `T` as an argument by cloning it back out of its userdata.
macro_rules! from_lua_ud {
    ($($t:ty => $name:literal),* $(,)?) => {$(
        impl mlua::FromLua for $t {
            fn from_lua(value: Value, _: &Lua) -> Result<Self> {
                match value {
                    Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
                    other => Err(mlua::Error::FromLuaConversionError { from: other.type_name(), to: ($name).into(), message: None }),
                }
            }
        }
    )*};
}

#[derive(Debug, Clone, Copy)]
pub struct LuaVec2(pub Vector2<f32>);

#[derive(Debug, Clone, Copy)]
pub struct LuaSize(pub Size<f32>);

#[derive(Debug, Clone, Copy)]
pub struct LuaColor(pub Color);

from_lua_ud! {
    LuaVec2  => "vec2",
    LuaSize  => "size",
    LuaColor => "color",
}

impl UserData for LuaVec2 {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("x", |_, v| Ok(v.0.x));
        f.add_field_method_get("y", |_, v| Ok(v.0.y));
        f.add_field_method_set("x", |_, v, n: f32| Ok(v.0.x = n));
        f.add_field_method_set("y", |_, v, n: f32| Ok(v.0.y = n));
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("length", |_, v, ()| Ok(v.0.length()));
        m.add_method("length_sq", |_, v, ()| Ok(v.0.length_sq()));
        m.add_method("normalize", |_, v, ()| Ok(Self(v.0.normalize())));
        m.add_method("perp", |_, v, ()| Ok(Self(v.0.perp())));
        m.add_method("angle", |_, v, ()| Ok(v.0.angle()));
        m.add_method("rotate", |_, v, a: f32| Ok(Self(v.0.rotate(a))));
        m.add_method("dot", |_, v, o: Self| Ok(v.0.dot(&o.0)));
        m.add_method("distance", |_, v, o: Self| Ok(v.0.distance(&o.0)));
        m.add_method("lerp", |_, v, (o, t): (Self, f32)| {
            Ok(Self(v.0.lerp(&o.0, t)))
        });
        m.add_method("unpack", |_, v, ()| Ok((v.0.x, v.0.y)));
        m.add_method("clone", |_, v, ()| Ok(*v));

        m.add_method_mut("set", |_, v, (x, y): (f32, f32)| Ok(v.0.set([x, y])));

        m.add_meta_method(MetaMethod::Add, |_, a, b: Self| Ok(Self(a.0 + b.0)));
        m.add_meta_method(MetaMethod::Sub, |_, a, b: Self| Ok(Self(a.0 - b.0)));
        m.add_meta_method(MetaMethod::Unm, |_, a, ()| {
            Ok(Self(Vector2::new(-a.0.x, -a.0.y)))
        });
        m.add_meta_method(MetaMethod::Eq, |_, a, b: Self| Ok(a.0 == b.0));

        // `v * 2` and `2 * v` both land here; Lua only guarantees that one of
        // the operands carries the metatable, not which.
        m.add_meta_function(MetaMethod::Mul, |_, (a, b): (Value, Value)| {
            mul_like(
                a,
                b,
                |v, s| v * s,
                |a, b| Self(Vector2::new(a.0.x * b.0.x, a.0.y * b.0.y)),
            )
        });

        m.add_meta_function(MetaMethod::Div, |_, (a, b): (Value, Value)| {
            mul_like(
                a,
                b,
                |v, s| v / s,
                |a, b| Self(Vector2::new(a.0.x / b.0.x, a.0.y / b.0.y)),
            )
        });

        m.add_meta_method(MetaMethod::ToString, |_, v, ()| {
            Ok(format!("vec2({}, {})", v.0.x, v.0.y))
        });
    }
}

/// Shared body for `__mul` / `__div`, which must handle scalar on either side.
fn mul_like(
    a: Value,
    b: Value,
    scalar: fn(Vector2<f32>, f32) -> Vector2<f32>,
    component: fn(LuaVec2, LuaVec2) -> LuaVec2,
) -> Result<LuaVec2> {
    match (&a, &b) {
        (Value::UserData(v), Value::Number(_) | Value::Integer(_)) => {
            let v = *v.borrow::<LuaVec2>()?;
            Ok(LuaVec2(scalar(v.0, f32_of(&b)?)))
        }

        // `2 / v` is componentwise on a splatted scalar, matching `2 * v`.
        (Value::Number(_) | Value::Integer(_), Value::UserData(v)) => {
            let v = *v.borrow::<LuaVec2>()?;
            let s = f32_of(&a)?;
            Ok(component(LuaVec2(Vector2::splat(s)), v))
        }

        (Value::UserData(x), Value::UserData(y)) => {
            Ok(component(*x.borrow::<LuaVec2>()?, *y.borrow::<LuaVec2>()?))
        }

        _ => Err(mlua::Error::runtime(
            "vec2 arithmetic expects a vec2 or a number",
        )),
    }
}

/// Lua 5.4 keeps integers and floats distinct, so `v * 2` and `v * 2.0` arrive
/// as different `Value` variants.
fn f32_of(v: &Value) -> Result<f32> {
    match v {
        Value::Number(n) => Ok(*n as f32),
        Value::Integer(i) => Ok(*i as f32),
        other => Err(mlua::Error::runtime(format!(
            "expected a number, got {}",
            other.type_name()
        ))),
    }
}

impl UserData for LuaSize {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("width", |_, s| Ok(s.0.width));
        f.add_field_method_get("height", |_, s| Ok(s.0.height));
        f.add_field_method_set("width", |_, s, n: f32| Ok(s.0.width = n));
        f.add_field_method_set("height", |_, s, n: f32| Ok(s.0.height = n));
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("area", |_, s, ()| Ok(s.0.area()));
        m.add_method("aspect_ratio", |_, s, ()| Ok(s.0.aspect_ratio()));
        m.add_method("scale", |_, s, f: f32| Ok(Self(s.0.scale(f))));
        m.add_method("unpack", |_, s, ()| Ok((s.0.width, s.0.height)));
        m.add_method("clone", |_, s, ()| Ok(*s));

        m.add_meta_method(MetaMethod::Eq, |_, a, b: Self| Ok(a.0 == b.0));
        m.add_meta_method(MetaMethod::ToString, |_, s, ()| {
            Ok(format!("size({}, {})", s.0.width, s.0.height))
        });
    }
}

impl UserData for LuaColor {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("r", |_, c| Ok(c.0.r));
        f.add_field_method_get("g", |_, c| Ok(c.0.g));
        f.add_field_method_get("b", |_, c| Ok(c.0.b));
        f.add_field_method_get("a", |_, c| Ok(c.0.a));
        f.add_field_method_set("r", |_, c, n: f32| Ok(c.0.r = n));
        f.add_field_method_set("g", |_, c, n: f32| Ok(c.0.g = n));
        f.add_field_method_set("b", |_, c, n: f32| Ok(c.0.b = n));
        f.add_field_method_set("a", |_, c, n: f32| Ok(c.0.a = n));
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("unpack", |_, c, ()| Ok(c.0.tuple()));
        m.add_method("clone", |_, c, ()| Ok(*c));

        m.add_method("with_alpha", |_, c, a: f32| {
            Ok(Self(Color::rgba(c.0.r, c.0.g, c.0.b, a)))
        });

        m.add_meta_method(MetaMethod::Eq, |_, a, b: Self| Ok(a.0 == b.0));
        m.add_meta_method(MetaMethod::ToString, |_, c, ()| {
            let (r, g, b, a) = c.0.tuple();
            Ok(format!("color({r}, {g}, {b}, {a})"))
        });
    }
}

/// Coerce whatever Lua passed where a color is expected: a `color` userdata,
/// a `"#rrggbb"` string, or 3-4 numbers already unpacked by the caller.
pub fn color_from(value: Value) -> Result<Color> {
    match value {
        Value::UserData(ud) => Ok(ud.borrow::<LuaColor>()?.0),
        Value::String(s) => Color::try_hex(&s.to_str()?)
            .ok_or_else(|| mlua::Error::runtime(format!("not a hex color: {}", s.display()))),
        Value::Integer(v) => Ok(Color::hex(v as u32)),
        other => Err(mlua::Error::FromLuaConversionError {
            from: other.type_name(),
            to: ("color").into(),
            message: None,
        }),
    }
}
