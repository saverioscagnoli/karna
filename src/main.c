#include <SDL3/SDL.h>
#include <SDL3/SDL_main.h>
#include <SDL3/SDL_oldnames.h>
#include "../math/vector.h"
#include "../logging/log.h"
#include "../types.h"
#include <SDL3/SDL_stdinc.h>
#include <SDL3/SDL_timer.h>

const Uint64 FRAME_NS = SDL_NS_PER_SECOND / 60; // target 60fps

int main(int argc, char *argv[]) {
    (void)argc;
    (void)argv;

    log_set_level_from_env("KARNA_LOG");
    log_set_level(LOG_DEBUG);

    if (!SDL_Init(SDL_INIT_VIDEO)) {
        FATAL("SDL_Init failed: %s", SDL_GetError());
        return 1;
    }

    SDL_Window *window = NULL;
    SDL_Renderer *renderer = NULL;
    if (!SDL_CreateWindowAndRenderer("karna", 800, 600, 0, &window, &renderer)) {
        FATAL("SDL_CreateWindowAndRenderer failed: %s", SDL_GetError());
        SDL_Quit();
        return 1;
    }

    DEBUG("BRUH>>");
    INFO("Created window 'karna' (%dx%d)", 800, 600);

    //    SDL_SetRenderVSync(renderer, 1);

    bool running = true;

    u64 freq = SDL_GetPerformanceFrequency();
    u64 last = SDL_GetPerformanceCounter();

    while (running) {
        u64 frame_start = SDL_GetTicksNS();
        f32 dt = (f32)(frame_start - last) / 1e9f;

        if (dt > 0.05f)
            dt = 0.05f;
        last = frame_start;

        SDL_Event e;
        while (SDL_PollEvent(&e)) {
            if (e.type == SDL_EVENT_QUIT)
                running = false;
            if (e.type == SDL_EVENT_KEY_DOWN && e.key.key == SDLK_ESCAPE)
                running = false;
        }

        SDL_SetRenderDrawColor(renderer, 30, 30, 46, 255);
        SDL_RenderClear(renderer);

        SDL_SetRenderDrawColor(renderer, 137, 180, 250, 255);
        SDL_FRect r = {350.0f, 250.0f, 100.0f, 100.0f};
        SDL_RenderFillRect(renderer, &r);

        SDL_RenderPresent(renderer); // <-- this is the buffer commit
        Uint64 elapsed = SDL_GetTicksNS() - frame_start;
        if (elapsed < FRAME_NS) {
            SDL_DelayNS(FRAME_NS - elapsed);
        }
    }

    DEBUG("Quit signal received");

    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    SDL_Quit();

    INFO("Lifecycle loop over. Exiting.");
    return 0;
}
