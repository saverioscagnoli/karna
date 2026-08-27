use std::path::PathBuf;

use logging::fatal;
use logging::warn;
use sdl3::SDL_GetBasePath;
use utils::cstr_to_pathbuf;

use crate::err::sdl_last_error;

fn sdl_base_path() -> Result<PathBuf, String> {
    let raw = unsafe { SDL_GetBasePath() };

    if raw.is_null() {
        return Err(sdl_last_error());
    }

    Ok(unsafe { cstr_to_pathbuf(raw) })
}

pub fn resolve_base_path() -> PathBuf {
    match sdl_base_path() {
        Ok(p) => return p,
        Err(e) => warn!("SDL_GetBasePath failed ({e}), falling back to current_exe"),
    }

    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => fatal!("Failed to resolve base path: {}", e),
    };

    match exe.parent() {
        Some(dir) => dir.to_path_buf(),
        None => fatal!("Executable path has no parent directory: {}", exe.display()),
    }
}
