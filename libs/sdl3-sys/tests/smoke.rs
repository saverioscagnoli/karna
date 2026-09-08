use sdl3_sys::*;

#[test]
fn links_and_reports_version() {
    let v = unsafe { SDL_GetVersion() };
    let major = v / 1_000_000;
    let minor = (v / 1_000) % 1_000;
    assert_eq!(major, 3, "expected SDL3, got version {v}");
    println!("linked against SDL {major}.{minor}.{}", v % 1_000);
}

#[test]
fn init_and_quit_events_subsystem() {
    unsafe {
        assert!(SDL_Init(SDL_INIT_EVENTS), "SDL_Init failed");
        SDL_Quit();
    }
}
