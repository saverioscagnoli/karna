#include <string.h>

#include <SDL3/SDL.h>

#include "input/input.h"
#include "js/js.h"

// Keys, buttons and cursors reach JavaScript as opaque tokens rather than
// numbers. `input.keyDown(Key.W)` then fails loudly on a typo, where a bare
// integer would silently mean some other key.

static JSClassID key_class_id;
static JSClassID button_class_id;
static JSClassID cursor_class_id;

#define COUNT_OF(a) (sizeof(a) / sizeof(*(a)))

// Tokens carry their value in the opaque pointer itself; there is nothing to
// allocate and so nothing to finalize. Stored biased by one so a valid token is
// never a NULL opaque.
static void *pack(usize value) {
    return (void *)(uintptr_t)(value + 1);
}

static bool unpack(void *opaque, usize *out) {
    if (!opaque)
        return false;

    *out = (usize)(uintptr_t)opaque - 1;

    return true;
}

// Nothing to finalize: a token's value lives in the opaque pointer itself.
static JSClassDef key_class = {.class_name = "Key", .finalizer = NULL};
static JSClassDef button_class = {.class_name = "Button", .finalizer = NULL};
static JSClassDef cursor_class = {.class_name = "Cursor", .finalizer = NULL};

static const struct {
    const char *name;
    u8 button;
} BUTTONS[] = {
    {"Left", SDL_BUTTON_LEFT},   {"Middle", SDL_BUTTON_MIDDLE}, {"Right", SDL_BUTTON_RIGHT},
    {"X1", SDL_BUTTON_X1},       {"X2", SDL_BUTTON_X2},
};

static const struct {
    const char *key;
    const char *cursor;
} CURSORS[] = {
    {"DEFAULT", "default"},         {"POINTER", "pointer"},
    {"TEXT", "text"},               {"WAIT", "wait"},
    {"CROSSHAIR", "crosshair"},     {"PROGRESS", "progress"},
    {"MOVE", "move"},               {"NOT_ALLOWED", "not-allowed"},
    {"EW_RESIZE", "ew-resize"},     {"NS_RESIZE", "ns-resize"},
    {"NESW_RESIZE", "nesw-resize"}, {"NWSE_RESIZE", "nwse-resize"},
};

static JSValue token_name(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;
    (void)argv;

    JSClassID class_id;
    void *opaque = JS_GetAnyOpaque(this_val, &class_id);

    usize value;

    if (!unpack(opaque, &value))
        return JS_ThrowTypeError(ctx, "not a token");

    if (class_id == key_class_id) {
        char buf[64];
        const char *name = input_key_identifier((SDL_Scancode)value, buf, sizeof(buf));

        return JS_NewString(ctx, name ? name : "Unknown");
    }

    if (class_id == button_class_id) {
        for (usize i = 0; i < COUNT_OF(BUTTONS); i++)
            if (BUTTONS[i].button == value)
                return JS_NewString(ctx, BUTTONS[i].name);

        return JS_NewString(ctx, "Unknown");
    }

    return JS_NewString(ctx, CURSORS[value].key);
}

static JSValue token_eq(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    (void)argc;

    JSClassID mine, theirs;
    void *a = JS_GetAnyOpaque(this_val, &mine);
    void *b = JS_GetAnyOpaque(argv[0], &theirs);

    return JS_NewBool(ctx, a && b && mine == theirs && a == b);
}

static JSValue token_to_string(JSContext *ctx, JSValueConst this_val, int argc,
                               JSValueConst *argv) {
    return token_name(ctx, this_val, argc, argv);
}

static const JSCFunctionListEntry token_proto[] = {
    JS_CFUNC_DEF("name", 0, token_name),
    JS_CFUNC_DEF("eq", 1, token_eq),
    JS_CFUNC_DEF("toString", 0, token_to_string),
};

static void register_token_class(JSContext *ctx, JSClassID *class_id, JSClassDef *def) {
    JSRuntime *rt = JS_GetRuntime(ctx);

    JS_NewClassID(rt, class_id);
    JS_NewClass(rt, *class_id, def);

    JSValue proto = JS_NewObject(ctx);
    JS_SetPropertyFunctionList(ctx, proto, token_proto, COUNT_OF(token_proto));
    JS_SetClassProto(ctx, *class_id, proto);
}

void js_register_token_classes(JSContext *ctx) {
    register_token_class(ctx, &key_class_id, &key_class);
    register_token_class(ctx, &button_class_id, &button_class);
    register_token_class(ctx, &cursor_class_id, &cursor_class);
}

static JSValue new_token(JSContext *ctx, JSClassID class_id, usize value) {
    JSValue obj = JS_NewObjectClass(ctx, class_id);

    if (JS_IsException(obj))
        return obj;

    JS_SetOpaque(obj, pack(value));

    return obj;
}

JSValue js_key_namespace(JSContext *ctx) {
    JSValue ns = JS_NewObject(ctx);

    // Built from SDL's own scancode names rather than a hand-kept list, so the
    // set a script can name is exactly the set the engine can report.
    for (int code = SDL_SCANCODE_UNKNOWN + 1; code < SDL_SCANCODE_COUNT; code++) {
        char buf[64];
        const char *name = input_key_identifier((SDL_Scancode)code, buf, sizeof(buf));

        if (!name)
            continue;

        JS_DefinePropertyValueStr(ctx, ns, name, new_token(ctx, key_class_id, (usize)code),
                                  JS_PROP_ENUMERABLE);
    }

    JS_PreventExtensions(ctx, ns);

    return ns;
}

JSValue js_button_namespace(JSContext *ctx) {
    JSValue ns = JS_NewObject(ctx);

    for (usize i = 0; i < COUNT_OF(BUTTONS); i++)
        JS_DefinePropertyValueStr(ctx, ns, BUTTONS[i].name,
                                  new_token(ctx, button_class_id, BUTTONS[i].button),
                                  JS_PROP_ENUMERABLE);

    JS_PreventExtensions(ctx, ns);

    return ns;
}

JSValue js_cursor_namespace(JSContext *ctx) {
    JSValue ns = JS_NewObject(ctx);

    for (usize i = 0; i < COUNT_OF(CURSORS); i++)
        JS_DefinePropertyValueStr(ctx, ns, CURSORS[i].key, new_token(ctx, cursor_class_id, i),
                                  JS_PROP_ENUMERABLE);

    JS_PreventExtensions(ctx, ns);

    return ns;
}

bool js_get_key(JSContext *ctx, JSValueConst value, SDL_Scancode *out) {
    usize code;

    if (!unpack(JS_GetOpaque(value, key_class_id), &code)) {
        JS_ThrowTypeError(ctx, "expected a Key, as in Key.W");
        return false;
    }

    *out = (SDL_Scancode)code;

    return true;
}

bool js_get_button(JSContext *ctx, JSValueConst value, u8 *out) {
    usize button;

    if (!unpack(JS_GetOpaque(value, button_class_id), &button)) {
        JS_ThrowTypeError(ctx, "expected a Button, as in Button.Left");
        return false;
    }

    *out = (u8)button;

    return true;
}

const char *js_get_cursor(JSContext *ctx, JSValueConst value) {
    usize index;

    if (!unpack(JS_GetOpaque(value, cursor_class_id), &index)) {
        JS_ThrowTypeError(ctx, "expected a Cursor, as in Cursor.POINTER");
        return NULL;
    }

    return CURSORS[index].cursor;
}
