#include <SDL3/SDL_timer.h>

#include <karna/types.h>

#include "clock.h"

Clock clock_new() {
    return (Clock){.tick_rate = 1.0 / 60.0,
                   .accumulator = 0.0,
                   .last = SDL_GetTicksNS(),
                   .elapsed = 0.0,
                   .dropped = 0.0,
                   .scale = 1.0};
}

void clock_advance(Clock *clock, u64 now) {
    f64 dt = (f64)(now - clock->last) / 1e9;
    f64 cap = clock->tick_rate * 5.0;

    clock->dt = dt;
    clock->last = now;

    if (dt > cap)
        clock->dropped += dt - cap;

    clock->accumulator += (dt < cap ? dt : cap) * clock->scale;
}

void clock_consume(Clock *clock) {
    clock->accumulator -= clock->tick_rate;
    clock->elapsed += clock->tick_rate;
}

void clock_set_target_tps(Clock *clock, u32 target) {
    clock->tick_rate = 1.0 / (f32)target;
}

u64 clock_next_tick(Clock *clock) {
    f64 remaining = clock->tick_rate - clock->accumulator;

    if (remaining <= 0.0 || clock->scale <= 0.0)
        return clock->last;

    return clock->last + (u64)((remaining / clock->scale) * 1e9);
}

bool clock_should_tick(Clock *clock) {
    return clock->accumulator >= clock->tick_rate;
}
