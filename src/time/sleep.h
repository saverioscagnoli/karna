#ifndef KARNA_SLEEP_H
#define KARNA_SLEEP_H

#include <SDL3/SDL_timer.h>

#include <karna/types.h>

static inline void sleep_until(u64 deadline_ns) {
    u64 now = SDL_GetTicksNS();

    if (deadline_ns > now)
        SDL_DelayPrecise(deadline_ns - now);
}

#endif
