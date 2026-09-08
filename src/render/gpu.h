#ifndef KARNA_GPU_H
#define KARNA_GPU_H

#include <stdbool.h>

#include <SDL3/SDL_gpu.h>

#include <karna/color.h>
#include <karna/types.h>

#include "render/draw.h"
#include "window/window.h"

typedef struct {
    SDL_GPUDevice *device;
    SDL_GPUGraphicsPipeline *pipeline;
    SDL_GPUSampler *sampler;

    // A single white texel, so the one pipeline can draw untextured geometry
    // without a second shader or a branch in the fragment stage.
    SDL_GPUTexture *white;

    SDL_GPUBuffer *vertices;
    SDL_GPUBuffer *indices;
    u32 vertex_cap; // in vertices
    u32 index_cap;  // in indices

    SDL_GPUTransferBuffer *transfer;
    u32 transfer_cap; // in bytes

    Color clear;
} Gpu;

bool gpu_init(Gpu *gpu, Window *window);
void gpu_shutdown(Gpu *gpu, Window *window);

// Uploads the frame's geometry and presents it. Clears even when there is
// nothing to draw, so a scene that paints nothing still refreshes the window.
void gpu_present(Gpu *gpu, Window *window, const Draw *draw);

#endif
