#ifndef KARNA_JS_H
#define KARNA_JS_H

#include <quickjs.h>

#include "app/app.h"
#include "bundle/payload.h"

// Which callback is on the stack. The context object handed to scripts is one
// object refilled per call rather than a fresh allocation, so the phase is what
// tells a binding whether it is allowed to run: stashing `ctx` and poking at it
// from a timer raises instead of quietly working.
typedef enum {
    JS_PHASE_NONE = 0,
    JS_PHASE_LOAD,
    JS_PHASE_UPDATE,
    JS_PHASE_FIXED_UPDATE,
    JS_PHASE_DRAW,
} JsPhase;

typedef struct Js {
    JSRuntime *rt;
    JSContext *ctx;
    App *app;

    // Module specifiers and asset paths resolve against this directory when
    // running from source; `pkg` takes over when running from a bundle.
    char *root;
    const Pkg *pkg;

    char *entry; // module name of the entry point, and the console's tag

    JsPhase phase;

    JSValue context_obj;
    JSValue draw_obj;

    // `bundle` sets this to be told about every module the loader compiles, so
    // it can serialize exactly the graph the entry point actually reaches.
    void (*on_module)(struct Js *js, const char *name, JSValueConst module);
    void *on_module_user;
} Js;

static inline Js *js_of(JSContext *ctx) {
    return (Js *)JS_GetContextOpaque(ctx);
}

Js *js_create(const char *root, const Pkg *pkg);
void js_destroy(Js *js);

// Prints an exception with its JS stack. `where` names what was running.
void js_dump_error(JSContext *ctx, const char *where);

// Compiles or reads the module without evaluating it. The name is
// root-relative and identical whether it came from disk or from a bundle.
JSValue js_compile_module(Js *js, const char *name);

// Compiles, links and evaluates the module, returning its namespace object.
JSValue js_eval_module(Js *js, const char *name);

// Runs queued promise jobs to completion. Returns false if one threw.
bool js_drain_jobs(Js *js);

// runtime.c: shared by `run` and `bundle`, which both have to resolve a
// specifier the same way for module names to match across the two.
char *js_normalize_name(const char *base, const char *name);

// module.c
JSModuleDef *js_karna_module(JSContext *ctx);

// value.c
void js_register_classes(JSContext *ctx);

JSValue js_new_vec2(JSContext *ctx, Vec2 v);
JSValue js_new_size(JSContext *ctx, Size s);
JSValue js_new_color(JSContext *ctx, Color c);

bool js_get_vec2(JSContext *ctx, JSValueConst value, Vec2 *out);

// Accepts a Color, "#rrggbb" or 0xrrggbb, which is what ColorLike means.
bool js_get_color(JSContext *ctx, JSValueConst value, Color *out);

JSValue js_vec2_constructor(JSContext *ctx);
JSValue js_size_constructor(JSContext *ctx);
JSValue js_color_constructor(JSContext *ctx);

// enums.c
JSValue js_key_namespace(JSContext *ctx);
JSValue js_button_namespace(JSContext *ctx);
JSValue js_cursor_namespace(JSContext *ctx);

bool js_get_key(JSContext *ctx, JSValueConst value, SDL_Scancode *out);
bool js_get_button(JSContext *ctx, JSValueConst value, u8 *out);
const char *js_get_cursor(JSContext *ctx, JSValueConst value);

void js_register_token_classes(JSContext *ctx);

// context.c
JSValue js_new_context_object(JSContext *ctx);

// draw.c
JSValue js_new_draw_object(JSContext *ctx);

// console.c
void js_install_console(JSContext *ctx);

// scene.c
bool js_build_app(Js *js, JSValueConst config);

#endif
