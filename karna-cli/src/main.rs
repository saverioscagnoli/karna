use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use sdl3::gpu::ShaderStage;
use sdl3::shadercross::CompileOptions;
use sdl3::shadercross::ShaderCross;

const USAGE: &str = "\
usage: karna <command>

commands:
    shaders [PATH...]    compile *.vert.hlsl / *.frag.hlsl to .spv next to each source
                         (PATH is a file or a directory, default: shaders)
    version              print the karna version";

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        Some("shaders") => shaders(&args[1..]),
        Some("version" | "--version" | "-V") => {
            println!("karna {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("help" | "--help" | "-h") | None => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown command `{other}`\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn stage(path: &Path) -> Option<ShaderStage> {
    let stem = path.file_stem()?.to_str()?;
    let (_, kind) = stem.rsplit_once('.')?;

    match kind {
        "vert" | "vs" | "vertex" => Some(ShaderStage::Vertex),
        "frag" | "fs" | "ps" | "pixel" | "fragment" => Some(ShaderStage::Fragment),
        _ => None,
    }
}

fn is_hlsl(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("hlsl"))
}

fn collect(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let roots = if paths.is_empty() {
        vec![PathBuf::from("shaders")]
    } else {
        paths.iter().map(PathBuf::from).collect()
    };

    let mut files = Vec::new();

    for root in roots {
        if root.is_dir() {
            let entries = fs::read_dir(&root).map_err(|e| format!("{}: {e}", root.display()))?;

            files.extend(
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| path.is_file() && is_hlsl(path) && stage(path).is_some()),
            );
        } else if root.is_file() {
            if stage(&root).is_none() {
                return Err(format!(
                    "{}: cannot tell the stage, name it like `name.vert.hlsl` or `name.frag.hlsl`",
                    root.display()
                ));
            }

            files.push(root);
        } else {
            return Err(format!("{}: no such file or directory", root.display()));
        }
    }

    files.sort();
    files.dedup();

    Ok(files)
}

fn compile(sc: &ShaderCross, path: &Path) -> Result<PathBuf, String> {
    let fail = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());

    let source = fs::read_to_string(path).map_err(|e| fail(&e))?;
    let stage = stage(path).ok_or_else(|| fail(&"unknown shader stage"))?;
    let include_dir = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .and_then(Path::to_str);

    let mut options = CompileOptions::new(stage);

    if let Some(dir) = include_dir {
        options = options.with_include_dir(dir);
    }

    let spirv = sc
        .spirv_from_hlsl(&source, &options)
        .map_err(|e| fail(&e))?;
    let out = path.with_extension("spv");

    fs::write(&out, spirv).map_err(|e| format!("{}: {e}", out.display()))?;

    Ok(out)
}

fn shaders(paths: &[String]) -> ExitCode {
    let files = match collect(paths) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    if files.is_empty() {
        eprintln!("no *.vert.hlsl or *.frag.hlsl files found");
        return ExitCode::FAILURE;
    }

    let sc = match ShaderCross::init() {
        Ok(sc) => sc,
        Err(e) => {
            eprintln!("error: failed to initialize SDL_shadercross: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut failed = 0;

    for file in &files {
        match compile(&sc, file) {
            Ok(out) => println!("{} -> {}", file.display(), out.display()),
            Err(e) => {
                eprintln!("error: {e}");
                failed += 1;
            }
        }
    }

    if failed > 0 {
        eprintln!("{failed} of {} shader(s) failed", files.len());
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
