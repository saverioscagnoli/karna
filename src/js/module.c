#include "js/js.h"

// The native `karna` module. Everything a script imports by name comes from
// here; there is no ambient global beyond `console`, so a module's imports say
// exactly what it touches.

static JSValue vec2_helper(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;

    double x = 0, y = 0;

    if (argc > 0 && JS_ToFloat64(ctx, &x, argv[0]))
        return JS_EXCEPTION;

    if (argc > 1 && JS_ToFloat64(ctx, &y, argv[1]))
        return JS_EXCEPTION;

    return js_new_vec2(ctx, vec2((f32)x, (f32)y));
}

static JSValue size_helper(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;

    double width = 0, height = 0;

    if (argc > 0 && JS_ToFloat64(ctx, &width, argv[0]))
        return JS_EXCEPTION;

    if (argc > 1 && JS_ToFloat64(ctx, &height, argv[1]))
        return JS_EXCEPTION;

    return js_new_size(ctx, size((f32)width, (f32)height));
}

static JSValue color_helper(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;

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

static const char *const EXPORTS[] = {
    "Vec2", "Size", "Color", "vec2", "size", "color", "Key", "Button", "Cursor",
};

static int karna_module_init(JSContext *ctx, JSModuleDef *m) {
    JS_SetModuleExport(ctx, m, "Vec2", js_vec2_constructor(ctx));
    JS_SetModuleExport(ctx, m, "Size", js_size_constructor(ctx));
    JS_SetModuleExport(ctx, m, "Color", js_color_constructor(ctx));

    JS_SetModuleExport(ctx, m, "vec2", JS_NewCFunction(ctx, vec2_helper, "vec2", 2));
    JS_SetModuleExport(ctx, m, "size", JS_NewCFunction(ctx, size_helper, "size", 2));
    JS_SetModuleExport(ctx, m, "color", JS_NewCFunction(ctx, color_helper, "color", 4));

    JS_SetModuleExport(ctx, m, "Key", js_key_namespace(ctx));
    JS_SetModuleExport(ctx, m, "Button", js_button_namespace(ctx));
    JS_SetModuleExport(ctx, m, "Cursor", js_cursor_namespace(ctx));

    return 0;
}

JSModuleDef *js_karna_module(JSContext *ctx) {
    JSModuleDef *m = JS_NewCModule(ctx, "karna", karna_module_init);

    if (!m)
        return NULL;

    for (usize i = 0; i < sizeof(EXPORTS) / sizeof(*EXPORTS); i++)
        JS_AddModuleExport(ctx, m, EXPORTS[i]);

    return m;
}
