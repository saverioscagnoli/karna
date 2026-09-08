#include <stdlib.h>
#include <string.h>

#include "cli/cli.h"
#include "core/fs.h"
#include "core/log.h"
#include "js/js.h"

// Both entry points do the same three things -- evaluate the entry module, read
// its default export, build an app out of it -- and differ only in where the
// modules come from.
static int play(Js *js) {
    if (!js)
        return 1;

    JSContext *ctx = js->ctx;

    JSValue namespace = js_eval_module(js, js->entry);

    if (JS_IsException(namespace)) {
        js_dump_error(ctx, "while loading the game");
        js_destroy(js);
        return 1;
    }

    JSValue config = JS_GetPropertyStr(ctx, namespace, "default");

    JS_FreeValue(ctx, namespace);

    if (JS_IsUndefined(config)) {
        log_error("%s has no default export", js->entry);
        JS_FreeValue(ctx, config);
        js_destroy(js);
        return 1;
    }

    bool ok = js_build_app(js, config);

    JS_FreeValue(ctx, config);

    if (!ok) {
        if (JS_HasException(ctx))
            js_dump_error(ctx, "while starting the game");

        if (js->app)
            app_destroy(js->app);

        js_destroy(js);

        return 1;
    }

    app_run(js->app);

    // The app owns the scene objects, so it goes first: destroying it releases
    // the JSValues while the runtime is still alive to take them.
    app_destroy(js->app);
    js->app = NULL;

    js_destroy(js);

    return 0;
}

int cli_run(const char *entry) {
    if (!fs_exists(entry)) {
        log_error("no such file: %s", entry);
        return 1;
    }

    char *root = fs_dirname(entry);
    Js *js = js_create(root, NULL);

    free(root);

    if (!js) {
        log_error("could not start the javascript runtime");
        return 1;
    }

    js->entry = strdup(fs_basename(entry));

    return play(js);
}

int cli_play(const Pkg *pkg) {
    // The root still matters for anything that reads from disk beside the
    // executable; modules and bundled assets come out of the payload.
    Js *js = js_create(".", pkg);

    if (!js) {
        log_error("could not start the javascript runtime");
        return 1;
    }

    js->entry = strdup(pkg->entry);

    return play(js);
}
