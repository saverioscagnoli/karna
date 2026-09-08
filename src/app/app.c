#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <SDL3/SDL.h>

#include "app/app.h"
#include "core/da.h"
#include "core/log.h"

App *app_create(const WindowConfig *config, const char *root) {
    if (!SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS)) {
        log_error("could not initialize sdl: %s", SDL_GetError());
        return NULL;
    }

    App *app = calloc(1, sizeof(App));

    app->clock = clock_new();
    app->root = strdup(root && *root ? root : ".");
    app->running = true;

    if (!window_open(&app->window, config)) {
        app_destroy(app);
        return NULL;
    }

    if (!gpu_init(&app->gpu, &app->window)) {
        app_destroy(app);
        return NULL;
    }

    return app;
}

void app_destroy(App *app) {
    if (!app)
        return;

    for (usize i = 0; i < app->scenes.len; i++) {
        Scene *scene = &app->scenes.items[i];

        if (scene->loaded && scene->vt.unload)
            scene->vt.unload(app, scene->state);

        if (scene->vt.destroy)
            scene->vt.destroy(scene->state);
    }

    da_free(&app->scenes);
    da_free(&app->pending);

    draw_free(&app->draw);
    gpu_shutdown(&app->gpu, &app->window);
    window_close(&app->window);

    free(app->root);
    free(app);

    SDL_Quit();
}

bool app_register_scene(App *app, const char *id, SceneVTable vt, void *state) {
    if (app_find_scene(app, id)) {
        log_error("scene '%s' is already registered", id);
        return false;
    }

    Scene scene = {.vt = vt, .state = state};
    snprintf(scene.id, sizeof(scene.id), "%s", id);

    da_push(&app->scenes, scene);

    return true;
}

Scene *app_find_scene(App *app, const char *id) {
    for (usize i = 0; i < app->scenes.len; i++)
        if (strcmp(app->scenes.items[i].id, id) == 0)
            return &app->scenes.items[i];

    return NULL;
}

static void queue(App *app, SceneCmdKind kind, const char *id) {
    SceneCmd cmd = {.kind = kind};
    snprintf(cmd.id, sizeof(cmd.id), "%s", id);

    da_push(&app->pending, cmd);
}

void app_load_scene(App *app, const char *id) {
    queue(app, SCENE_CMD_LOAD, id);
}

void app_unload_scene(App *app, const char *id) {
    queue(app, SCENE_CMD_UNLOAD, id);
}

void app_activate_scene(App *app, const char *id) {
    queue(app, SCENE_CMD_ACTIVATE, id);
}

void app_deactivate_scene(App *app, const char *id) {
    queue(app, SCENE_CMD_DEACTIVATE, id);
}

void app_quit(App *app) {
    app->running = false;
}

static void scene_load(App *app, Scene *scene) {
    if (scene->loaded)
        return;

    scene->loaded = true;

    if (scene->vt.load)
        scene->vt.load(app, scene->state);
}

static void scene_unload(App *app, Scene *scene) {
    if (!scene->loaded)
        return;

    scene->active = false;
    scene->loaded = false;

    if (scene->vt.unload)
        scene->vt.unload(app, scene->state);
}

static void apply_pending(App *app) {
    // Indexed rather than iterated: a load hook may queue more commands, and
    // those belong to this same drain.
    for (usize i = 0; i < app->pending.len; i++) {
        SceneCmd cmd = app->pending.items[i];
        Scene *scene = app_find_scene(app, cmd.id);

        if (!scene) {
            log_warn("no scene named '%s'", cmd.id);
            continue;
        }

        switch (cmd.kind) {
        case SCENE_CMD_LOAD: scene_load(app, scene); break;
        case SCENE_CMD_UNLOAD: scene_unload(app, scene); break;

        case SCENE_CMD_ACTIVATE:
            // Activating is the only entry point a script needs: a scene that
            // has never been loaded is loaded on the way in.
            scene_load(app, scene);
            scene->active = true;
            break;

        case SCENE_CMD_DEACTIVATE: scene->active = false; break;
        }
    }

    da_clear(&app->pending);
}

static void pump_events(App *app) {
    input_begin_frame(&app->input);
    app->window.mouse_delta = vec2_zero();

    SDL_Event event;

    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_EVENT_QUIT)
            app->running = false;

        window_handle_event(&app->window, &event);
        input_handle_event(&app->input, &event);
    }
}

void app_run(App *app) {
    // Scenes registered before the loop start out queued, so the first frame
    // sees them already through `load`.
    apply_pending(app);

    app->clock.last = SDL_GetTicksNS();

    while (app->running) {
        pump_events(app);

        clock_advance(&app->clock, SDL_GetTicksNS());

        // Smoothed, because a per-frame reciprocal reads as noise on screen.
        if (app->clock.dt > 0.0f) {
            f32 instant = 1.0f / app->clock.dt;
            app->fps = app->fps == 0.0f ? instant : app->fps * 0.9f + instant * 0.1f;
        }

        while (clock_should_tick(&app->clock)) {
            for (usize i = 0; i < app->scenes.len; i++) {
                Scene *scene = &app->scenes.items[i];

                if (scene->active && scene->vt.fixed_update)
                    scene->vt.fixed_update(app, scene->state);
            }

            clock_consume(&app->clock);
        }

        for (usize i = 0; i < app->scenes.len; i++) {
            Scene *scene = &app->scenes.items[i];

            if (scene->active && scene->vt.update)
                scene->vt.update(app, scene->state);
        }

        apply_pending(app);

        draw_begin(&app->draw, window_size(&app->window));

        for (usize i = 0; i < app->scenes.len; i++) {
            Scene *scene = &app->scenes.items[i];

            if (scene->active && scene->vt.draw)
                scene->vt.draw(app, scene->state, &app->draw);
        }

        gpu_present(&app->gpu, &app->window, &app->draw);
    }
}
