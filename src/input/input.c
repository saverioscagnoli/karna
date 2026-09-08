#include <ctype.h>
#include <string.h>

#include "input/input.h"

void input_begin_frame(Input *input) {
    memset(input->key_pressed, 0, sizeof(input->key_pressed));
    memset(input->key_released, 0, sizeof(input->key_released));
    memset(input->button_pressed, 0, sizeof(input->button_pressed));
    memset(input->button_released, 0, sizeof(input->button_released));

    input->wheel = vec2_zero();
}

void input_handle_event(Input *input, const SDL_Event *event) {
    switch (event->type) {
    case SDL_EVENT_KEY_DOWN: {
        SDL_Scancode key = event->key.scancode;

        if (key >= SDL_SCANCODE_COUNT)
            break;

        // Auto-repeat re-fires the down event; an edge should not.
        if (!event->key.repeat)
            input->key_pressed[key] = true;

        input->key_down[key] = true;
        break;
    }

    case SDL_EVENT_KEY_UP: {
        SDL_Scancode key = event->key.scancode;

        if (key >= SDL_SCANCODE_COUNT)
            break;

        input->key_down[key] = false;
        input->key_released[key] = true;
        break;
    }

    case SDL_EVENT_MOUSE_BUTTON_DOWN: {
        u8 button = event->button.button;

        if (button >= INPUT_BUTTON_COUNT)
            break;

        input->button_down[button] = true;
        input->button_pressed[button] = true;
        break;
    }

    case SDL_EVENT_MOUSE_BUTTON_UP: {
        u8 button = event->button.button;

        if (button >= INPUT_BUTTON_COUNT)
            break;

        input->button_down[button] = false;
        input->button_released[button] = true;
        break;
    }

    case SDL_EVENT_MOUSE_WHEEL:
        input->wheel = vec2_add(input->wheel, vec2(event->wheel.x, event->wheel.y));
        break;

    default: break;
    }
}

bool input_key_down(const Input *input, SDL_Scancode key) {
    return key < SDL_SCANCODE_COUNT && input->key_down[key];
}

bool input_key_pressed(const Input *input, SDL_Scancode key) {
    return key < SDL_SCANCODE_COUNT && input->key_pressed[key];
}

bool input_key_released(const Input *input, SDL_Scancode key) {
    return key < SDL_SCANCODE_COUNT && input->key_released[key];
}

bool input_button_down(const Input *input, u8 button) {
    return button < INPUT_BUTTON_COUNT && input->button_down[button];
}

bool input_button_pressed(const Input *input, u8 button) {
    return button < INPUT_BUTTON_COUNT && input->button_pressed[button];
}

bool input_button_released(const Input *input, u8 button) {
    return button < INPUT_BUTTON_COUNT && input->button_released[button];
}

const char *input_key_identifier(SDL_Scancode key, char *buf, usize size) {
    const char *name = SDL_GetScancodeName(key);

    if (!name || !*name)
        return NULL;

    usize out = 0;

    // A digit cannot start a JavaScript identifier, and `Key.1` would not parse
    // even as a property access, so the number row becomes Num1..Num0.
    if (isdigit((unsigned char)name[0]) && out + 3 < size) {
        memcpy(buf, "Num", 3);
        out = 3;
    }

    for (const char *p = name; *p && out + 1 < size; p++) {
        if (*p == ' ' || *p == '-')
            continue;

        buf[out++] = *p;
    }

    if (out == 0)
        return NULL;

    buf[out] = '\0';

    return buf;
}
