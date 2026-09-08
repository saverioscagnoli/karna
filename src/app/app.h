#ifndef KARNA_APP_H
#define KARNA_APP_H

#include <stdbool.h>

#include <karna/types.h>

#include "input/input.h"
#include "render/draw.h"
#include "render/gpu.h"
#include "scene/scene.h"
#include "time/clock.h"
#include "window/window.h"

// Scene transitions are requested from inside a scene's own update, so they are
// queued and applied between frames rather than mutating the list mid-iteration.
typedef enum {
    SCENE_CMD_LOAD,
    SCENE_CMD_UNLOAD,
    SCENE_CMD_ACTIVATE,
    SCENE_CMD_DEACTIVATE,
} SceneCmdKind;

typedef struct {
    SceneCmdKind kind;
    char id[SCENE_ID_MAX];
} SceneCmd;

typedef struct {
    SceneCmd *items;
    usize len, cap;
} SceneCmdList;

struct App {
    Window window;
    Gpu gpu;
    Clock clock;
    Input input;
    Draw draw;

    SceneList scenes;
    SceneCmdList pending;

    // Everything a script names -- module paths, asset paths -- is relative to
    // this, so the same game runs from a source tree or out of a bundle.
    char *root;

    bool running;
    f32 fps;

    // The scripting layer, so a binding can get from a JSContext back to here.
    void *user;
};

App *app_create(const WindowConfig *config, const char *root);
void app_destroy(App *app);

// `state` is adopted: the app calls vt.destroy on it at shutdown.
bool app_register_scene(App *app, const char *id, SceneVTable vt, void *state);
Scene *app_find_scene(App *app, const char *id);

// All four are queued and take effect after the current frame's update.
void app_load_scene(App *app, const char *id);
void app_unload_scene(App *app, const char *id);
void app_activate_scene(App *app, const char *id);
void app_deactivate_scene(App *app, const char *id);

void app_run(App *app);
void app_quit(App *app);

#endif
