#include "core/da.h"
#include "render/draw.h"

void draw_free(Draw *draw) {
    da_free(&draw->vertices);
    da_free(&draw->indices);
}

void draw_begin(Draw *draw, Size viewport) {
    // The lists keep their capacity across frames; only the lengths reset, so
    // steady-state drawing does no allocation at all.
    da_clear(&draw->vertices);
    da_clear(&draw->indices);

    draw->color = COLOR_WHITE;
    draw->viewport = viewport;
}

void draw_set_color(Draw *draw, Color color) {
    draw->color = color;
}

Color draw_color(const Draw *draw) {
    return draw->color;
}

Size draw_viewport(const Draw *draw) {
    return draw->viewport;
}

void draw_rect(Draw *draw, f32 x, f32 y, f32 width, f32 height) {
    Color c = draw->color;
    u32 base = (u32)draw->vertices.len;

    Vertex quad[4] = {
        {x, y, c.r, c.g, c.b, c.a, 0.0f, 0.0f},
        {x + width, y, c.r, c.g, c.b, c.a, 1.0f, 0.0f},
        {x + width, y + height, c.r, c.g, c.b, c.a, 1.0f, 1.0f},
        {x, y + height, c.r, c.g, c.b, c.a, 0.0f, 1.0f},
    };

    da_append(&draw->vertices, quad, 4);

    u32 indices[6] = {base, base + 1, base + 2, base, base + 2, base + 3};

    da_append(&draw->indices, indices, 6);
}
