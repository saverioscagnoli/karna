cmake_minimum_required(VERSION 3.16)
project(karna C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# --- vendored SDL3 (vendor/SDL, submodule pinned to release-3.2.10) ---
# Build a static SDL and bake it into the executable, so the binary is
# self-contained and there is no .so to ship or rpath to chase. `karna bundle`
# copies this executable wholesale, so anything it links has to be static.
set(SDL_SHARED       OFF CACHE BOOL "" FORCE)
set(SDL_STATIC       ON  CACHE BOOL "" FORCE)
set(SDL_TEST_LIBRARY OFF CACHE BOOL "" FORCE)
set(SDL_EXAMPLES     OFF CACHE BOOL "" FORCE)
set(SDL_INSTALL      OFF CACHE BOOL "" FORCE)

# EXCLUDE_FROM_ALL: only the parts karna actually links get built.
add_subdirectory(vendor/SDL EXCLUDE_FROM_ALL)

# --- vendored quickjs-ng (vendor/quickjs, submodule pinned to v0.16.2) ---
set(BUILD_SHARED_LIBS  OFF CACHE BOOL "" FORCE)
set(QJS_ENABLE_INSTALL OFF CACHE BOOL "" FORCE)
set(QJS_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(QJS_BUILD_LIBC     OFF CACHE BOOL "" FORCE)

add_subdirectory(vendor/quickjs EXCLUDE_FROM_ALL)

# --- shaders ---
# glslc turns the GLSL into SPIR-V, then embed.cmake turns that into a header.
# A bundled game is a single file, so the shaders cannot stay on disk.
find_program(GLSLC glslc REQUIRED
    DOC "Shaderc's GLSL compiler; ships with the Vulkan SDK")

set(KARNA_GENERATED ${CMAKE_BINARY_DIR}/generated/karna)
file(MAKE_DIRECTORY ${KARNA_GENERATED})

# karna_shader(<name> <stage>) compiles shaders/<name>.<ext> for <stage> and
# leaves a `karna_shader_<name>_<stage>` array in a generated header.
function(karna_shader name stage ext)
    set(src ${CMAKE_SOURCE_DIR}/shaders/${name}.${ext})
    set(spv ${KARNA_GENERATED}/${name}.${ext}.spv)
    set(hdr ${KARNA_GENERATED}/${name}_${stage}.h)

    add_custom_command(
        OUTPUT ${spv}
        COMMAND ${GLSLC} -fshader-stage=${stage} -O ${src} -o ${spv}
        DEPENDS ${src}
        COMMENT "glslc ${name}.${ext}"
        VERBATIM)

    add_custom_command(
        OUTPUT ${hdr}
        COMMAND ${CMAKE_COMMAND}
                -DIN=${spv} -DOUT=${hdr} -DNAME=karna_shader_${name}_${stage}
                -P ${CMAKE_SOURCE_DIR}/cmake/embed.cmake
        DEPENDS ${spv} ${CMAKE_SOURCE_DIR}/cmake/embed.cmake
        COMMENT "embed ${name}_${stage}.h"
        VERBATIM)

    list(APPEND KARNA_SHADER_HEADERS ${hdr})
    set(KARNA_SHADER_HEADERS ${KARNA_SHADER_HEADERS} PARENT_SCOPE)
endfunction()

karna_shader(immediate vertex   vert)
karna_shader(immediate fragment frag)

add_custom_target(karna_shaders DEPENDS ${KARNA_SHADER_HEADERS})

# CONFIGURE_DEPENDS: cmake re-runs the glob on build, so a newly added
# .c file gets picked up without a manual reconfigure.
file(GLOB_RECURSE KARNA_SOURCES CONFIGURE_DEPENDS src/*.c)

add_executable(karna
    ${KARNA_SOURCES}
)

add_dependencies(karna karna_shaders)

# Everything under include/ is the public API surface.
# This is what makes `#include <karna/types.h>` resolve.
target_include_directories(karna PUBLIC include)

# src/ so modules can reach each other by subdirectory, and the generated dir
# so `#include <karna/immediate_vertex.h>` finds the embedded shaders.
target_include_directories(karna PRIVATE src ${CMAKE_BINARY_DIR}/generated)

target_link_libraries(karna PRIVATE SDL3::SDL3 qjs)

target_compile_options(karna PRIVATE -Wall -Wextra)
