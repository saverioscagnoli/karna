#ifndef KARNA_COLOR_H
#define KARNA_COLOR_H

#include <stdbool.h>

#include <karna/types.h>

// Straight (non-premultiplied) RGBA, each component in 0..1.
typedef struct {
    f32 r;
    f32 g;
    f32 b;
    f32 a;
} Color;

#define COLOR_FMT "Color(%.3f, %.3f, %.3f, %.3f)"
#define COLOR_ARG(c) (c).r, (c).g, (c).b, (c).a

#define COLOR_RED ((Color){1.0f, 0.0f, 0.0f, 1.0f})
#define COLOR_GREEN ((Color){0.0f, 1.0f, 0.0f, 1.0f})
#define COLOR_BLUE ((Color){0.0f, 0.0f, 1.0f, 1.0f})
#define COLOR_WHITE ((Color){1.0f, 1.0f, 1.0f, 1.0f})
#define COLOR_BLACK ((Color){0.0f, 0.0f, 0.0f, 1.0f})
#define COLOR_YELLOW ((Color){1.0f, 1.0f, 0.0f, 1.0f})
#define COLOR_CYAN ((Color){0.0f, 1.0f, 1.0f, 1.0f})
#define COLOR_MAGENTA ((Color){1.0f, 0.0f, 1.0f, 1.0f})
#define COLOR_GRAY ((Color){0.5f, 0.5f, 0.5f, 1.0f})
#define COLOR_ORANGE ((Color){1.0f, 0.647f, 0.0f, 1.0f})
#define COLOR_PURPLE ((Color){0.5f, 0.0f, 0.5f, 1.0f})
#define COLOR_BROWN ((Color){0.647f, 0.165f, 0.165f, 1.0f})
#define COLOR_PINK ((Color){1.0f, 0.753f, 0.796f, 1.0f})
#define COLOR_TRANSPARENT ((Color){0.0f, 0.0f, 0.0f, 0.0f})

Color color_rgb(f32 r, f32 g, f32 b);
Color color_rgba(f32 r, f32 g, f32 b, f32 a);

// 0xRRGGBB (opaque). Alpha comes from color_with_alpha or the string form.
Color color_hex(u32 hex);

// "#rgb", "#rgba", "#rrggbb", "#rrggbbaa"; the '#' is optional. Returns false
// and leaves `out` untouched if the string is not one of those.
bool color_parse(const char *str, Color *out);

Color color_with_alpha(Color c, f32 a);
bool color_eq(Color a, Color b);

// Looks a constant up by the name the bindings expose ("RED", "CYAN", ...).
bool color_named(const char *name, Color *out);

// The same table by index, so the bindings can define Color.RED and the rest
// without repeating the list. Returns NULL past the end.
extern const usize COLOR_NAMED_COUNT;
const char *color_named_at(usize index, Color *out);

#endif
