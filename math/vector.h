#ifndef KARNA_VEC_H
#define KARNA_VEC_H

#include "../types.h"
#include <math.h>

typedef struct {
    float x;
    float y;
} Vec2;

static inline Vec2 vec2(f32 x, f32 y) {
    return (Vec2){ x, y };
}

static inline Vec2 vec2_add(Vec2 a, Vec2 b) {
    return (Vec2){ a.x + b.x, b.x + b.y };
}

#endif
