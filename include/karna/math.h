#ifndef KARNA_MATH_H
#define KARNA_MATH_H

#include <stdbool.h>

#include <karna/types.h>

typedef struct {
    f32 x;
    f32 y;
} Vec2;

typedef struct {
    f32 width;
    f32 height;
} Size;

#define VEC2_FMT "Vec2(%.3f, %.3f)"
#define VEC2_ARG(v) (v).x, (v).y

#define SIZE_FMT "Size(%.3f, %.3f)"
#define SIZE_ARG(s) (s).width, (s).height

Vec2 vec2(f32 x, f32 y);
Vec2 vec2_zero(void);
Vec2 vec2_one(void);
Vec2 vec2_splat(f32 v);

// The unit vector `angle` radians from +x.
Vec2 vec2_from_angle(f32 angle);

Vec2 vec2_add(Vec2 a, Vec2 b);
Vec2 vec2_sub(Vec2 a, Vec2 b);

// Componentwise; vec2_scale is the scalar one.
Vec2 vec2_mul(Vec2 a, Vec2 b);
Vec2 vec2_div(Vec2 a, Vec2 b);
Vec2 vec2_scale(Vec2 v, f32 factor);
Vec2 vec2_neg(Vec2 v);

bool vec2_eq(Vec2 a, Vec2 b);

f32 vec2_length(Vec2 v);
f32 vec2_length_sq(Vec2 v);
f32 vec2_dot(Vec2 a, Vec2 b);
f32 vec2_distance(Vec2 a, Vec2 b);
f32 vec2_angle(Vec2 v);

Vec2 vec2_normalize(Vec2 v);
Vec2 vec2_perp(Vec2 v);
Vec2 vec2_rotate(Vec2 v, f32 angle);
Vec2 vec2_lerp(Vec2 a, Vec2 b, f32 t);

Size size(f32 width, f32 height);
f32 size_area(Size s);
f32 size_aspect_ratio(Size s);
Size size_scale(Size s, f32 factor);
bool size_eq(Size a, Size b);

// Column-major 4x4, the layout both SPIR-V and the GPU backends expect.
typedef struct {
    f32 m[16];
} Mat4;

Mat4 mat4_identity(void);

// Maps (0,0)..(width,height) onto clip space with +y pointing down, so screen
// coordinates read the way a 2D API should: origin top-left.
Mat4 mat4_ortho(f32 width, f32 height);

#endif
