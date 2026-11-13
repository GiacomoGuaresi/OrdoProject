//! Collects the files to sort from the paths given by the user.
//!
//! - A file is taken as is.
//! - A folder contributes only the files at its first level; subfolders are
//!   never entered.
//! - Hidden and system files are always excluded, even when selected
//!   explicitly. Symbolic links are excluded too.

use std::collections::HashSet;
use std::fs::Metadata;
use std::path::{Path, PathBuf};

/// A file to sort, or an input path that could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanEntry {
    File(PathBuf),
    Error { path: PathBuf, message: String },
}

/// File names that are always treated as system files.
const SYSTEM_NAMES: &[&str] = &[
    ".DS_Store",
    ".localized",
    "desktop.ini",
    "Desktop.ini",
    "Thumbs.db",
    "thumbs.db",
    "ehthumbs.db",
    "Icon\r",
    "$RECYCLE.BIN",
];

/// Returns the files to sort for the given input paths, in input order
/// (folder contents sorted by name), without duplicates.
pub fn collect(paths: &[PathBuf]) -> Vec<ScanEntry> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut push = |out: &mut Vec<ScanEntry>, path: PathBuf| {
        if seen.insert(path.clone()) {
            out.push(ScanEntry::File(path));
        }
    };

    for input in paths {
        let path = std::path::absolute(input).unwrap_or_else(|_| input.clone());
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                out.push(ScanEntry::Error {
                    path,
                    message: e.to_string(),
                });
                continue;
            }
        };
        if meta.is_dir() {
            let mut files = match first_level_files(&path) {
                Ok(f) => f,
                Err(e) => {
                    out.push(ScanEntry::Error {
                        path,
                        message: e.to_string(),
                    });
                    continue;
                }
            };
            files.sort();
            for f in files {
                push(&mut out, f);
            }
        } else if meta.is_file() && !is_hidden(&path, &meta) {
            push(&mut out, path);
        }
    }
    out
}

fn first_level_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let Ok(meta) = entry.metadata() else { continue };
        let path = entry.path();
        if meta.is_file() && !is_hidden(&path, &meta) {
            files.push(path);
        }
    }
    Ok(files)
}

/// Whether the file is hidden or a system file on the current platform.
pub fn is_hidden(path: &Path, meta: &Metadata) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        // Non UTF-8 names cannot be matched by rules anyway.
        return true;
    };
    if name.starts_with('.') || SYSTEM_NAMES.contains(&name) || name.starts_with("~$") {
        return true;
    }
    has_hidden_attribute(meta)
}

#[cfg(windows)]
fn has_hidden_attribute(meta: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
    meta.file_attributes() & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) != 0
}

#[cfg(target_os = "macos")]
fn has_hidden_attribute(meta: &Metadata) -> bool {
    use std::os::macos::fs::MetadataExt;
    const UF_HIDDEN: u32 = 0x8000;
    meta.st_flags() & UF_HIDDEN != 0
}

#[cfg(not(any(windows, target_os = "macos")))]
fn has_hidden_attribute(_meta: &Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn files(entries: &[ScanEntry]) -> Vec<String> {
        entries
            .iter()
            .filter_map(|e| match e {
                ScanEntry::File(p) => Some(p.file_name().unwrap().to_string_lossy().into_owned()),
                ScanEntry::Error { .. } => None,
            })
            .collect()
    }

    #[test]
    fn folder_contributes_first_level_visible_files_only() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        fs::write(d.join("b.txt"), "").unwrap();
        fs::write(d.join("a.pdf"), "").unwrap();
        fs::write(d.join(".hidden"), "").unwrap();
        fs::write(d.join(".DS_Store"), "").unwrap();
        fs::write(d.join("desktop.ini"), "").unwrap();
        fs::write(d.join("~$lock.docx"), "").unwrap();
        fs::create_dir(d.join("sub")).unwrap();
        fs::write(d.join("sub/inner.txt"), "").unwrap();

        let out = collect(&[d.to_path_buf()]);
        assert_eq!(files(&out), ["a.pdf", "b.txt"]);
    }

    #[test]
    fn explicit_files_hidden_excluded_and_deduplicated() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        fs::write(d.join("a.txt"), "").unwrap();
        fs::write(d.join(".secret"), "").unwrap();
        let out = collect(&[d.join("a.txt"), d.join(".secret"), d.to_path_buf()]);
        assert_eq!(files(&out), ["a.txt"]);
    }

    #[test]
    fn missing_paths_are_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let out = collect(&[tmp.path().join("nope.txt")]);
        assert!(matches!(out.as_slice(), [ScanEntry::Error { .. }]));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path();
        fs::write(d.join("real.txt"), "").unwrap();
        std::os::unix::fs::symlink(d.join("real.txt"), d.join("link.txt")).unwrap();
        let out = collect(&[d.to_path_buf(), d.join("link.txt")]);
        assert_eq!(files(&out), ["real.txt"]);
    }

    #[test]
    fn returns_absolute_paths() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a.txt"), "").unwrap();
        let out = collect(&[tmp.path().join("a.txt")]);
        let ScanEntry::File(p) = &out[0] else {
            panic!()
        };
        assert!(p.is_absolute());
    }
}
