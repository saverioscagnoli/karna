#include <math.h>
#include <string.h>

#include <karna/color.h>
#include <karna/math.h>

#include "js/js.h"

// Vec2, Size and Color are the only values a script may hold on to past a
// callback, so they own their data outright rather than borrowing the engine's.
// Everything else the bindings hand out is a token or a per-frame view.

static JSClassID vec2_class_id;
static JSClassID size_class_id;
static JSClassID color_class_id;

#define COUNT_OF(a) (sizeof(a) / sizeof(*(a)))

static void free_opaque(JSRuntime *rt, JSValue value, JSClassID class_id) {
    void *data = JS_GetOpaque(value, class_id);

    if (data)
        js_free_rt(rt, data);
}

// --- Vec2 -------------------------------------------------------------------

static void vec2_finalizer(JSRuntime *rt, JSValue value) {
    free_opaque(rt, value, vec2_class_id);
}

static JSClassDef vec2_class = {"Vec2", .finalizer = vec2_finalizer};

JSValue js_new_vec2(JSContext *ctx, Vec2 v) {
    JSValue obj = JS_NewObjectClass(ctx, vec2_class_id);

    if (JS_IsException(obj))
        return obj;

    Vec2 *data = js_malloc(ctx, sizeof(Vec2));

    if (!data) {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }

    *data = v;
    JS_SetOpaque(obj, data);

    return obj;
}

static Vec2 *vec2_of(JSContext *ctx, JSValueConst value) {
    return JS_GetOpaque2(ctx, value, vec2_class_id);
}

bool js_get_vec2(JSContext *ctx, JSValueConst value, Vec2 *out) {
    Vec2 *v = JS_GetOpaque(value, vec2_class_id);

    if (!v) {
        JS_ThrowTypeError(ctx, "expected a Vec2");
        return false;
    }

    *out = *v;

    return true;
}

static JSValue vec2_ctor(JSContext *ctx, JSValueConst target, int argc, JSValueConst *argv) {
    (void)target;

    double x = 0, y = 0;

    if (argc > 0 && JS_ToFloat64(ctx, &x, argv[0]))
        return JS_EXCEPTION;

    if (argc > 1 && JS_ToFloat64(ctx, &y, argv[1]))
        return JS_EXCEPTION;

    return js_new_vec2(ctx, vec2((f32)x, (f32)y));
}

static JSValue vec2_get_xy(JSContext *ctx, JSValueConst this_val, int magic) {
    Vec2 *v = vec2_of(ctx, this_val);

    if (!v)
        return JS_EXCEPTION;

    return JS_NewFloat64(ctx, magic == 0 ? v->x : v->y);
}

static JSValue vec2_set_xy(JSContext *ctx, JSValueConst this_val, JSValueConst value, int magic) {
    Vec2 *v = vec2_of(ctx, this_val);

    if (!v)
        return JS_EXCEPTION;

    double n;

    if (JS_ToFloat64(ctx, &n, value))
        return JS_EXCEPTION;

    if (magic == 0)
        v->x = (f32)n;
    else
        v->y = (f32)n;

    return JS_UNDEFINED;
}

enum {
    VEC2_ADD,
    VEC2_SUB,
    VEC2_MUL,
    VEC2_DIV,
    VEC2_DOT,
    VEC2_DISTANCE,
    VEC2_EQ,
};

// The binary operators all read one Vec2 argument, so they share a body and
// differ only in what they do with it.
static JSValue vec2_binary(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                           int magic) {
    (void)argc;

    Vec2 *self = vec2_of(ctx, this_val);

    if (!self)
        return JS_EXCEPTION;

    Vec2 other;

    if (!js_get_vec2(ctx, argv[0], &other))
        return JS_EXCEPTION;

    switch (magic) {
    case VEC2_ADD: return js_new_vec2(ctx, vec2_add(*self, other));
    case VEC2_SUB: return js_new_vec2(ctx, vec2_sub(*self, other));
    case VEC2_MUL: return js_new_vec2(ctx, vec2_mul(*self, other));
    case VEC2_DIV: return js_new_vec2(ctx, vec2_div(*self, other));
    case VEC2_DOT: return JS_NewFloat64(ctx, vec2_dot(*self, other));
    case VEC2_DISTANCE: return JS_NewFloat64(ctx, vec2_distance(*self, other));
    default: return JS_NewBool(ctx, vec2_eq(*self, other));
    }
}

enum {
    VEC2_LENGTH,
    VEC2_LENGTH_SQ,
    VEC2_ANGLE,
    VEC2_NORMALIZE,
    VEC2_PERP,
    VEC2_NEG,
    VEC2_CLONE,
};

static JSValue vec2_unary(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                          int magic) {
    (void)argc;
    (void)argv;

    Vec2 *self = vec2_of(ctx, this_val);

    if (!self)
        return JS_EXCEPTION;

    switch (magic) {
    case VEC2_LENGTH: return JS_NewFloat64(ctx, vec2_length(*self));
    case VEC2_LENGTH_SQ: return JS_NewFloat64(ctx, vec2_length_sq(*self));
    case VEC2_ANGLE: return JS_NewFloat64(ctx, vec2_angle(*self));
    case VEC2_NORMALIZE: return js_new_vec2(ctx, vec2_normalize(*self));
    case VEC2_PERP: return js_new_vec2(ctx, vec2_perp(*self));
    case VEC2_NEG: return js_new_vec2(ctx, vec2_neg(*self));
    default: return js_new_vec2(ctx, *self);
    }
}

static JSValue vec2_set(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    Vec2 *self = vec2_of(ctx, this_val);

    if (!self)
        return JS_EXCEPTION;

    double x = 0, y = 0;

    if (argc > 0 && JS_ToFloat64(ctx, &x, argv[0]))
        return JS_EXCEPTION;

    if (argc > 1 && JS_ToFloat64(ctx, &y, argv[1]))
        return JS_EXCEPTION;

    self->x = (f32)x;
    self->y = (f32)y;

    return JS_UNDEFINED;
}

static JSValue js_vec2_scale(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;

    Vec2 *self = vec2_of(ctx, this_val);

    if (!self)
        return JS_EXCEPTION;

    double factor;

    if (JS_ToFloat64(ctx, &factor, argv[0]))
        return JS_EXCEPTION;

    return js_new_vec2(ctx, vec2_scale(*self, (f32)factor));
}

static JSValue js_vec2_rotate(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;

    Vec2 *self = vec2_of(ctx, this_val);

    if (!self)
        return JS_EXCEPTION;

    double angle;

    if (JS_ToFloat64(ctx, &angle, argv[0]))
        return JS_EXCEPTION;

    return js_new_vec2(ctx, vec2_rotate(*self, (f32)angle));
}

static JSValue js_vec2_lerp(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;

    Vec2 *self = vec2_of(ctx, this_val);

    if (!self)
        return JS_EXCEPTION;

    Vec2 other;

    if (!js_get_vec2(ctx, argv[0], &other))
        return JS_EXCEPTION;

    double t;

    if (JS_ToFloat64(ctx, &t, argv[1]))
        return JS_EXCEPTION;

    return js_new_vec2(ctx, vec2_lerp(*self, other, (f32)t));
}

static JSValue vec2_to_string(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;
    (void)argv;

    Vec2 *self = vec2_of(ctx, this_val);

    if (!self)
        return JS_EXCEPTION;

    char buf[64];
    snprintf(buf, sizeof(buf), "Vec2(%g, %g)", (double)self->x, (double)self->y);

    return JS_NewString(ctx, buf);
}

enum { VEC2_ZERO, VEC2_ONE };

static JSValue vec2_static_const(JSContext *ctx, JSValueConst this_val, int argc,
                                 JSValueConst *argv, int magic) {
    (void)this_val;
    (void)argc;
    (void)argv;

    return js_new_vec2(ctx, magic == VEC2_ZERO ? vec2_zero() : vec2_one());
}

static JSValue js_vec2_splat(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    double v;

    if (JS_ToFloat64(ctx, &v, argv[0]))
        return JS_EXCEPTION;

    return js_new_vec2(ctx, vec2_splat((f32)v));
}

static JSValue js_vec2_from_angle(JSContext *ctx, JSValueConst this_val, int argc,
                               JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    double angle;

    if (JS_ToFloat64(ctx, &angle, argv[0]))
        return JS_EXCEPTION;

    return js_new_vec2(ctx, vec2_from_angle((f32)angle));
}

static const JSCFunctionListEntry vec2_proto[] = {
    JS_CGETSET_MAGIC_DEF("x", vec2_get_xy, vec2_set_xy, 0),
    JS_CGETSET_MAGIC_DEF("y", vec2_get_xy, vec2_set_xy, 1),
    JS_CFUNC_DEF("set", 2, vec2_set),
    JS_CFUNC_DEF("scale", 1, js_vec2_scale),
    JS_CFUNC_DEF("rotate", 1, js_vec2_rotate),
    JS_CFUNC_DEF("lerp", 2, js_vec2_lerp),
    JS_CFUNC_DEF("toString", 0, vec2_to_string),
    JS_CFUNC_MAGIC_DEF("add", 1, vec2_binary, VEC2_ADD),
    JS_CFUNC_MAGIC_DEF("sub", 1, vec2_binary, VEC2_SUB),
    JS_CFUNC_MAGIC_DEF("mul", 1, vec2_binary, VEC2_MUL),
    JS_CFUNC_MAGIC_DEF("div", 1, vec2_binary, VEC2_DIV),
    JS_CFUNC_MAGIC_DEF("dot", 1, vec2_binary, VEC2_DOT),
    JS_CFUNC_MAGIC_DEF("distance", 1, vec2_binary, VEC2_DISTANCE),
    JS_CFUNC_MAGIC_DEF("eq", 1, vec2_binary, VEC2_EQ),
    JS_CFUNC_MAGIC_DEF("length", 0, vec2_unary, VEC2_LENGTH),
    JS_CFUNC_MAGIC_DEF("lengthSq", 0, vec2_unary, VEC2_LENGTH_SQ),
    JS_CFUNC_MAGIC_DEF("angle", 0, vec2_unary, VEC2_ANGLE),
    JS_CFUNC_MAGIC_DEF("normalize", 0, vec2_unary, VEC2_NORMALIZE),
    JS_CFUNC_MAGIC_DEF("perp", 0, vec2_unary, VEC2_PERP),
    JS_CFUNC_MAGIC_DEF("neg", 0, vec2_unary, VEC2_NEG),
    JS_CFUNC_MAGIC_DEF("clone", 0, vec2_unary, VEC2_CLONE),
};

static const JSCFunctionListEntry vec2_static[] = {
    JS_CFUNC_MAGIC_DEF("zero", 0, vec2_static_const, VEC2_ZERO),
    JS_CFUNC_MAGIC_DEF("one", 0, vec2_static_const, VEC2_ONE),
    JS_CFUNC_DEF("splat", 1, js_vec2_splat),
    JS_CFUNC_DEF("fromAngle", 1, js_vec2_from_angle),
};

// --- Size -------------------------------------------------------------------

static void size_finalizer(JSRuntime *rt, JSValue value) {
    free_opaque(rt, value, size_class_id);
}

static JSClassDef size_class = {"Size", .finalizer = size_finalizer};

JSValue js_new_size(JSContext *ctx, Size s) {
    JSValue obj = JS_NewObjectClass(ctx, size_class_id);

    if (JS_IsException(obj))
        return obj;

    Size *data = js_malloc(ctx, sizeof(Size));

    if (!data) {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }

    *data = s;
    JS_SetOpaque(obj, data);

    return obj;
}

static Size *size_of(JSContext *ctx, JSValueConst value) {
    return JS_GetOpaque2(ctx, value, size_class_id);
}

static JSValue size_ctor(JSContext *ctx, JSValueConst target, int argc, JSValueConst *argv) {
    (void)target;

    double width = 0, height = 0;

    if (argc > 0 && JS_ToFloat64(ctx, &width, argv[0]))
        return JS_EXCEPTION;

    if (argc > 1 && JS_ToFloat64(ctx, &height, argv[1]))
        return JS_EXCEPTION;

    return js_new_size(ctx, size((f32)width, (f32)height));
}

static JSValue size_get_wh(JSContext *ctx, JSValueConst this_val, int magic) {
    Size *s = size_of(ctx, this_val);

    if (!s)
        return JS_EXCEPTION;

    return JS_NewFloat64(ctx, magic == 0 ? s->width : s->height);
}

static JSValue size_set_wh(JSContext *ctx, JSValueConst this_val, JSValueConst value, int magic) {
    Size *s = size_of(ctx, this_val);

    if (!s)
        return JS_EXCEPTION;

    double n;

    if (JS_ToFloat64(ctx, &n, value))
        return JS_EXCEPTION;

    if (magic == 0)
        s->width = (f32)n;
    else
        s->height = (f32)n;

    return JS_UNDEFINED;
}

enum { SIZE_CLONE, SIZE_AREA, SIZE_ASPECT };

static JSValue size_unary(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                          int magic) {
    (void)argc;
    (void)argv;

    Size *s = size_of(ctx, this_val);

    if (!s)
        return JS_EXCEPTION;

    switch (magic) {
    case SIZE_AREA: return JS_NewFloat64(ctx, size_area(*s));
    case SIZE_ASPECT: return JS_NewFloat64(ctx, size_aspect_ratio(*s));
    default: return js_new_size(ctx, *s);
    }
}

static JSValue size_scale_method(JSContext *ctx, JSValueConst this_val, int argc,
                                 JSValueConst *argv) {
    (void)argc;

    Size *s = size_of(ctx, this_val);

    if (!s)
        return JS_EXCEPTION;

    double factor;

    if (JS_ToFloat64(ctx, &factor, argv[0]))
        return JS_EXCEPTION;

    return js_new_size(ctx, size_scale(*s, (f32)factor));
}

static JSValue size_eq_method(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;

    Size *s = size_of(ctx, this_val);
    Size *other = size_of(ctx, argv[0]);

    if (!s || !other)
        return JS_EXCEPTION;

    return JS_NewBool(ctx, size_eq(*s, *other));
}

static JSValue size_to_string(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;
    (void)argv;

    Size *s = size_of(ctx, this_val);

    if (!s)
        return JS_EXCEPTION;

    char buf[64];
    snprintf(buf, sizeof(buf), "Size(%g, %g)", (double)s->width, (double)s->height);

    return JS_NewString(ctx, buf);
}

static JSValue size_zero(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;
    (void)argc;
    (void)argv;

    return js_new_size(ctx, size(0.0f, 0.0f));
}

static JSValue size_square(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    double side;

    if (JS_ToFloat64(ctx, &side, argv[0]))
        return JS_EXCEPTION;

    return js_new_size(ctx, size((f32)side, (f32)side));
}

static const JSCFunctionListEntry size_proto[] = {
    JS_CGETSET_MAGIC_DEF("width", size_get_wh, size_set_wh, 0),
    JS_CGETSET_MAGIC_DEF("height", size_get_wh, size_set_wh, 1),
    JS_CFUNC_DEF("scale", 1, size_scale_method),
    JS_CFUNC_DEF("eq", 1, size_eq_method),
    JS_CFUNC_DEF("toString", 0, size_to_string),
    JS_CFUNC_MAGIC_DEF("clone", 0, size_unary, SIZE_CLONE),
    JS_CFUNC_MAGIC_DEF("area", 0, size_unary, SIZE_AREA),
    JS_CFUNC_MAGIC_DEF("aspectRatio", 0, size_unary, SIZE_ASPECT),
};

static const JSCFunctionListEntry size_static[] = {
    JS_CFUNC_DEF("zero", 0, size_zero),
    JS_CFUNC_DEF("square", 1, size_square),
};

// --- Color ------------------------------------------------------------------

static void color_finalizer(JSRuntime *rt, JSValue value) {
    free_opaque(rt, value, color_class_id);
}

static JSClassDef color_class = {"Color", .finalizer = color_finalizer};

JSValue js_new_color(JSContext *ctx, Color c) {
    JSValue obj = JS_NewObjectClass(ctx, color_class_id);

    if (JS_IsException(obj))
        return obj;

    Color *data = js_malloc(ctx, sizeof(Color));

    if (!data) {
        JS_FreeValue(ctx, obj);
        return JS_EXCEPTION;
    }

    *data = c;
    JS_SetOpaque(obj, data);

    return obj;
}

static Color *color_of(JSContext *ctx, JSValueConst value) {
    return JS_GetOpaque2(ctx, value, color_class_id);
}

bool js_get_color(JSContext *ctx, JSValueConst value, Color *out) {
    Color *c = JS_GetOpaque(value, color_class_id);

    if (c) {
        *out = *c;
        return true;
    }

    // A hex string is the shorthand a script reaches for most, so it is worth
    // accepting anywhere a Color is: draw.setColor("#f38ba8").
    if (JS_IsString(value)) {
        const char *str = JS_ToCString(ctx, value);

        if (!str)
            return false;

        bool ok = color_parse(str, out);

        if (!ok)
            JS_ThrowTypeError(ctx, "'%s' is not a hex color", str);

        JS_FreeCString(ctx, str);

        return ok;
    }

    if (JS_IsNumber(value)) {
        uint32_t hex;

        if (JS_ToUint32(ctx, &hex, value))
            return false;

        *out = color_hex(hex);

        return true;
    }

    JS_ThrowTypeError(ctx, "expected a Color, a hex string or a hex number");

    return false;
}

static JSValue color_ctor(JSContext *ctx, JSValueConst target, int argc, JSValueConst *argv) {
    (void)target;

    double r = 0, g = 0, b = 0, a = 1;

    if (argc > 0 && JS_ToFloat64(ctx, &r, argv[0]))
        return JS_EXCEPTION;

    if (argc > 1 && JS_ToFloat64(ctx, &g, argv[1]))
        return JS_EXCEPTION;

    if (argc > 2 && JS_ToFloat64(ctx, &b, argv[2]))
        return JS_EXCEPTION;

    if (argc > 3 && JS_ToFloat64(ctx, &a, argv[3]))
        return JS_EXCEPTION;

    return js_new_color(ctx, color_rgba((f32)r, (f32)g, (f32)b, (f32)a));
}

static JSValue color_get_channel(JSContext *ctx, JSValueConst this_val, int magic) {
    Color *c = color_of(ctx, this_val);

    if (!c)
        return JS_EXCEPTION;

    return JS_NewFloat64(ctx, (&c->r)[magic]);
}

static JSValue color_set_channel(JSContext *ctx, JSValueConst this_val, JSValueConst value,
                                 int magic) {
    Color *c = color_of(ctx, this_val);

    if (!c)
        return JS_EXCEPTION;

    double n;

    if (JS_ToFloat64(ctx, &n, value))
        return JS_EXCEPTION;

    (&c->r)[magic] = (f32)n;

    return JS_UNDEFINED;
}

static JSValue color_clone(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;
    (void)argv;

    Color *c = color_of(ctx, this_val);

    if (!c)
        return JS_EXCEPTION;

    return js_new_color(ctx, *c);
}

static JSValue color_with_alpha_method(JSContext *ctx, JSValueConst this_val, int argc,
                                       JSValueConst *argv) {
    (void)argc;

    Color *c = color_of(ctx, this_val);

    if (!c)
        return JS_EXCEPTION;

    double a;

    if (JS_ToFloat64(ctx, &a, argv[0]))
        return JS_EXCEPTION;

    return js_new_color(ctx, color_with_alpha(*c, (f32)a));
}

static JSValue color_eq_method(JSContext *ctx, JSValueConst this_val, int argc,
                               JSValueConst *argv) {
    (void)argc;

    Color *c = color_of(ctx, this_val);

    if (!c)
        return JS_EXCEPTION;

    Color other;

    if (!js_get_color(ctx, argv[0], &other))
        return JS_EXCEPTION;

    return JS_NewBool(ctx, color_eq(*c, other));
}

static JSValue color_to_string(JSContext *ctx, JSValueConst this_val, int argc,
                               JSValueConst *argv) {
    (void)argc;
    (void)argv;

    Color *c = color_of(ctx, this_val);

    if (!c)
        return JS_EXCEPTION;

    char buf[96];
    snprintf(buf, sizeof(buf), "Color(%g, %g, %g, %g)", (double)c->r, (double)c->g, (double)c->b,
             (double)c->a);

    return JS_NewString(ctx, buf);
}

static JSValue color_rgb_static(JSContext *ctx, JSValueConst this_val, int argc,
                                JSValueConst *argv, int magic) {
    (void)this_val;

    double channels[4] = {0, 0, 0, 1};
    int wanted = magic ? 4 : 3;

    for (int i = 0; i < wanted && i < argc; i++)
        if (JS_ToFloat64(ctx, &channels[i], argv[i]))
            return JS_EXCEPTION;

    return js_new_color(
        ctx, color_rgba((f32)channels[0], (f32)channels[1], (f32)channels[2], (f32)channels[3]));
}

static JSValue color_hex_static(JSContext *ctx, JSValueConst this_val, int argc,
                                JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    Color out;

    if (!js_get_color(ctx, argv[0], &out))
        return JS_EXCEPTION;

    return js_new_color(ctx, out);
}

static const JSCFunctionListEntry color_proto[] = {
    JS_CGETSET_MAGIC_DEF("r", color_get_channel, color_set_channel, 0),
    JS_CGETSET_MAGIC_DEF("g", color_get_channel, color_set_channel, 1),
    JS_CGETSET_MAGIC_DEF("b", color_get_channel, color_set_channel, 2),
    JS_CGETSET_MAGIC_DEF("a", color_get_channel, color_set_channel, 3),
    JS_CFUNC_DEF("clone", 0, color_clone),
    JS_CFUNC_DEF("withAlpha", 1, color_with_alpha_method),
    JS_CFUNC_DEF("eq", 1, color_eq_method),
    JS_CFUNC_DEF("toString", 0, color_to_string),
};

static const JSCFunctionListEntry color_static[] = {
    JS_CFUNC_MAGIC_DEF("rgb", 3, color_rgb_static, 0),
    JS_CFUNC_MAGIC_DEF("rgba", 4, color_rgb_static, 1),
    JS_CFUNC_DEF("hex", 1, color_hex_static),
};

// --- registration -----------------------------------------------------------

static JSValue define_class(JSContext *ctx, JSClassID class_id, JSClassDef *def,
                            const char *name, JSCFunction *ctor_fn, int arg_count,
                            const JSCFunctionListEntry *proto_funcs, usize proto_count,
                            const JSCFunctionListEntry *static_funcs, usize static_count) {
    JS_NewClass(JS_GetRuntime(ctx), class_id, def);

    JSValue proto = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(ctx, proto, proto_funcs, (int)proto_count);
    JS_SetClassProto(ctx, class_id, proto);

    JSValue ctor =
        JS_NewCFunction2(ctx, ctor_fn, name, arg_count, JS_CFUNC_constructor, 0);

    JS_SetConstructor(ctx, ctor, proto);

    if (static_count)
        JS_SetPropertyFunctionList(ctx, ctor, static_funcs, (int)static_count);

    return ctor;
}

void js_register_classes(JSContext *ctx) {
    JSRuntime *rt = JS_GetRuntime(ctx);

    JS_NewClassID(rt, &vec2_class_id);
    JS_NewClassID(rt, &size_class_id);
    JS_NewClassID(rt, &color_class_id);

    JSValue ctor = define_class(ctx, vec2_class_id, &vec2_class, "Vec2", vec2_ctor, 2, vec2_proto,
                                COUNT_OF(vec2_proto), vec2_static, COUNT_OF(vec2_static));
    JS_FreeValue(ctx, ctor);

    ctor = define_class(ctx, size_class_id, &size_class, "Size", size_ctor, 2, size_proto,
                        COUNT_OF(size_proto), size_static, COUNT_OF(size_static));
    JS_FreeValue(ctx, ctor);

    ctor = define_class(ctx, color_class_id, &color_class, "Color", color_ctor, 4, color_proto,
                        COUNT_OF(color_proto), color_static, COUNT_OF(color_static));

    // Color.RED and friends live on the constructor, frozen, so a script cannot
    // reassign one out from under another module.
    for (usize i = 0; i < COLOR_NAMED_COUNT; i++) {
        Color value;
        const char *name = color_named_at(i, &value);

        JS_DefinePropertyValueStr(ctx, ctor, name, js_new_color(ctx, value), 0);
    }

    JS_FreeValue(ctx, ctor);

    js_register_token_classes(ctx);
}

// The module exports the constructors themselves, and JS_SetClassProto already
// wired proto.constructor, so there is one source of truth for each.
static JSValue constructor_of(JSContext *ctx, JSClassID class_id) {
    JSValue proto = JS_GetClassProto(ctx, class_id);
    JSValue ctor = JS_GetPropertyStr(ctx, proto, "constructor");

    JS_FreeValue(ctx, proto);

    return ctor;
}

JSValue js_vec2_constructor(JSContext *ctx) {
    return constructor_of(ctx, vec2_class_id);
}

JSValue js_size_constructor(JSContext *ctx) {
    return constructor_of(ctx, size_class_id);
}

JSValue js_color_constructor(JSContext *ctx) {
    return constructor_of(ctx, color_class_id);
}
