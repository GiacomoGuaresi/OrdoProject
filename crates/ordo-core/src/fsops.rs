//! Low-level file operations: robust moves, destination folders, free names,
//! content comparison and the system trash.

use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use crate::model::{ErrorCode, OpError};

/// Returns the root of the volume that `path` lives on.
///
/// On Windows this is the drive or UNC share (`C:\`, `\\server\share\`).
/// On Unix it is the mount point for paths under well-known mount containers
/// (`/Volumes/<name>`, `/mnt/<name>`, `/media/[<user>/]<name>`,
/// `/run/media/<user>/<name>`), and `/` otherwise.
pub fn volume_root(path: &Path) -> PathBuf {
    let comps: Vec<Component> = path.components().collect();
    if cfg!(windows) {
        return comps
            .iter()
            .take_while(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
            .collect();
    }
    let names: Vec<&str> = comps
        .iter()
        .filter_map(|c| match c {
            Component::Normal(n) => n.to_str(),
            _ => None,
        })
        .collect();
    let take = match names.as_slice() {
        ["Volumes", _, ..] | ["mnt", _, ..] => 2,
        ["run", "media", _, _, ..] => 4,
        ["media", first, _, ..] if !is_mount_point(&Path::new("/media").join(first)) => 3,
        ["media", _, ..] => 2,
        _ => 0,
    };
    let mut root = PathBuf::from("/");
    root.extend(&names[..take]);
    root
}

#[cfg(unix)]
fn is_mount_point(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let (Ok(meta), Some(parent)) = (fs::metadata(path), path.parent()) else {
        return false;
    };
    fs::metadata(parent).is_ok_and(|p| p.dev() != meta.dev())
}

#[cfg(not(unix))]
fn is_mount_point(_path: &Path) -> bool {
    false
}

/// Makes sure `dir` exists, creating it when its volume is available.
///
/// Never creates "ghost" folders for unmounted volumes: in that case
/// [`ErrorCode::VolumeMissing`] is returned.
pub fn ensure_dir(dir: &Path) -> Result<(), OpError> {
    if dir.is_dir() {
        return Ok(());
    }
    let root = volume_root(dir);
    if !root.is_dir() {
        return Err(OpError::new(
            ErrorCode::VolumeMissing,
            root.display().to_string(),
        ));
    }
    fs::create_dir_all(dir).map_err(|e| OpError::new(ErrorCode::Io, e.to_string()))
}

fn is_cross_device(err: &io::Error) -> bool {
    const EXDEV: i32 = 18; // Unix
    const ERROR_NOT_SAME_DEVICE: i32 = 17; // Windows
    err.kind() == io::ErrorKind::CrossesDevices
        || match err.raw_os_error() {
            Some(code) if cfg!(windows) => code == ERROR_NOT_SAME_DEVICE,
            Some(code) => code == EXDEV,
            None => false,
        }
}

/// Moves `src` to `dst`. The caller guarantees that `dst` does not exist and
/// that its parent folder exists.
///
/// Uses a rename when possible and falls back to copy + verify + remove when
/// the two paths are on different volumes.
pub fn move_file(src: &Path, dst: &Path) -> Result<(), OpError> {
    if !src.exists() {
        return Err(OpError::new(
            ErrorCode::SourceMissing,
            src.display().to_string(),
        ));
    }
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) if is_cross_device(&e) => copy_verify_remove(src, dst),
        Err(e) => Err(OpError::io(&e)),
    }
}

/// Copies `src` to `dst`, checks that the copy is identical, keeps the
/// modification time and finally removes `src`. On any failure the partial
/// copy is removed and `src` is left untouched.
pub fn copy_verify_remove(src: &Path, dst: &Path) -> Result<(), OpError> {
    let rollback = |err: OpError| {
        let _ = fs::remove_file(dst);
        Err(err)
    };
    let io_err = |e: io::Error| OpError::new(ErrorCode::Io, e.to_string());

    if let Err(e) = fs::copy(src, dst) {
        return rollback(OpError::io(&e));
    }
    match files_identical(src, dst) {
        Ok(true) => {}
        Ok(false) => {
            return rollback(OpError::new(
                ErrorCode::Io,
                format!("copy verification failed for {}", dst.display()),
            ));
        }
        Err(e) => return rollback(io_err(e)),
    }
    if let Ok(modified) = fs::metadata(src).and_then(|m| m.modified())
        && let Ok(f) = fs::File::options().write(true).open(dst)
    {
        let _ = f.set_modified(modified);
    }
    if let Err(e) = fs::remove_file(src) {
        return rollback(io_err(e));
    }
    Ok(())
}

/// Returns `path` if free, otherwise the first free `name (n).ext` next to it.
pub fn unique_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let parent = path.parent().unwrap_or(Path::new(""));
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (1..)
        .map(|n| parent.join(format!("{stem} ({n}){ext}")))
        .find(|p| !p.exists())
        .expect("an unbounded range always yields a free name")
}

/// Whether the two files have the same content.
pub fn files_identical(a: &Path, b: &Path) -> io::Result<bool> {
    if fs::metadata(a)?.len() != fs::metadata(b)?.len() {
        return Ok(false);
    }
    Ok(hash_file(a)? == hash_file(b)?)
}

fn hash_file(path: &Path) -> io::Result<blake3::Hash> {
    let mut hasher = blake3::Hasher::new();
    let mut file = fs::File::open(path)?;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize())
}

/// Moves a file to the system trash.
pub fn trash_file(path: &Path) -> Result<(), OpError> {
    #[allow(unused_mut)]
    let mut ctx = trash::TrashContext::default();
    #[cfg(target_os = "macos")]
    {
        // Avoids driving Finder via AppleScript (permission prompt, sound).
        use trash::macos::{DeleteMethod, TrashContextExtMacos};
        ctx.set_delete_method(DeleteMethod::NsFileManager);
    }
    ctx.delete(path)
        .map_err(|e| OpError::new(ErrorCode::Trash, e.to_string()))
}

/// Outcome of an attempt to restore a file from the trash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrashRestore {
    Restored,
    /// The platform cannot restore programmatically, or the item was not found:
    /// the user has to restore it manually from the trash.
    Manual,
}

/// Tries to put back the most recently trashed item whose original location
/// was `original`.
#[cfg(any(windows, all(unix, not(target_os = "macos"))))]
pub fn restore_from_trash(original: &Path) -> TrashRestore {
    let Ok(items) = trash::os_limited::list() else {
        return TrashRestore::Manual;
    };
    let Some(item) = items
        .into_iter()
        .filter(|i| i.original_path() == original)
        .max_by_key(|i| i.time_deleted)
    else {
        return TrashRestore::Manual;
    };
    match trash::os_limited::restore_all([item]) {
        Ok(()) => TrashRestore::Restored,
        Err(_) => TrashRestore::Manual,
    }
}

/// Tries to put back the most recently trashed item whose original location
/// was `original`.
#[cfg(target_os = "macos")]
pub fn restore_from_trash(_original: &Path) -> TrashRestore {
    TrashRestore::Manual
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_path_appends_counter() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("report.pdf");
        assert_eq!(unique_path(&p), p);
        fs::write(&p, "").unwrap();
        assert_eq!(unique_path(&p), tmp.path().join("report (1).pdf"));
        fs::write(tmp.path().join("report (1).pdf"), "").unwrap();
        assert_eq!(unique_path(&p), tmp.path().join("report (2).pdf"));

        let noext = tmp.path().join("README");
        fs::write(&noext, "").unwrap();
        assert_eq!(unique_path(&noext), tmp.path().join("README (1)"));
    }

    #[test]
    fn move_file_renames() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("a.txt");
        let dst = tmp.path().join("b.txt");
        fs::write(&src, "hello").unwrap();
        move_file(&src, &dst).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello");
    }

    #[test]
    fn move_file_missing_source() {
        let tmp = tempfile::tempdir().unwrap();
        let err = move_file(&tmp.path().join("x"), &tmp.path().join("y")).unwrap_err();
        assert_eq!(err.code, ErrorCode::SourceMissing);
    }

    #[test]
    fn copy_verify_remove_moves_content_and_mtime() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("a.bin");
        let dst = tmp.path().join("b.bin");
        let data: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        fs::write(&src, &data).unwrap();
        let old = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
        fs::File::options()
            .write(true)
            .open(&src)
            .unwrap()
            .set_modified(old)
            .unwrap();

        copy_verify_remove(&src, &dst).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read(&dst).unwrap(), data);
        assert_eq!(fs::metadata(&dst).unwrap().modified().unwrap(), old);
    }

    #[test]
    fn copy_verify_remove_rolls_back_on_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("a.txt");
        fs::write(&src, "x").unwrap();
        // Destination parent does not exist: copy fails, source untouched.
        let dst = tmp.path().join("missing/b.txt");
        assert!(copy_verify_remove(&src, &dst).is_err());
        assert!(src.exists());
    }

    #[test]
    fn identical_files() {
        let tmp = tempfile::tempdir().unwrap();
        let (a, b, c) = (
            tmp.path().join("a"),
            tmp.path().join("b"),
            tmp.path().join("c"),
        );
        fs::write(&a, "same").unwrap();
        fs::write(&b, "same").unwrap();
        fs::write(&c, "diff").unwrap();
        assert!(files_identical(&a, &b).unwrap());
        assert!(!files_identical(&a, &c).unwrap());
        fs::write(&c, "longer content").unwrap();
        assert!(!files_identical(&a, &c).unwrap());
    }

    #[test]
    fn ensure_dir_creates_on_present_volume() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path().join("x/y/z");
        ensure_dir(&d).unwrap();
        assert!(d.is_dir());
        ensure_dir(&d).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn volume_roots_unix() {
        assert_eq!(volume_root(Path::new("/home/u/Docs")), PathBuf::from("/"));
        assert_eq!(
            volume_root(Path::new("/Volumes/USB/Photos/2024")),
            PathBuf::from("/Volumes/USB")
        );
        assert_eq!(
            volume_root(Path::new("/mnt/data/x")),
            PathBuf::from("/mnt/data")
        );
        assert_eq!(
            volume_root(Path::new("/run/media/user/DISK/x")),
            PathBuf::from("/run/media/user/DISK")
        );
        assert_eq!(volume_root(Path::new("/Volumes")), PathBuf::from("/"));
    }

    #[cfg(unix)]
    #[test]
    fn ensure_dir_refuses_missing_volume() {
        let err = ensure_dir(Path::new("/Volumes/ordo-test-not-mounted/Photos")).unwrap_err();
        assert_eq!(err.code, ErrorCode::VolumeMissing);
        assert!(!Path::new("/Volumes/ordo-test-not-mounted").exists());
    }

    #[cfg(windows)]
    #[test]
    fn volume_roots_windows() {
        assert_eq!(
            volume_root(Path::new(r"C:\Users\u\Docs")),
            PathBuf::from(r"C:\")
        );
        assert_eq!(
            volume_root(Path::new(r"\\server\share\dir")),
            PathBuf::from(r"\\server\share\")
        );
    }

    #[cfg(windows)]
    #[test]
    fn ensure_dir_refuses_missing_drive() {
        // Find an unused drive letter.
        let free = ('D'..='Z')
            .map(|c| format!("{c}:\\"))
            .find(|d| !Path::new(d).exists());
        if let Some(drive) = free {
            let err = ensure_dir(&Path::new(&drive).join("Photos")).unwrap_err();
            assert_eq!(err.code, ErrorCode::VolumeMissing);
        }
    }
}
