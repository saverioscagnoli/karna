use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;

use nostd::fs::Archive;

const MAGIC: &[u8; 8] = b"KARNABND";
const FOOTER_LEN: u64 = 16;

trait At<T> {
    fn at(self, path: &Path) -> io::Result<T>;
}

impl<T> At<T> for io::Result<T> {
    fn at(self, path: &Path) -> io::Result<T> {
        self.map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
    }
}

fn walk_dir<F>(path: &Path, f: &mut F) -> io::Result<()>
where
    F: FnMut(&Path) -> io::Result<()>,
{
    for entry in fs::read_dir(path).at(path)? {
        let entry = entry.at(path)?;

        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        let ft = entry.file_type().at(&entry.path())?;

        if ft.is_dir() {
            walk_dir(&entry.path(), f)?;
        } else if ft.is_file() {
            f(&entry.path())?;
        }
    }

    Ok(())
}

fn footer(file: &mut File) -> Option<u64> {
    let len = file.seek(SeekFrom::End(0)).ok()?;

    if len < FOOTER_LEN {
        return None;
    }

    let mut footer = [0u8; FOOTER_LEN as usize];
    file.seek(SeekFrom::End(-(FOOTER_LEN as i64))).ok()?;
    file.read_exact(&mut footer).ok()?;

    if &footer[8..] != MAGIC {
        return None;
    }

    let offset = u64::from_le_bytes(footer[..8].try_into().unwrap());

    (offset <= len - FOOTER_LEN).then_some(offset)
}

pub fn embedded() -> Option<&'static Archive> {
    let mut file = File::open(std::env::current_exe().ok()?).ok()?;
    let offset = footer(&mut file)?;
    let len = file.seek(SeekFrom::End(0)).ok()? - FOOTER_LEN - offset;

    let mut bytes = vec![0; usize::try_from(len).ok()?];

    file.seek(SeekFrom::Start(offset)).ok()?;
    file.read_exact(&mut bytes).ok()?;

    let archive = Archive::parse(Box::leak(bytes.into_boxed_slice())).ok()?;

    Some(Box::leak(Box::new(archive)))
}

pub fn exec(path: &Path) -> io::Result<()> {
    let main = path.join("main.js");

    if !main.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "no main.js")).at(path);
    }

    let mut files = Vec::new();

    walk_dir(path, &mut |file| {
        let rel = file.strip_prefix(path).map_err(io::Error::other).at(file)?;
        let rel = rel
            .to_str()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidFilename, "path is not valid UTF-8")
            })
            .at(file)?
            .to_string();

        files.push((rel, fs::read(file).at(file)?));
        Ok(())
    })?;

    let archive = nostd::fs::archive::write(files.iter().map(|(p, d)| (p.as_str(), d.as_slice())));

    let exe_path = std::env::current_exe()?;
    let mut exe = fs::read(&exe_path).at(&exe_path)?;

    if let Some(offset) = footer(&mut File::open(&exe_path).at(&exe_path)?) {
        exe.truncate(offset as usize);
    }

    let offset = exe.len() as u64;

    exe.extend_from_slice(&archive);
    exe.extend_from_slice(&offset.to_le_bytes());
    exe.extend_from_slice(MAGIC);

    let name = path
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "game".to_string());

    let dist = PathBuf::from("dist");
    fs::create_dir_all(&dist).at(&dist)?;

    let mut dst = dist.join(name);

    if cfg!(windows) {
        dst.set_extension("exe");
    }

    fs::write(&dst, &exe).at(&dst)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&dst, fs::Permissions::from_mode(0o755)).at(&dst)?;
    }

    println!(
        "bundled {} files ({} bytes) from {} -> {}",
        files.len(),
        archive.len(),
        path.display(),
        dst.display()
    );

    Ok(())
}
