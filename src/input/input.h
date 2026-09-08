#ifndef KARNA_INPUT_H
#define KARNA_INPUT_H

#include <stdbool.h>

#include <SDL3/SDL.h>

#include <karna/math.h>
#include <karna/types.h>

#define INPUT_BUTTON_COUNT 8

// Held state persists; the pressed/released edges live for exactly one frame,
// which is what makes `keyPressed` mean "went down this frame".
typedef struct {
    bool key_down[SDL_SCANCODE_COUNT];
    bool key_pressed[SDL_SCANCODE_COUNT];
    bool key_released[SDL_SCANCODE_COUNT];

    bool button_down[INPUT_BUTTON_COUNT];
    bool button_pressed[INPUT_BUTTON_COUNT];
    bool button_released[INPUT_BUTTON_COUNT];

    Vec2 wheel;
} Input;

// Clears the edges. Called once per frame, before events are pumped.
void input_begin_frame(Input *input);
void input_handle_event(Input *input, const SDL_Event *event);

bool input_key_down(const Input *input, SDL_Scancode key);
bool input_key_pressed(const Input *input, SDL_Scancode key);
bool input_key_released(const Input *input, SDL_Scancode key);

bool input_button_down(const Input *input, u8 button);
bool input_button_pressed(const Input *input, u8 button);
bool input_button_released(const Input *input, u8 button);

// The identifier a script uses, derived from SDL's own name: "Left Shift"
// becomes "LeftShift", "1" becomes "Num1". NULL for an unnamed scancode.
const char *input_key_identifier(SDL_Scancode key, char *buf, usize size);

#endif
