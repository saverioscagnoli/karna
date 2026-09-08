#ifndef KARNA_DRAW_H
#define KARNA_DRAW_H

#include <karna/color.h>
#include <karna/math.h>
#include <karna/types.h>

// One layout for everything the batcher emits. Untextured primitives point at
// a 1x1 white texel, so a rect and an image differ only in their uvs.
typedef struct {
    f32 x, y;
    f32 r, g, b, a;
    f32 u, v;
} Vertex;

typedef struct {
    Vertex *items;
    usize len, cap;
} VertexList;

typedef struct {
    u32 *items;
    usize len, cap;
} IndexList;

// The handle a scene paints through. Geometry accumulates for the whole frame
// and goes to the GPU in one upload once every active scene has drawn.
typedef struct Draw {
    Color color;
    Size viewport;

    VertexList vertices;
    IndexList indices;
} Draw;

void draw_free(Draw *draw);

void draw_begin(Draw *draw, Size viewport);

void draw_set_color(Draw *draw, Color color);
Color draw_color(const Draw *draw);
Size draw_viewport(const Draw *draw);

void draw_rect(Draw *draw, f32 x, f32 y, f32 width, f32 height);

#endif
