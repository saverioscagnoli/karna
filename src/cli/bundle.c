#include <stdlib.h>
#include <string.h>

#include "bundle/payload.h"
#include "cli/cli.h"
#include "core/da.h"
#include "core/fs.h"
#include "core/log.h"
#include "js/js.h"

typedef struct {
    char *name;
    JSValue value;
} RecordedModule;

typedef struct {
    RecordedModule *items;
    usize len, cap;
} ModuleList;

// Called for every module compiled while walking the entry point's imports.
// The graph the bundler writes is therefore exactly the graph the entry
// reaches -- a file in the directory that nothing imports is not carried.
static void record_module(Js *js, const char *name, JSValueConst module) {
    ModuleList *modules = js->on_module_user;

    for (usize i = 0; i < modules->len; i++)
        if (strcmp(modules->items[i].name, name) == 0)
            return;

    RecordedModule recorded = {
        .name = strdup(name),
        .value = JS_DupValue(js->ctx, module),
    };

    da_push(modules, recorded);
}

static void free_modules(JSContext *ctx, ModuleList *modules) {
    for (usize i = 0; i < modules->len; i++) {
        free(modules->items[i].name);
        JS_FreeValue(ctx, modules->items[i].value);
    }

    da_free(modules);
}

// "main.js" -> "main", so `karna bundle main.js` leaves a `main` next to it.
static char *default_output(const char *entry) {
    const char *base = fs_basename(entry);
    const char *dot = strrchr(base, '.');

    usize len = dot && dot != base ? (usize)(dot - base) : strlen(base);

    if (len == 0)
        return strdup("game");

    char *out = malloc(len + 1);
    memcpy(out, base, len);
    out[len] = '\0';

    return out;
}

static usize bundle_assets(PkgWriter *writer, const char *root) {
    char *dir = fs_join(root, "assets");

    if (!fs_is_dir(dir)) {
        free(dir);
        return 0;
    }

    StrList files = {0};
    fs_walk(dir, &files);

    usize count = 0;

    for (usize i = 0; i < files.len; i++) {
        char *path = fs_join(dir, files.items[i]);

        usize size = 0;
        u8 *data = fs_read(path, &size);

        if (data) {
            // Keyed the way a script names it -- "assets/pcb.png" -- so the
            // same path works from a source tree and from a bundle.
            char *name = fs_join("assets", files.items[i]);

            pkg_writer_add(writer, name, PKG_ASSET, data, size);

            free(name);
            free(data);

            count++;
        } else {
            log_warn("could not read asset %s", path);
        }

        free(path);
    }

    strlist_free(&files);
    free(dir);

    return count;
}

int cli_bundle(const char *entry, const char *out) {
    if (!fs_exists(entry)) {
        log_error("no such file: %s", entry);
        return 1;
    }

    char *root = fs_dirname(entry);
    char *name = strdup(fs_basename(entry));

    Js *js = js_create(root, NULL);

    if (!js) {
        log_error("could not start the javascript runtime");
        free(root);
        free(name);
        return 1;
    }

    js->entry = strdup(name);

    ModuleList modules = {0};

    js->on_module = record_module;
    js->on_module_user = &modules;

    JSContext *ctx = js->ctx;

    JSValue compiled = js_compile_module(js, name);

    if (JS_IsException(compiled)) {
        js_dump_error(ctx, "while compiling the game");
        goto fail;
    }

    // The entry is compiled directly rather than through the loader, so it is
    // the one module the hook does not see on its own.
    record_module(js, name, compiled);

    // Resolving is what pulls the imports in, and each one arrives at the hook.
    if (JS_ResolveModule(ctx, compiled) < 0) {
        js_dump_error(ctx, "while resolving imports");
        JS_FreeValue(ctx, compiled);
        goto fail;
    }

    JS_FreeValue(ctx, compiled);

    PkgWriter writer = {0};
    pkg_writer_set_entry(&writer, name);

    for (usize i = 0; i < modules.len; i++) {
        usize size = 0;
        u8 *bytecode = JS_WriteObject(ctx, &size, modules.items[i].value, JS_WRITE_OBJ_BYTECODE);

        if (!bytecode) {
            js_dump_error(ctx, "while writing bytecode");
            pkg_writer_free(&writer);
            goto fail;
        }

        pkg_writer_add(&writer, modules.items[i].name, PKG_MODULE, bytecode, size);
        js_free(ctx, bytecode);

        log_debug("module %s (%zu bytes of bytecode)", modules.items[i].name, size);
    }

    usize assets = bundle_assets(&writer, root);

    char *output = out ? strdup(out) : default_output(name);
    char *runtime = fs_exe_path();

    if (!runtime) {
        log_error("could not locate the running karna executable");
        free(output);
        pkg_writer_free(&writer);
        goto fail;
    }

    bool ok = pkg_writer_emit(&writer, runtime, output);

    if (ok) {
        usize size = 0;
        u8 *written = fs_read(output, &size);
        free(written);

        log_info("wrote %s (%zu modules, %zu assets, %.1f MB)", output, modules.len, assets,
                 size / (1024.0 * 1024.0));
    }

    free(runtime);
    free(output);

    pkg_writer_free(&writer);
    free_modules(ctx, &modules);

    js_destroy(js);
    free(root);
    free(name);

    return ok ? 0 : 1;

fail:
    free_modules(ctx, &modules);

    js_destroy(js);
    free(root);
    free(name);

    return 1;
}
