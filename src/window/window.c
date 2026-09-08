#include <stdio.h>
#include <string.h>

#include "core/log.h"
#include "window/window.h"

WindowConfig window_config_default(void) {
    return (WindowConfig){
        .title = "karna",
        .width = 1280,
        .height = 720,
        .resizable = true,
    };
}

bool window_open(Window *window, const WindowConfig *config) {
    memset(window, 0, sizeof(*window));

    SDL_WindowFlags flags = 0;

    if (config->resizable)
        flags |= SDL_WINDOW_RESIZABLE;

    window->handle = SDL_CreateWindow(config->title, config->width, config->height, flags);

    if (!window->handle) {
        log_error("could not create window: %s", SDL_GetError());
        return false;
    }

    snprintf(window->title, sizeof(window->title), "%s", config->title);

    window->width = config->width;
    window->height = config->height;
    window->resizable = config->resizable;

    return true;
}

void window_close(Window *window) {
    if (window->handle) {
        SDL_DestroyWindow(window->handle);
        window->handle = NULL;
    }
}

void window_set_title(Window *window, const char *title) {
    snprintf(window->title, sizeof(window->title), "%s", title);
    SDL_SetWindowTitle(window->handle, window->title);
}

void window_set_size(Window *window, i32 width, i32 height) {
    // The resize event is what actually updates width/height, so the values a
    // script reads always match what the swapchain is sized to.
    SDL_SetWindowSize(window->handle, width, height);
}

void window_set_resizable(Window *window, bool resizable) {
    window->resizable = resizable;
    SDL_SetWindowResizable(window->handle, resizable);
}

Size window_size(const Window *window) {
    return size((f32)window->width, (f32)window->height);
}

static const struct {
    const char *name;
    SDL_SystemCursor cursor;
} CURSORS[] = {
    {"default", SDL_SYSTEM_CURSOR_DEFAULT},
    {"pointer", SDL_SYSTEM_CURSOR_POINTER},
    {"text", SDL_SYSTEM_CURSOR_TEXT},
    {"wait", SDL_SYSTEM_CURSOR_WAIT},
    {"crosshair", SDL_SYSTEM_CURSOR_CROSSHAIR},
    {"progress", SDL_SYSTEM_CURSOR_PROGRESS},
    {"move", SDL_SYSTEM_CURSOR_MOVE},
    {"not-allowed", SDL_SYSTEM_CURSOR_NOT_ALLOWED},
    {"ew-resize", SDL_SYSTEM_CURSOR_EW_RESIZE},
    {"ns-resize", SDL_SYSTEM_CURSOR_NS_RESIZE},
    {"nesw-resize", SDL_SYSTEM_CURSOR_NESW_RESIZE},
    {"nwse-resize", SDL_SYSTEM_CURSOR_NWSE_RESIZE},
};

// Cursors are created once and kept: a script may flip between two of them
// every frame as the mouse crosses a hotspot.
static SDL_Cursor *g_cursors[sizeof(CURSORS) / sizeof(*CURSORS)];

bool window_set_cursor(Window *window, const char *name) {
    (void)window;

    for (usize i = 0; i < sizeof(CURSORS) / sizeof(*CURSORS); i++) {
        if (strcmp(CURSORS[i].name, name) != 0)
            continue;

        if (!g_cursors[i])
            g_cursors[i] = SDL_CreateSystemCursor(CURSORS[i].cursor);

        if (g_cursors[i])
            SDL_SetCursor(g_cursors[i]);

        return true;
    }

    return false;
}

void window_handle_event(Window *window, const SDL_Event *event) {
    switch (event->type) {
    case SDL_EVENT_WINDOW_RESIZED:
    case SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED:
        window->width = event->window.data1;
        window->height = event->window.data2;
        break;

    case SDL_EVENT_MOUSE_MOTION:
        window->mouse = vec2(event->motion.x, event->motion.y);
        window->mouse_delta =
            vec2_add(window->mouse_delta, vec2(event->motion.xrel, event->motion.yrel));
        break;

    default: break;
    }
}
