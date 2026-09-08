#ifndef KARNA_SCENE_H
#define KARNA_SCENE_H

#include <stdbool.h>

#include <karna/types.h>

typedef struct App App;
typedef struct Draw Draw;

#define SCENE_ID_MAX 64

// Every hook is optional. `state` is whatever was handed to
// app_register_scene: a struct for a C scene, the default-exported object for
// a JavaScript one.
typedef struct {
    void (*load)(App *app, void *state);
    void (*update)(App *app, void *state);
    void (*fixed_update)(App *app, void *state);
    void (*draw)(App *app, void *state, Draw *draw);
    void (*unload)(App *app, void *state);

    // Releases `state` itself, once the scene is gone for good.
    void (*destroy)(void *state);
} SceneVTable;

typedef struct {
    char id[SCENE_ID_MAX];
    SceneVTable vt;
    void *state;

    // Loaded but inactive is a real state: the scene keeps whatever it built in
    // `load`, and costs nothing per frame until it is activated again.
    bool loaded;
    bool active;
} Scene;

typedef struct {
    Scene *items;
    usize len, cap;
} SceneList;

#endif
