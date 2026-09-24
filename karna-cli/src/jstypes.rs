use std::fs;
use std::path::PathBuf;

const TYPES: &str = include_str!("../../assets/karna.d.ts");

pub fn exec(path: Option<&PathBuf>) -> Result<(), String> {
    let path = path
        .map(|p| p.to_path_buf())
        .unwrap_or(PathBuf::from("./karna.d.ts"));

    fs::write(path, TYPES).map_err(|e| e.to_string())
}
