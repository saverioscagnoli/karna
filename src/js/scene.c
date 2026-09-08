#include <stdlib.h>
#include <string.h>

#include "core/log.h"
#include "js/js.h"

// A scene is a plain object with optional load/update/fixedUpdate/draw/unload
// methods. Each is called with the object as `this`, so a script keeps state on
// `this` the way a C scene keeps it in its struct.
typedef struct {
    Js *js;
    JSValue object;
    char id[SCENE_ID_MAX];

    // A scene that throws is switched off rather than allowed to throw again on
    // every frame for the rest of the run.
    bool broken;
} JsScene;

static bool get_bool(JSContext *ctx, JSValueConst object, const char *key, bool fallback) {
    JSValue value = JS_GetPropertyStr(ctx, object, key);
    bool out = JS_IsUndefined(value) ? fallback : JS_ToBool(ctx, value);

    JS_FreeValue(ctx, value);

    return out;
}

static i32 get_int(JSContext *ctx, JSValueConst object, const char *key, i32 fallback) {
    JSValue value = JS_GetPropertyStr(ctx, object, key);

    int32_t out = fallback;

    if (!JS_IsUndefined(value))
        JS_ToInt32(ctx, &out, value);

    JS_FreeValue(ctx, value);

    return out;
}

static bool has_method(JSContext *ctx, JSValueConst object, const char *name) {
    JSValue value = JS_GetPropertyStr(ctx, object, name);
    bool out = JS_IsFunction(ctx, value);

    JS_FreeValue(ctx, value);

    return out;
}

static void call_hook(JsScene *scene, JsPhase phase, const char *method, bool with_draw) {
    if (scene->broken)
        return;

    Js *js = scene->js;
    JSContext *ctx = js->ctx;

    JSValue fn = JS_GetPropertyStr(ctx, scene->object, method);

    if (!JS_IsFunction(ctx, fn)) {
        JS_FreeValue(ctx, fn);
        return;
    }

    JSValue args[2] = {js->context_obj, js->draw_obj};

    JsPhase previous = js->phase;
    js->phase = phase;

    JSValue result = JS_Call(ctx, fn, scene->object, with_draw ? 2 : 1, args);

    js->phase = previous;

    JS_FreeValue(ctx, fn);

    if (JS_IsException(result)) {
        char where[128];
        snprintf(where, sizeof(where), "in scene '%s' %s", scene->id, method);

        js_dump_error(ctx, where);

        // Deactivating goes through the same queue a script would use, so it
        // lands between frames like any other transition.
        scene->broken = true;
        app_deactivate_scene(js->app, scene->id);

        log_warn("scene '%s' disabled after the error above", scene->id);
    }

    JS_FreeValue(ctx, result);

    js_drain_jobs(js);
}

static void scene_load(App *app, void *state) {
    (void)app;
    call_hook(state, JS_PHASE_LOAD, "load", false);
}

static void scene_update(App *app, void *state) {
    (void)app;
    call_hook(state, JS_PHASE_UPDATE, "update", false);
}

static void scene_fixed_update(App *app, void *state) {
    (void)app;
    call_hook(state, JS_PHASE_FIXED_UPDATE, "fixedUpdate", false);
}

static void scene_draw(App *app, void *state, Draw *draw) {
    (void)app;
    (void)draw;
    call_hook(state, JS_PHASE_DRAW, "draw", true);
}

static void scene_unload(App *app, void *state) {
    (void)app;
    call_hook(state, JS_PHASE_LOAD, "unload", false);
}

static void scene_destroy(void *state) {
    JsScene *scene = state;

    JS_FreeValue(scene->js->ctx, scene->object);
    free(scene);
}

static bool register_scene(Js *js, const char *id, JSValueConst object) {
    if (!JS_IsObject(object)) {
        log_error("scene '%s' is not an object", id);
        return false;
    }

    JsScene *scene = calloc(1, sizeof(JsScene));

    scene->js = js;
    scene->object = JS_DupValue(js->ctx, object);
    snprintf(scene->id, sizeof(scene->id), "%s", id);

    // Only the hooks the script actually defines are wired up, so an empty
    // method never costs a call into the engine's dispatch.
    SceneVTable vt = {
        .load = has_method(js->ctx, object, "load") ? scene_load : NULL,
        .update = has_method(js->ctx, object, "update") ? scene_update : NULL,
        .fixed_update = has_method(js->ctx, object, "fixedUpdate") ? scene_fixed_update : NULL,
        .draw = has_method(js->ctx, object, "draw") ? scene_draw : NULL,
        .unload = has_method(js->ctx, object, "unload") ? scene_unload : NULL,
        .destroy = scene_destroy,
    };

    if (!app_register_scene(js->app, id, vt, scene)) {
        scene_destroy(scene);
        return false;
    }

    return true;
}

// A default export with a `scenes` map is a whole app; one with lifecycle
// methods and no map is a single scene, and gets the id "main".
static bool looks_like_scene(JSContext *ctx, JSValueConst config) {
    JSValue scenes = JS_GetPropertyStr(ctx, config, "scenes");
    bool has_scenes = JS_IsObject(scenes);

    JS_FreeValue(ctx, scenes);

    if (has_scenes)
        return false;

    return has_method(ctx, config, "load") || has_method(ctx, config, "update") ||
           has_method(ctx, config, "draw");
}

bool js_build_app(Js *js, JSValueConst config) {
    JSContext *ctx = js->ctx;

    if (!JS_IsObject(config)) {
        log_error("%s must default-export a scene or an app config", js->entry);
        return false;
    }

    bool single = looks_like_scene(ctx, config);

    WindowConfig window = window_config_default();

    JSValue title = JS_GetPropertyStr(ctx, config, "title");
    const char *title_str = JS_IsUndefined(title) ? NULL : JS_ToCString(ctx, title);

    if (title_str)
        window.title = title_str;

    window.width = get_int(ctx, config, "width", window.width);
    window.height = get_int(ctx, config, "height", window.height);
    window.resizable = get_bool(ctx, config, "resizable", window.resizable);

    js->app = app_create(&window, js->root);

    if (title_str)
        JS_FreeCString(ctx, title_str);

    JS_FreeValue(ctx, title);

    if (!js->app)
        return false;

    js->app->user = js;

    if (single) {
        if (!register_scene(js, "main", config))
            return false;

        app_activate_scene(js->app, "main");

        return true;
    }

    JSValue scenes = JS_GetPropertyStr(ctx, config, "scenes");

    if (!JS_IsObject(scenes)) {
        JS_FreeValue(ctx, scenes);
        log_error("%s exports neither scene methods nor a `scenes` map", js->entry);
        return false;
    }

    JSPropertyEnum *names = NULL;
    uint32_t count = 0;

    if (JS_GetOwnPropertyNames(ctx, &names, &count, scenes, JS_GPN_STRING_MASK | JS_GPN_ENUM_ONLY) <
        0) {
        JS_FreeValue(ctx, scenes);
        return false;
    }

    bool ok = true;

    for (uint32_t i = 0; i < count && ok; i++) {
        const char *id = JS_AtomToCString(ctx, names[i].atom);
        JSValue scene = JS_GetProperty(ctx, scenes, names[i].atom);

        ok = register_scene(js, id, scene);

        JS_FreeValue(ctx, scene);
        JS_FreeCString(ctx, id);
    }

    JS_FreePropertyEnum(ctx, names, count);
    JS_FreeValue(ctx, scenes);

    if (!ok)
        return false;

    if (count == 0) {
        log_error("%s declares no scenes", js->entry);
        return false;
    }

    // `scene` names the one that starts active; without it the first declared
    // scene runs, which is what a single-scene game wants anyway.
    JSValue active = JS_GetPropertyStr(ctx, config, "scene");

    if (JS_IsString(active)) {
        const char *id = JS_ToCString(ctx, active);

        if (app_find_scene(js->app, id)) {
            app_activate_scene(js->app, id);
        } else {
            log_error("`scene` names '%s', which is not in `scenes`", id);
            ok = false;
        }

        JS_FreeCString(ctx, id);
    } else {
        app_activate_scene(js->app, js->app->scenes.items[0].id);
    }

    JS_FreeValue(ctx, active);

    return ok;
}
