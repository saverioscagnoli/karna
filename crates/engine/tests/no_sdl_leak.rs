//! Enforces that SDL stays behind the platform boundary.
//!
//! The point of `events::event` is that swapping SDL for another backend
//! touches one file. That only holds if `SDL_` never appears outside the
//! modules allowed to own it, and nothing but a test will keep it that way —
//! the first `use sdl3::SDL_Event` in a renderer compiles perfectly well.
//!
//! Add a path to `ALLOWED` only when the module is genuinely part of the
//! platform layer.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Modules permitted to name SDL types directly.
const ALLOWED: &[&str] = &[
    "events/poll.rs", // the translation layer itself
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();

        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn sdl_types_stay_in_the_platform_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);

    assert!(
        !files.is_empty(),
        "found no sources under {}",
        root.display()
    );

    let mut leaks = Vec::new();

    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("path under src")
            .to_string_lossy()
            .replace('\\', "/");

        if ALLOWED.contains(&rel.as_str()) {
            continue;
        }

        let src = fs::read_to_string(&file).expect("read source");

        for (i, line) in src.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");

            if code.contains("SDL_") || code.contains("sdl3::") {
                leaks.push(format!("{rel}:{} -- {}", i + 1, line.trim()));
            }
        }
    }

    assert!(
        leaks.is_empty(),
        "SDL types escaped the platform layer.\n\
         Translate them in events/poll.rs, or add the module to ALLOWED if it \
         is genuinely part of that layer.\n  {}",
        leaks.join("\n  ")
    );
}
