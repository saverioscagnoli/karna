#include <string.h>

#include <karna/math.h>

Mat4 mat4_identity(void) {
    Mat4 out;
    memset(out.m, 0, sizeof(out.m));

    out.m[0] = out.m[5] = out.m[10] = out.m[15] = 1.0f;

    return out;
}

Mat4 mat4_ortho(f32 width, f32 height) {
    Mat4 out = mat4_identity();

    // x: [0, width]  -> [-1, 1]
    // y: [0, height] -> [-1, 1] flipped, so y grows downward
    // z is left alone; there is no depth in the 2D pipeline.
    out.m[0] = 2.0f / width;
    out.m[5] = -2.0f / height;
    out.m[12] = -1.0f;
    out.m[13] = 1.0f;

    return out;
}
