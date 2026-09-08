#include "js/js.h"

#define COUNT_OF(a) (sizeof(a) / sizeof(*(a)))

// Drawing is only meaningful while a draw callback is on the stack; outside one
// there is no frame open to add geometry to.
static Draw *draw_for(JSContext *ctx) {
    Js *js = js_of(ctx);

    if (!js || !js->app || js->phase != JS_PHASE_DRAW) {
        JS_ThrowTypeError(ctx, "draw is only valid inside a scene's draw callback");
        return NULL;
    }

    return &js->app->draw;
}

static JSValue draw_color_js(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;
    (void)argc;
    (void)argv;

    Draw *draw = draw_for(ctx);

    if (!draw)
        return JS_EXCEPTION;

    return js_new_color(ctx, draw_color(draw));
}

static JSValue draw_set_color_js(JSContext *ctx, JSValueConst this_val, int argc,
                                 JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    Draw *draw = draw_for(ctx);

    if (!draw)
        return JS_EXCEPTION;

    Color color;

    if (!js_get_color(ctx, argv[0], &color))
        return JS_EXCEPTION;

    draw_set_color(draw, color);

    return JS_UNDEFINED;
}

static JSValue draw_viewport_js(JSContext *ctx, JSValueConst this_val, int argc,
                                JSValueConst *argv) {
    (void)this_val;
    (void)argc;
    (void)argv;

    Draw *draw = draw_for(ctx);

    if (!draw)
        return JS_EXCEPTION;

    return js_new_size(ctx, draw_viewport(draw));
}

static JSValue draw_rect_js(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;

    Draw *draw = draw_for(ctx);

    if (!draw)
        return JS_EXCEPTION;

    if (argc < 4)
        return JS_ThrowTypeError(ctx, "rect(x, y, width, height) takes four numbers");

    double values[4];

    for (int i = 0; i < 4; i++)
        if (JS_ToFloat64(ctx, &values[i], argv[i]))
            return JS_EXCEPTION;

    draw_rect(draw, (f32)values[0], (f32)values[1], (f32)values[2], (f32)values[3]);

    return JS_UNDEFINED;
}

static const JSCFunctionListEntry draw_funcs[] = {
    JS_CFUNC_DEF("color", 0, draw_color_js),
    JS_CFUNC_DEF("setColor", 1, draw_set_color_js),
    JS_CFUNC_DEF("viewport", 0, draw_viewport_js),
    JS_CFUNC_DEF("rect", 4, draw_rect_js),
};

JSValue js_new_draw_object(JSContext *ctx) {
    JSValue draw = JS_NewObject(ctx);

    JS_SetPropertyFunctionList(ctx, draw, draw_funcs, COUNT_OF(draw_funcs));
    JS_PreventExtensions(ctx, draw);

    return draw;
}
