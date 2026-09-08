#include <math.h>

#include <karna/math.h>
#include <karna/types.h>

Vec2 vec2(f32 x, f32 y) {
    return (Vec2){x, y};
}

Vec2 vec2_zero(void) {
    return (Vec2){0.0f, 0.0f};
}

Vec2 vec2_one(void) {
    return (Vec2){1.0f, 1.0f};
}

Vec2 vec2_splat(f32 v) {
    return (Vec2){v, v};
}

Vec2 vec2_from_angle(f32 angle) {
    return (Vec2){cosf(angle), sinf(angle)};
}

Vec2 vec2_add(Vec2 a, Vec2 b) {
    return (Vec2){a.x + b.x, a.y + b.y};
}

Vec2 vec2_sub(Vec2 a, Vec2 b) {
    return (Vec2){a.x - b.x, a.y - b.y};
}

Vec2 vec2_mul(Vec2 a, Vec2 b) {
    return (Vec2){a.x * b.x, a.y * b.y};
}

Vec2 vec2_div(Vec2 a, Vec2 b) {
    return (Vec2){a.x / b.x, a.y / b.y};
}

Vec2 vec2_scale(Vec2 v, f32 factor) {
    return (Vec2){v.x * factor, v.y * factor};
}

Vec2 vec2_neg(Vec2 v) {
    return (Vec2){-v.x, -v.y};
}

bool vec2_eq(Vec2 a, Vec2 b) {
    return a.x == b.x && a.y == b.y;
}

f32 vec2_length(Vec2 v) {
    return sqrtf(v.x * v.x + v.y * v.y);
}

f32 vec2_length_sq(Vec2 v) {
    return v.x * v.x + v.y * v.y;
}

f32 vec2_dot(Vec2 a, Vec2 b) {
    return a.x * b.x + a.y * b.y;
}

f32 vec2_distance(Vec2 a, Vec2 b) {
    return vec2_length(vec2_sub(a, b));
}

f32 vec2_angle(Vec2 v) {
    return atan2f(v.y, v.x);
}

Vec2 vec2_normalize(Vec2 v) {
    f32 len = vec2_length(v);
    return len > 0.0f ? vec2_scale(v, 1.0f / len) : vec2_zero();
}

Vec2 vec2_perp(Vec2 v) {
    return (Vec2){-v.y, v.x};
}

Vec2 vec2_rotate(Vec2 v, f32 angle) {
    f32 c = cosf(angle), s = sinf(angle);
    return (Vec2){v.x * c - v.y * s, v.x * s + v.y * c};
}

Vec2 vec2_lerp(Vec2 a, Vec2 b, f32 t) {
    return (Vec2){a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t};
}

Size size(f32 width, f32 height) {
    return (Size){width, height};
}

f32 size_area(Size s) {
    return s.width * s.height;
}

f32 size_aspect_ratio(Size s) {
    return s.height != 0.0f ? s.width / s.height : 0.0f;
}

Size size_scale(Size s, f32 factor) {
    return (Size){s.width * factor, s.height * factor};
}

bool size_eq(Size a, Size b) {
    return a.width == b.width && a.height == b.height;
}
