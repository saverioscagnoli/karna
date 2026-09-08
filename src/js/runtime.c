#include <stdlib.h>
#include <string.h>

#include "core/fs.h"
#include "core/log.h"
#include "js/js.h"

char *js_normalize_name(const char *base, const char *name) {
    // A bare specifier is a name, not a path: "karna" resolves to the native
    // module, and nothing about the file layout should change that.
    if (name[0] != '.')
        return strdup(name);

    char *dir = fs_dirname(base);
    char *joined = fs_join(dir, name);

    free(dir);
    fs_normalize(joined);

    return joined;
}

static char *module_normalize(JSContext *ctx, const char *base, const char *name, void *opaque) {
    (void)opaque;

    char *normalized = js_normalize_name(base, name);

    // QuickJS frees what the normalizer returns with js_free, so it has to come
    // from the engine's allocator rather than plain malloc.
    usize len = strlen(normalized);
    char *out = js_malloc(ctx, len + 1);

    if (out)
        memcpy(out, normalized, len + 1);

    free(normalized);

    return out;
}

static JSModuleDef *module_loader(JSContext *ctx, const char *name, void *opaque) {
    Js *js = (Js *)opaque;
    (void)js;

    JSValue value = js_compile_module(js_of(ctx), name);

    if (JS_IsException(value))
        return NULL;

    JSModuleDef *m = JS_VALUE_GET_PTR(value);

    if (js_of(ctx)->on_module)
        js_of(ctx)->on_module(js_of(ctx), name, value);

    // The runtime keeps the module def in its loaded list; this reference was
    // only ever a handle to hand back.
    JS_FreeValue(ctx, value);

    return m;
}

JSValue js_compile_module(Js *js, const char *name) {
    JSContext *ctx = js->ctx;

    // A bundled game has no filesystem to fall back to: what is not in the
    // payload was not imported when it was built.
    if (js->pkg) {
        const PkgEntry *entry = pkg_find(js->pkg, name, PKG_MODULE);

        if (!entry)
            return JS_ThrowReferenceError(ctx, "module '%s' is not in this bundle", name);

        return JS_ReadObject(ctx, pkg_data(js->pkg, entry), entry->size, JS_READ_OBJ_BYTECODE);
    }

    char *path = fs_join(js->root, name);

    usize size = 0;
    u8 *source = fs_read(path, &size);

    if (!source) {
        JSValue error = JS_ThrowReferenceError(ctx, "could not read module '%s' (%s)", name, path);
        free(path);
        return error;
    }

    free(path);

    // COMPILE_ONLY: the caller links and evaluates, which is also what lets the
    // bundler walk the graph without running any of it.
    JSValue value = JS_Eval(ctx, (const char *)source, size, name,
                            JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);

    free(source);

    return value;
}

bool js_drain_jobs(Js *js) {
    for (;;) {
        JSContext *pending = NULL;
        int status = JS_ExecutePendingJob(js->rt, &pending);

        if (status == 0)
            return true;

        if (status < 0) {
            js_dump_error(pending ? pending : js->ctx, "in a queued job");
            return false;
        }
    }
}

JSValue js_eval_module(Js *js, const char *name) {
    JSContext *ctx = js->ctx;

    JSValue compiled = js_compile_module(js, name);

    if (JS_IsException(compiled))
        return compiled;

    JSModuleDef *m = JS_VALUE_GET_PTR(compiled);

    if (js->on_module)
        js->on_module(js, name, compiled);

    if (JS_ResolveModule(ctx, compiled) < 0) {
        JS_FreeValue(ctx, compiled);
        return JS_EXCEPTION;
    }

    // Evaluating a module yields a promise, since a module may await at top
    // level. Nothing here should, but the promise has to be settled and checked
    // either way or a failing import would look like a success.
    JSValue result = JS_EvalFunction(ctx, compiled);

    if (JS_IsException(result))
        return result;

    if (!js_drain_jobs(js)) {
        JS_FreeValue(ctx, result);
        return JS_EXCEPTION;
    }

    JSPromiseStateEnum state = JS_PromiseState(ctx, result);

    if (state == JS_PROMISE_REJECTED) {
        JSValue reason = JS_PromiseResult(ctx, result);
        JS_FreeValue(ctx, result);

        return JS_Throw(ctx, reason);
    }

    JS_FreeValue(ctx, result);

    return JS_GetModuleNamespace(ctx, m);
}

void js_dump_error(JSContext *ctx, const char *where) {
    JSValue exception = JS_GetException(ctx);

    const char *message = JS_ToCString(ctx, exception);

    log_error("%s: %s", where, message ? message : "unknown error");

    if (message)
        JS_FreeCString(ctx, message);

    if (JS_IsError(exception)) {
        JSValue stack = JS_GetPropertyStr(ctx, exception, "stack");

        if (!JS_IsUndefined(stack)) {
            const char *text = JS_ToCString(ctx, stack);

            // One log line per frame, so the stack lines up under the message
            // instead of arriving as one long blob.
            for (const char *line = text; line && *line;) {
                const char *end = strchr(line, '\n');
                int len = end ? (int)(end - line) : (int)strlen(line);

                if (len > 0)
                    log_error("  %.*s", len, line);

                line = end ? end + 1 : NULL;
            }

            if (text)
                JS_FreeCString(ctx, text);
        }

        JS_FreeValue(ctx, stack);
    }

    JS_FreeValue(ctx, exception);
}

Js *js_create(const char *root, const Pkg *pkg) {
    Js *js = calloc(1, sizeof(Js));

    js->rt = JS_NewRuntime();

    if (!js->rt) {
        free(js);
        return NULL;
    }

    js->ctx = JS_NewContext(js->rt);

    if (!js->ctx) {
        JS_FreeRuntime(js->rt);
        free(js);
        return NULL;
    }

    js->root = strdup(root && *root ? root : ".");
    js->pkg = pkg;
    js->phase = JS_PHASE_NONE;
    js->context_obj = JS_UNDEFINED;
    js->draw_obj = JS_UNDEFINED;

    JS_SetContextOpaque(js->ctx, js);
    JS_SetRuntimeOpaque(js->rt, js);
    JS_SetModuleLoaderFunc(js->rt, module_normalize, module_loader, js);

    js_register_classes(js->ctx);

    if (!js_karna_module(js->ctx)) {
        js_destroy(js);
        return NULL;
    }

    js_install_console(js->ctx);

    js->context_obj = js_new_context_object(js->ctx);
    js->draw_obj = js_new_draw_object(js->ctx);

    return js;
}

void js_destroy(Js *js) {
    if (!js)
        return;

    JS_FreeValue(js->ctx, js->context_obj);
    JS_FreeValue(js->ctx, js->draw_obj);

    JS_FreeContext(js->ctx);
    JS_FreeRuntime(js->rt);

    free(js->root);
    free(js->entry);
    free(js);
}
