#include <stdlib.h>
#include <string.h>

#include "core/log.h"
#include "js/js.h"

#define COUNT_OF(a) (sizeof(a) / sizeof(*(a)))

// One string per argument, joined by spaces, the way a browser console does.
// Objects go through JSON so `console.log(state)` shows something, rather than
// the "[object Object]" that plain string conversion would give.
static char *format_arg(JSContext *ctx, JSValueConst value) {
    const char *str = NULL;

    if (JS_IsObject(value) && !JS_IsFunction(ctx, value)) {
        JSValue json = JS_JSONStringify(ctx, value, JS_UNDEFINED, JS_UNDEFINED);

        if (!JS_IsException(json) && !JS_IsUndefined(json))
            str = JS_ToCString(ctx, json);

        JS_FreeValue(ctx, json);

        // A cycle, a BigInt, or a class with a toString: fall back rather than
        // letting the failed stringify leave an exception pending.
        if (!str)
            JS_FreeValue(ctx, JS_GetException(ctx));
    }

    if (!str)
        str = JS_ToCString(ctx, value);

    if (!str)
        return NULL;

    char *copy = strdup(str);
    JS_FreeCString(ctx, str);

    return copy;
}

static JSValue console_write(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,
                             int magic) {
    (void)this_val;

    LogLevel level = (LogLevel)magic;

    if (level < log_level())
        return JS_UNDEFINED;

    usize total = 0;
    char **parts = calloc((usize)argc, sizeof(char *));

    for (int i = 0; i < argc; i++) {
        parts[i] = format_arg(ctx, argv[i]);
        total += (parts[i] ? strlen(parts[i]) : 0) + 1;
    }

    char *line = malloc(total + 1);
    usize at = 0;

    for (int i = 0; i < argc; i++) {
        if (i)
            line[at++] = ' ';

        if (parts[i]) {
            usize len = strlen(parts[i]);
            memcpy(line + at, parts[i], len);
            at += len;
            free(parts[i]);
        }
    }

    line[at] = '\0';

    Js *js = js_of(ctx);

    log_write(level, js && js->entry ? js->entry : "js", "%s", line);

    free(parts);
    free(line);

    return JS_UNDEFINED;
}

static const JSCFunctionListEntry console_funcs[] = {
    JS_CFUNC_MAGIC_DEF("log", 1, console_write, LOG_INFO),
    JS_CFUNC_MAGIC_DEF("info", 1, console_write, LOG_INFO),
    JS_CFUNC_MAGIC_DEF("warn", 1, console_write, LOG_WARN),
    JS_CFUNC_MAGIC_DEF("error", 1, console_write, LOG_ERROR),
    JS_CFUNC_MAGIC_DEF("debug", 1, console_write, LOG_DEBUG),
    JS_CFUNC_MAGIC_DEF("trace", 1, console_write, LOG_TRACE),
};

void js_install_console(JSContext *ctx) {
    JSValue global = JS_GetGlobalObject(ctx);
    JSValue console = JS_NewObject(ctx);

    JS_SetPropertyFunctionList(ctx, console, console_funcs, COUNT_OF(console_funcs));
    JS_SetPropertyStr(ctx, global, "console", console);

    JS_FreeValue(ctx, global);
}
