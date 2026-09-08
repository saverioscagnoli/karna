#include <stdlib.h>
#include <string.h>

#include <karna/color.h>

Color color_rgb(f32 r, f32 g, f32 b) {
    return (Color){r, g, b, .a = 1.0f};
}

Color color_rgba(f32 r, f32 g, f32 b, f32 a) {
    return (Color){r, g, b, a};
}

Color color_hex(u32 hex) {
    return (Color){
        ((hex >> 16) & 0xff) / 255.0f,
        ((hex >> 8) & 0xff) / 255.0f,
        (hex & 0xff) / 255.0f,
        1.0f,
    };
}

static int hex_digit(char c) {
    if (c >= '0' && c <= '9')
        return c - '0';

    if (c >= 'a' && c <= 'f')
        return c - 'a' + 10;

    if (c >= 'A' && c <= 'F')
        return c - 'A' + 10;

    return -1;
}

bool color_parse(const char *str, Color *out) {
    if (!str)
        return false;

    if (*str == '#')
        str++;

    usize len = strlen(str);

    if (len != 3 && len != 4 && len != 6 && len != 8)
        return false;

    int digits[8];

    for (usize i = 0; i < len; i++) {
        digits[i] = hex_digit(str[i]);

        if (digits[i] < 0)
            return false;
    }

    // Short form doubles each digit, so #f8a is #ff88aa.
    bool packed = len <= 4;

    f32 channels[4] = {0.0f, 0.0f, 0.0f, 1.0f};
    usize count = packed ? len : len / 2;

    for (usize i = 0; i < count; i++)
        channels[i] = packed ? (digits[i] * 17) / 255.0f
                             : (digits[i * 2] * 16 + digits[i * 2 + 1]) / 255.0f;

    *out = (Color){channels[0], channels[1], channels[2], channels[3]};

    return true;
}

Color color_with_alpha(Color c, f32 a) {
    c.a = a;
    return c;
}

bool color_eq(Color a, Color b) {
    return a.r == b.r && a.g == b.g && a.b == b.b && a.a == b.a;
}

static const struct {
    const char *name;
    Color color;
} NAMED[] = {
    {"RED", COLOR_RED},         {"GREEN", COLOR_GREEN},     {"BLUE", COLOR_BLUE},
    {"WHITE", COLOR_WHITE},     {"BLACK", COLOR_BLACK},     {"YELLOW", COLOR_YELLOW},
    {"CYAN", COLOR_CYAN},       {"MAGENTA", COLOR_MAGENTA}, {"GRAY", COLOR_GRAY},
    {"ORANGE", COLOR_ORANGE},   {"PURPLE", COLOR_PURPLE},   {"BROWN", COLOR_BROWN},
    {"PINK", COLOR_PINK},       {"TRANSPARENT", COLOR_TRANSPARENT},
};

const usize COLOR_NAMED_COUNT = sizeof(NAMED) / sizeof(*NAMED);

const char *color_named_at(usize index, Color *out) {
    if (index >= COLOR_NAMED_COUNT)
        return NULL;

    *out = NAMED[index].color;

    return NAMED[index].name;
}

bool color_named(const char *name, Color *out) {
    for (usize i = 0; i < COLOR_NAMED_COUNT; i++) {
        if (strcmp(NAMED[i].name, name) == 0) {
            *out = NAMED[i].color;
            return true;
        }
    }

    return false;
}
