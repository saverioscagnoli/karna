#ifndef KARNA_CLOCK_H
#define KARNA_CLOCK_H

#include <stdbool.h>

#include <karna/types.h>

typedef struct {
    f32 dt;
    f64 tick_rate;
    f64 accumulator;
    u64 last;
    f32 elapsed;
    f64 dropped;
    f32 scale;
} Clock;

Clock clock_new();

void clock_advance(Clock *clock, u64 now);
void clock_consume(Clock *clock);
void clock_set_target_tps(Clock *clock, u32 target);

u64 clock_next_tick(Clock *clock);

bool clock_should_tick(Clock *clock);

#endif
