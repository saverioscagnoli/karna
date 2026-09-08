#include <string.h>

#include "js/js.h"

#define COUNT_OF(a) (sizeof(a) / sizeof(*(a)))

// The context object is created once and handed to every callback. What keeps
// it honest is the phase: outside a callback there is no frame to talk about,
// and a script that squirrelled `ctx` away gets an exception rather than a
// stale reading.
static App *app_for(JSContext *ctx) {
    Js *js = js_of(ctx);

    if (!js || !js->app || js->phase == JS_PHASE_NONE) {
        JS_ThrowTypeError(ctx, "the context is only valid inside a scene callback");
        return NULL;
    }

    return js->app;
}

// `draw` gets a read-only window: a resize or a scene switch halfway through
// painting a frame would apply to geometry already submitted.
static App *mutable_app_for(JSContext *ctx, const char *what) {
    Js *js = js_of(ctx);
    App *app = app_for(ctx);

    if (!app)
        return NULL;

    if (js->phase == JS_PHASE_DRAW) {
        JS_ThrowTypeError(ctx, "%s cannot be called from draw", what);
        return NULL;
    }

    return app;
}

// --- window -----------------------------------------------------------------

enum {
    WINDOW_TITLE,
    WINDOW_SIZE,
    WINDOW_WIDTH,
    WINDOW_HEIGHT,
    WINDOW_RESIZABLE,
    WINDOW_MOUSE_POSITION,
    WINDOW_MOUSE_DELTA,
};

static JSValue window_read(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                           int magic) {
    (void)this_val;
    (void)argc;
    (void)argv;

    App *app = app_for(ctx);

    if (!app)
        return JS_EXCEPTION;

    Window *window = &app->window;

    switch (magic) {
    case WINDOW_TITLE: return JS_NewString(ctx, window->title);
    case WINDOW_SIZE: return js_new_size(ctx, window_size(window));
    case WINDOW_WIDTH: return JS_NewInt32(ctx, window->width);
    case WINDOW_HEIGHT: return JS_NewInt32(ctx, window->height);
    case WINDOW_RESIZABLE: return JS_NewBool(ctx, window->resizable);
    case WINDOW_MOUSE_POSITION: return js_new_vec2(ctx, window->mouse);
    default: return js_new_vec2(ctx, window->mouse_delta);
    }
}

static JSValue window_set_title_js(JSContext *ctx, JSValueConst this_val, int argc,
                                   JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    App *app = mutable_app_for(ctx, "setTitle");

    if (!app)
        return JS_EXCEPTION;

    const char *title = JS_ToCString(ctx, argv[0]);

    if (!title)
        return JS_EXCEPTION;

    window_set_title(&app->window, title);
    JS_FreeCString(ctx, title);

    return JS_UNDEFINED;
}

static JSValue window_set_size_js(JSContext *ctx, JSValueConst this_val, int argc,
                                  JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    App *app = mutable_app_for(ctx, "setSize");

    if (!app)
        return JS_EXCEPTION;

    int32_t width, height;

    if (JS_ToInt32(ctx, &width, argv[0]) || JS_ToInt32(ctx, &height, argv[1]))
        return JS_EXCEPTION;

    window_set_size(&app->window, width, height);

    return JS_UNDEFINED;
}

static JSValue window_set_resizable_js(JSContext *ctx, JSValueConst this_val, int argc,
                                       JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    App *app = mutable_app_for(ctx, "setResizable");

    if (!app)
        return JS_EXCEPTION;

    window_set_resizable(&app->window, JS_ToBool(ctx, argv[0]));

    return JS_UNDEFINED;
}

static JSValue window_set_cursor_js(JSContext *ctx, JSValueConst this_val, int argc,
                                    JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    App *app = mutable_app_for(ctx, "setCursor");

    if (!app)
        return JS_EXCEPTION;

    const char *cursor = js_get_cursor(ctx, argv[0]);

    if (!cursor)
        return JS_EXCEPTION;

    window_set_cursor(&app->window, cursor);

    return JS_UNDEFINED;
}

enum {
    SCENE_OP_LOAD,
    SCENE_OP_UNLOAD,
    SCENE_OP_ACTIVATE,
    SCENE_OP_DEACTIVATE,
};

static JSValue window_scene_op(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                               int magic) {
    (void)this_val;
    (void)argc;

    App *app = mutable_app_for(ctx, "a scene transition");

    if (!app)
        return JS_EXCEPTION;

    const char *id = JS_ToCString(ctx, argv[0]);

    if (!id)
        return JS_EXCEPTION;

    // Named up front so a typo fails at the call site rather than as a warning
    // once the frame is over.
    if (!app_find_scene(app, id)) {
        JS_ThrowReferenceError(ctx, "no scene named '%s'", id);
        JS_FreeCString(ctx, id);
        return JS_EXCEPTION;
    }

    switch (magic) {
    case SCENE_OP_LOAD: app_load_scene(app, id); break;
    case SCENE_OP_UNLOAD: app_unload_scene(app, id); break;
    case SCENE_OP_ACTIVATE: app_activate_scene(app, id); break;
    default: app_deactivate_scene(app, id); break;
    }

    JS_FreeCString(ctx, id);

    return JS_UNDEFINED;
}

static JSValue window_quit(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)this_val;
    (void)argc;
    (void)argv;

    App *app = app_for(ctx);

    if (!app)
        return JS_EXCEPTION;

    app_quit(app);

    return JS_UNDEFINED;
}

static const JSCFunctionListEntry window_funcs[] = {
    JS_CFUNC_MAGIC_DEF("title", 0, window_read, WINDOW_TITLE),
    JS_CFUNC_MAGIC_DEF("size", 0, window_read, WINDOW_SIZE),
    JS_CFUNC_MAGIC_DEF("width", 0, window_read, WINDOW_WIDTH),
    JS_CFUNC_MAGIC_DEF("height", 0, window_read, WINDOW_HEIGHT),
    JS_CFUNC_MAGIC_DEF("resizable", 0, window_read, WINDOW_RESIZABLE),
    JS_CFUNC_MAGIC_DEF("mousePosition", 0, window_read, WINDOW_MOUSE_POSITION),
    JS_CFUNC_MAGIC_DEF("mouseDelta", 0, window_read, WINDOW_MOUSE_DELTA),
    JS_CFUNC_DEF("setTitle", 1, window_set_title_js),
    JS_CFUNC_DEF("setSize", 2, window_set_size_js),
    JS_CFUNC_DEF("setResizable", 1, window_set_resizable_js),
    JS_CFUNC_DEF("setCursor", 1, window_set_cursor_js),
    JS_CFUNC_DEF("quit", 0, window_quit),
    JS_CFUNC_MAGIC_DEF("loadScene", 1, window_scene_op, SCENE_OP_LOAD),
    JS_CFUNC_MAGIC_DEF("unloadScene", 1, window_scene_op, SCENE_OP_UNLOAD),
    JS_CFUNC_MAGIC_DEF("activateScene", 1, window_scene_op, SCENE_OP_ACTIVATE),
    JS_CFUNC_MAGIC_DEF("deactivateScene", 1, window_scene_op, SCENE_OP_DEACTIVATE),
};

// --- time -------------------------------------------------------------------

enum {
    TIME_DELTA,
    TIME_FIXED_DELTA,
    TIME_FPS,
    TIME_ALPHA,
    TIME_ELAPSED,
};

static JSValue time_read(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                         int magic) {
    (void)this_val;
    (void)argc;
    (void)argv;

    App *app = app_for(ctx);

    if (!app)
        return JS_EXCEPTION;

    Clock *clock = &app->clock;

    switch (magic) {
    case TIME_DELTA: return JS_NewFloat64(ctx, clock->dt);
    case TIME_FIXED_DELTA: return JS_NewFloat64(ctx, clock->tick_rate);
    case TIME_FPS: return JS_NewFloat64(ctx, app->fps);
    case TIME_ALPHA:
        // How far this frame sits between two fixed ticks, for interpolating
        // whatever fixedUpdate moved.
        return JS_NewFloat64(ctx, clock->tick_rate > 0.0 ? clock->accumulator / clock->tick_rate
                                                         : 0.0);
    default: return JS_NewFloat64(ctx, clock->elapsed);
    }
}

static JSValue time_set_target_tps(JSContext *ctx, JSValueConst this_val, int argc,
                                   JSValueConst *argv) {
    (void)this_val;
    (void)argc;

    App *app = app_for(ctx);

    if (!app)
        return JS_EXCEPTION;

    int32_t target;

    if (JS_ToInt32(ctx, &target, argv[0]))
        return JS_EXCEPTION;

    if (target <= 0)
        return JS_ThrowRangeError(ctx, "target tps must be positive");

    clock_set_target_tps(&app->clock, (u32)target);

    return JS_UNDEFINED;
}

static const JSCFunctionListEntry time_funcs[] = {
    JS_CFUNC_MAGIC_DEF("delta", 0, time_read, TIME_DELTA),
    JS_CFUNC_MAGIC_DEF("fixedDelta", 0, time_read, TIME_FIXED_DELTA),
    JS_CFUNC_MAGIC_DEF("fps", 0, time_read, TIME_FPS),
    JS_CFUNC_MAGIC_DEF("alpha", 0, time_read, TIME_ALPHA),
    JS_CFUNC_MAGIC_DEF("elapsed", 0, time_read, TIME_ELAPSED),
    JS_CFUNC_DEF("setTargetTps", 1, time_set_target_tps),
};

// --- input ------------------------------------------------------------------

enum {
    INPUT_KEY_DOWN,
    INPUT_KEY_PRESSED,
    INPUT_KEY_RELEASED,
    INPUT_MOUSE_DOWN,
    INPUT_MOUSE_PRESSED,
    INPUT_MOUSE_RELEASED,
};

static JSValue input_query(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                           int magic) {
    (void)this_val;
    (void)argc;

    App *app = app_for(ctx);

    if (!app)
        return JS_EXCEPTION;

    Input *input = &app->input;

    if (magic <= INPUT_KEY_RELEASED) {
        SDL_Scancode key;

        if (!js_get_key(ctx, argv[0], &key))
            return JS_EXCEPTION;

        switch (magic) {
        case INPUT_KEY_DOWN: return JS_NewBool(ctx, input_key_down(input, key));
        case INPUT_KEY_PRESSED: return JS_NewBool(ctx, input_key_pressed(input, key));
        default: return JS_NewBool(ctx, input_key_released(input, key));
        }
    }

    u8 button;

    if (!js_get_button(ctx, argv[0], &button))
        return JS_EXCEPTION;

    switch (magic) {
    case INPUT_MOUSE_DOWN: return JS_NewBool(ctx, input_button_down(input, button));
    case INPUT_MOUSE_PRESSED: return JS_NewBool(ctx, input_button_pressed(input, button));
    default: return JS_NewBool(ctx, input_button_released(input, button));
    }
}

static JSValue input_mouse_wheel(JSContext *ctx, JSValueConst this_val, int argc,
                                 JSValueConst *argv) {
    (void)this_val;
    (void)argc;
    (void)argv;

    App *app = app_for(ctx);

    if (!app)
        return JS_EXCEPTION;

    return js_new_vec2(ctx, app->input.wheel);
}

static const JSCFunctionListEntry input_funcs[] = {
    JS_CFUNC_MAGIC_DEF("keyDown", 1, input_query, INPUT_KEY_DOWN),
    JS_CFUNC_MAGIC_DEF("keyPressed", 1, input_query, INPUT_KEY_PRESSED),
    JS_CFUNC_MAGIC_DEF("keyReleased", 1, input_query, INPUT_KEY_RELEASED),
    JS_CFUNC_MAGIC_DEF("mouseDown", 1, input_query, INPUT_MOUSE_DOWN),
    JS_CFUNC_MAGIC_DEF("mousePressed", 1, input_query, INPUT_MOUSE_PRESSED),
    JS_CFUNC_MAGIC_DEF("mouseReleased", 1, input_query, INPUT_MOUSE_RELEASED),
    JS_CFUNC_DEF("mouseWheel", 0, input_mouse_wheel),
};

JSValue js_new_context_object(JSContext *ctx) {
    JSValue window = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(ctx, window, window_funcs, COUNT_OF(window_funcs));

    JSValue time = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(ctx, time, time_funcs, COUNT_OF(time_funcs));

    JSValue input = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(ctx, input, input_funcs, COUNT_OF(input_funcs));

    JSValue context = JS_NewObject(ctx);

    // Not writable: `ctx.window = ...` in one scene must not change what every
    // other scene sees.
    JS_DefinePropertyValueStr(ctx, context, "window", window, JS_PROP_ENUMERABLE);
    JS_DefinePropertyValueStr(ctx, context, "time", time, JS_PROP_ENUMERABLE);
    JS_DefinePropertyValueStr(ctx, context, "input", input, JS_PROP_ENUMERABLE);

    JS_PreventExtensions(ctx, context);

    return context;
}
