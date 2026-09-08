#ifndef KARNA_WINDOW_H
#define KARNA_WINDOW_H

#include <stdbool.h>

#include <SDL3/SDL.h>

#include <karna/math.h>
#include <karna/types.h>

typedef struct {
    const char *title;
    i32 width;
    i32 height;
    bool resizable;
} WindowConfig;

WindowConfig window_config_default(void);

typedef struct {
    SDL_Window *handle;
    char title[256];
    i32 width;
    i32 height;
    bool resizable;

    Vec2 mouse;
    // Accumulated over the frame and cleared by the app loop, so a script sees
    // one number per frame rather than whatever the last motion event carried.
    Vec2 mouse_delta;
} Window;

bool window_open(Window *window, const WindowConfig *config);
void window_close(Window *window);

void window_set_title(Window *window, const char *title);
void window_set_size(Window *window, i32 width, i32 height);
void window_set_resizable(Window *window, bool resizable);

Size window_size(const Window *window);

// The system cursor shapes a script can ask for by name ("default", "pointer",
// "text", ...). Returns false for a name that is not one of them.
bool window_set_cursor(Window *window, const char *name);

void window_handle_event(Window *window, const SDL_Event *event);

#endif
