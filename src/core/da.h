#ifndef KARNA_DA_H
#define KARNA_DA_H

#include <stdlib.h>
#include <string.h>

// A growable array is `struct { T *items; size_t len, cap; }` and these macros
// operate on any struct with those three fields. Not a container type, so an
// array of vertices can keep the exact struct the GPU wants to see.

#define DA_INIT_CAP 16

#define da_reserve(da, want)                                                                       \
    do {                                                                                           \
        if ((want) > (da)->cap) {                                                                  \
            size_t _cap = (da)->cap ? (da)->cap : DA_INIT_CAP;                                     \
            while (_cap < (want))                                                                  \
                _cap *= 2;                                                                         \
            (da)->items = realloc((da)->items, _cap * sizeof(*(da)->items));                       \
            (da)->cap = _cap;                                                                      \
        }                                                                                          \
    } while (0)

#define da_push(da, value)                                                                         \
    do {                                                                                           \
        da_reserve((da), (da)->len + 1);                                                           \
        (da)->items[(da)->len++] = (value);                                                        \
    } while (0)

#define da_append(da, src, count)                                                                  \
    do {                                                                                           \
        da_reserve((da), (da)->len + (count));                                                     \
        memcpy((da)->items + (da)->len, (src), (count) * sizeof(*(da)->items));                    \
        (da)->len += (count);                                                                      \
    } while (0)

#define da_clear(da) ((da)->len = 0)

#define da_free(da)                                                                                \
    do {                                                                                           \
        free((da)->items);                                                                         \
        (da)->items = NULL;                                                                        \
        (da)->len = (da)->cap = 0;                                                                 \
    } while (0)

#endif
