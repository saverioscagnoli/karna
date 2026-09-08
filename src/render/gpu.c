#include <string.h>

#include <karna/immediate_vertex.h>
#include <karna/immediate_fragment.h>

#include "core/log.h"
#include "render/gpu.h"

static SDL_GPUShader *create_shader(SDL_GPUDevice *device, SDL_GPUShaderStage stage,
                                    const u8 *code, usize size, u32 samplers, u32 uniforms) {
    SDL_GPUShaderCreateInfo info = {
        .code = code,
        .code_size = size,
        .entrypoint = "main",
        .format = SDL_GPU_SHADERFORMAT_SPIRV,
        .stage = stage,
        .num_samplers = samplers,
        .num_storage_textures = 0,
        .num_storage_buffers = 0,
        .num_uniform_buffers = uniforms,
    };

    SDL_GPUShader *shader = SDL_CreateGPUShader(device, &info);

    if (!shader)
        log_error("could not create shader: %s", SDL_GetError());

    return shader;
}

static bool create_pipeline(Gpu *gpu, Window *window) {
    SDL_GPUShader *vertex =
        create_shader(gpu->device, SDL_GPU_SHADERSTAGE_VERTEX, karna_shader_immediate_vertex,
                      karna_shader_immediate_vertex_len, 0, 1);

    SDL_GPUShader *fragment =
        create_shader(gpu->device, SDL_GPU_SHADERSTAGE_FRAGMENT, karna_shader_immediate_fragment,
                      karna_shader_immediate_fragment_len, 1, 0);

    if (!vertex || !fragment)
        return false;

    SDL_GPUVertexBufferDescription buffers[] = {{
        .slot = 0,
        .pitch = sizeof(Vertex),
        .input_rate = SDL_GPU_VERTEXINPUTRATE_VERTEX,
        .instance_step_rate = 0,
    }};

    SDL_GPUVertexAttribute attributes[] = {
        {.location = 0, .buffer_slot = 0, .format = SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2,
         .offset = offsetof(Vertex, x)},
        {.location = 1, .buffer_slot = 0, .format = SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4,
         .offset = offsetof(Vertex, r)},
        {.location = 2, .buffer_slot = 0, .format = SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2,
         .offset = offsetof(Vertex, u)},
    };

    SDL_GPUColorTargetDescription target = {
        .format = SDL_GetGPUSwapchainTextureFormat(gpu->device, window->handle),
        .blend_state = {
            .enable_blend = true,
            .src_color_blendfactor = SDL_GPU_BLENDFACTOR_SRC_ALPHA,
            .dst_color_blendfactor = SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
            .color_blend_op = SDL_GPU_BLENDOP_ADD,
            .src_alpha_blendfactor = SDL_GPU_BLENDFACTOR_ONE,
            .dst_alpha_blendfactor = SDL_GPU_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
            .alpha_blend_op = SDL_GPU_BLENDOP_ADD,
            .color_write_mask = 0xf,
        },
    };

    SDL_GPUGraphicsPipelineCreateInfo info = {
        .vertex_shader = vertex,
        .fragment_shader = fragment,
        .vertex_input_state = {
            .vertex_buffer_descriptions = buffers,
            .num_vertex_buffers = 1,
            .vertex_attributes = attributes,
            .num_vertex_attributes = 3,
        },
        .primitive_type = SDL_GPU_PRIMITIVETYPE_TRIANGLELIST,
        .rasterizer_state = {
            .fill_mode = SDL_GPU_FILLMODE_FILL,
            .cull_mode = SDL_GPU_CULLMODE_NONE,
            .front_face = SDL_GPU_FRONTFACE_COUNTER_CLOCKWISE,
        },
        .multisample_state = {.sample_count = SDL_GPU_SAMPLECOUNT_1},
        .target_info = {
            .color_target_descriptions = &target,
            .num_color_targets = 1,
        },
    };

    gpu->pipeline = SDL_CreateGPUGraphicsPipeline(gpu->device, &info);

    // The pipeline holds its own references; the shader objects are done.
    SDL_ReleaseGPUShader(gpu->device, vertex);
    SDL_ReleaseGPUShader(gpu->device, fragment);

    if (!gpu->pipeline) {
        log_error("could not create pipeline: %s", SDL_GetError());
        return false;
    }

    return true;
}

static bool create_white_texture(Gpu *gpu) {
    SDL_GPUTextureCreateInfo info = {
        .type = SDL_GPU_TEXTURETYPE_2D,
        .format = SDL_GPU_TEXTUREFORMAT_R8G8B8A8_UNORM,
        .usage = SDL_GPU_TEXTUREUSAGE_SAMPLER,
        .width = 1,
        .height = 1,
        .layer_count_or_depth = 1,
        .num_levels = 1,
    };

    gpu->white = SDL_CreateGPUTexture(gpu->device, &info);

    if (!gpu->white) {
        log_error("could not create white texture: %s", SDL_GetError());
        return false;
    }

    SDL_GPUTransferBufferCreateInfo transfer_info = {
        .usage = SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
        .size = 4,
    };

    SDL_GPUTransferBuffer *transfer = SDL_CreateGPUTransferBuffer(gpu->device, &transfer_info);

    if (!transfer)
        return false;

    u8 *pixels = SDL_MapGPUTransferBuffer(gpu->device, transfer, false);
    memset(pixels, 0xff, 4);
    SDL_UnmapGPUTransferBuffer(gpu->device, transfer);

    SDL_GPUCommandBuffer *cmd = SDL_AcquireGPUCommandBuffer(gpu->device);
    SDL_GPUCopyPass *pass = SDL_BeginGPUCopyPass(cmd);

    SDL_GPUTextureTransferInfo source = {.transfer_buffer = transfer};
    SDL_GPUTextureRegion region = {.texture = gpu->white, .w = 1, .h = 1, .d = 1};

    SDL_UploadToGPUTexture(pass, &source, &region, false);

    SDL_EndGPUCopyPass(pass);
    SDL_SubmitGPUCommandBuffer(cmd);

    SDL_ReleaseGPUTransferBuffer(gpu->device, transfer);

    return true;
}

bool gpu_init(Gpu *gpu, Window *window) {
    memset(gpu, 0, sizeof(*gpu));

    gpu->clear = COLOR_BLACK;

    gpu->device = SDL_CreateGPUDevice(SDL_GPU_SHADERFORMAT_SPIRV, false, NULL);

    if (!gpu->device) {
        log_error("could not create gpu device: %s", SDL_GetError());
        return false;
    }

    log_debug("gpu driver: %s", SDL_GetGPUDeviceDriver(gpu->device));

    if (!SDL_ClaimWindowForGPUDevice(gpu->device, window->handle)) {
        log_error("could not claim window for gpu device: %s", SDL_GetError());
        return false;
    }

    if (!create_pipeline(gpu, window))
        return false;

    SDL_GPUSamplerCreateInfo sampler_info = {
        .min_filter = SDL_GPU_FILTER_NEAREST,
        .mag_filter = SDL_GPU_FILTER_NEAREST,
        .mipmap_mode = SDL_GPU_SAMPLERMIPMAPMODE_NEAREST,
        .address_mode_u = SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE,
        .address_mode_v = SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE,
        .address_mode_w = SDL_GPU_SAMPLERADDRESSMODE_CLAMP_TO_EDGE,
    };

    gpu->sampler = SDL_CreateGPUSampler(gpu->device, &sampler_info);

    if (!gpu->sampler) {
        log_error("could not create sampler: %s", SDL_GetError());
        return false;
    }

    return create_white_texture(gpu);
}

void gpu_shutdown(Gpu *gpu, Window *window) {
    if (!gpu->device)
        return;

    // Nothing may be released while the GPU still has frames in flight.
    SDL_WaitForGPUIdle(gpu->device);

    if (gpu->transfer)
        SDL_ReleaseGPUTransferBuffer(gpu->device, gpu->transfer);

    if (gpu->vertices)
        SDL_ReleaseGPUBuffer(gpu->device, gpu->vertices);

    if (gpu->indices)
        SDL_ReleaseGPUBuffer(gpu->device, gpu->indices);

    if (gpu->white)
        SDL_ReleaseGPUTexture(gpu->device, gpu->white);

    if (gpu->sampler)
        SDL_ReleaseGPUSampler(gpu->device, gpu->sampler);

    if (gpu->pipeline)
        SDL_ReleaseGPUGraphicsPipeline(gpu->device, gpu->pipeline);

    if (window->handle)
        SDL_ReleaseWindowFromGPUDevice(gpu->device, window->handle);

    SDL_DestroyGPUDevice(gpu->device);

    memset(gpu, 0, sizeof(*gpu));
}

// Buffers grow by doubling and are never shrunk: a frame's geometry count is
// stable enough that reallocating is a startup cost, not a per-frame one.
static bool ensure_buffers(Gpu *gpu, u32 vertex_count, u32 index_count) {
    if (vertex_count > gpu->vertex_cap) {
        u32 cap = gpu->vertex_cap ? gpu->vertex_cap : 1024;

        while (cap < vertex_count)
            cap *= 2;

        if (gpu->vertices)
            SDL_ReleaseGPUBuffer(gpu->device, gpu->vertices);

        SDL_GPUBufferCreateInfo info = {
            .usage = SDL_GPU_BUFFERUSAGE_VERTEX,
            .size = cap * sizeof(Vertex),
        };

        gpu->vertices = SDL_CreateGPUBuffer(gpu->device, &info);

        if (!gpu->vertices)
            return false;

        gpu->vertex_cap = cap;
    }

    if (index_count > gpu->index_cap) {
        u32 cap = gpu->index_cap ? gpu->index_cap : 1536;

        while (cap < index_count)
            cap *= 2;

        if (gpu->indices)
            SDL_ReleaseGPUBuffer(gpu->device, gpu->indices);

        SDL_GPUBufferCreateInfo info = {
            .usage = SDL_GPU_BUFFERUSAGE_INDEX,
            .size = cap * sizeof(u32),
        };

        gpu->indices = SDL_CreateGPUBuffer(gpu->device, &info);

        if (!gpu->indices)
            return false;

        gpu->index_cap = cap;
    }

    u32 needed = vertex_count * sizeof(Vertex) + index_count * sizeof(u32);

    if (needed > gpu->transfer_cap) {
        u32 cap = gpu->transfer_cap ? gpu->transfer_cap : 65536;

        while (cap < needed)
            cap *= 2;

        if (gpu->transfer)
            SDL_ReleaseGPUTransferBuffer(gpu->device, gpu->transfer);

        SDL_GPUTransferBufferCreateInfo info = {
            .usage = SDL_GPU_TRANSFERBUFFERUSAGE_UPLOAD,
            .size = cap,
        };

        gpu->transfer = SDL_CreateGPUTransferBuffer(gpu->device, &info);

        if (!gpu->transfer)
            return false;

        gpu->transfer_cap = cap;
    }

    return true;
}

static void upload(Gpu *gpu, SDL_GPUCommandBuffer *cmd, const Draw *draw) {
    u32 vertex_bytes = (u32)(draw->vertices.len * sizeof(Vertex));
    u32 index_bytes = (u32)(draw->indices.len * sizeof(u32));

    // cycle=true hands back fresh storage rather than stalling on the copy the
    // GPU may still be reading from the previous frame.
    u8 *mapped = SDL_MapGPUTransferBuffer(gpu->device, gpu->transfer, true);

    memcpy(mapped, draw->vertices.items, vertex_bytes);
    memcpy(mapped + vertex_bytes, draw->indices.items, index_bytes);

    SDL_UnmapGPUTransferBuffer(gpu->device, gpu->transfer);

    SDL_GPUCopyPass *pass = SDL_BeginGPUCopyPass(cmd);

    SDL_GPUTransferBufferLocation vertex_source = {.transfer_buffer = gpu->transfer, .offset = 0};
    SDL_GPUBufferRegion vertex_region = {
        .buffer = gpu->vertices, .offset = 0, .size = vertex_bytes};

    SDL_UploadToGPUBuffer(pass, &vertex_source, &vertex_region, true);

    SDL_GPUTransferBufferLocation index_source = {.transfer_buffer = gpu->transfer,
                                                  .offset = vertex_bytes};
    SDL_GPUBufferRegion index_region = {.buffer = gpu->indices, .offset = 0, .size = index_bytes};

    SDL_UploadToGPUBuffer(pass, &index_source, &index_region, true);

    SDL_EndGPUCopyPass(pass);
}

void gpu_present(Gpu *gpu, Window *window, const Draw *draw) {
    SDL_GPUCommandBuffer *cmd = SDL_AcquireGPUCommandBuffer(gpu->device);

    if (!cmd) {
        log_error("could not acquire command buffer: %s", SDL_GetError());
        return;
    }

    SDL_GPUTexture *swapchain = NULL;
    u32 width = 0, height = 0;

    if (!SDL_WaitAndAcquireGPUSwapchainTexture(cmd, window->handle, &swapchain, &width, &height)) {
        log_error("could not acquire swapchain texture: %s", SDL_GetError());
        SDL_CancelGPUCommandBuffer(cmd);
        return;
    }

    // A minimized window has no swapchain texture; there is nothing to draw
    // into, so the frame is dropped rather than forced.
    if (!swapchain) {
        SDL_CancelGPUCommandBuffer(cmd);
        return;
    }

    bool has_geometry = draw->indices.len > 0;

    if (has_geometry) {
        if (!ensure_buffers(gpu, (u32)draw->vertices.len, (u32)draw->indices.len)) {
            log_error("could not size gpu buffers: %s", SDL_GetError());
            has_geometry = false;
        } else {
            upload(gpu, cmd, draw);
        }
    }

    SDL_GPUColorTargetInfo target = {
        .texture = swapchain,
        .clear_color = {gpu->clear.r, gpu->clear.g, gpu->clear.b, gpu->clear.a},
        .load_op = SDL_GPU_LOADOP_CLEAR,
        .store_op = SDL_GPU_STOREOP_STORE,
    };

    SDL_GPURenderPass *pass = SDL_BeginGPURenderPass(cmd, &target, 1, NULL);

    if (has_geometry) {
        SDL_BindGPUGraphicsPipeline(pass, gpu->pipeline);

        SDL_GPUBufferBinding vertex_binding = {.buffer = gpu->vertices, .offset = 0};
        SDL_BindGPUVertexBuffers(pass, 0, &vertex_binding, 1);

        SDL_GPUBufferBinding index_binding = {.buffer = gpu->indices, .offset = 0};
        SDL_BindGPUIndexBuffer(pass, &index_binding, SDL_GPU_INDEXELEMENTSIZE_32BIT);

        SDL_GPUTextureSamplerBinding texture_binding = {.texture = gpu->white,
                                                        .sampler = gpu->sampler};
        SDL_BindGPUFragmentSamplers(pass, 0, &texture_binding, 1);

        // The projection follows the swapchain rather than the window's logical
        // size, so geometry stays put through a resize on a hidpi display.
        Mat4 proj = mat4_ortho((f32)width, (f32)height);
        SDL_PushGPUVertexUniformData(cmd, 0, &proj, sizeof(proj));

        SDL_DrawGPUIndexedPrimitives(pass, (u32)draw->indices.len, 1, 0, 0, 0);
    }

    SDL_EndGPURenderPass(pass);
    SDL_SubmitGPUCommandBuffer(cmd);
}
