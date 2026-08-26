//! Requires every module to say what it is for.
//!
//! `//!` headers are the only documentation that reliably survives, because
//! they sit in the same diff as the code they describe. Nothing enforces them
//! though — a new file compiles perfectly well with no explanation at all, and
//! six months later nobody remembers why it exists.
//!
//! This checks presence, not quality. A header reading "the window module" is
//! worthless and passes. See ARCHITECTURE.md for what a good one covers.
//!
//! Empty placeholder files are skipped: a header on a file with no code yet is
//! a guess, and a wrong guess is worse than silence.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

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

/// A file documents itself if its first meaningful line is `//!`.
///
/// Inner attributes (`#![no_std]`) and blank lines may precede it; anything
/// else means the header is missing.
fn has_header(src: &str) -> bool {
    for line in src.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with("#![") {
            continue;
        }

        return line.starts_with("//!");
    }

    false
}

#[test]
fn every_module_explains_itself() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);

    assert!(
        !files.is_empty(),
        "found no sources under {}",
        root.display()
    );

    let mut undocumented = Vec::new();

    for file in files {
        let src = fs::read_to_string(&file).expect("read source");

        // Placeholder with no code yet — nothing to describe.
        if src.split_whitespace().next().is_none() {
            continue;
        }

        if !has_header(&src) {
            let rel = file.strip_prefix(&root).expect("path under src");
            undocumented.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }

    undocumented.sort();

    assert!(
        undocumented.is_empty(),
        "these modules have no `//!` header saying what they are for:\n  {}",
        undocumented.join("\n  ")
    );
}
