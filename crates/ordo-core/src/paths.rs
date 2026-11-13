//! Locations of Ordo's configuration and data files.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Environment variable that overrides every Ordo directory (used by tests and
/// portable setups). Config and data files are then stored directly inside it.
pub const HOME_ENV: &str = "ORDO_HOME";

/// Directories where Ordo stores its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// Directory holding `rules.json` and `settings.json`.
    pub config_dir: PathBuf,
    /// Directory holding `history.sqlite`.
    pub data_dir: PathBuf,
}

impl AppPaths {
    /// Resolves the standard per-user directories for the current OS, or
    /// `$ORDO_HOME` when set.
    pub fn resolve() -> Result<Self> {
        if let Some(home) = std::env::var_os(HOME_ENV).filter(|v| !v.is_empty()) {
            return Ok(Self::in_dir(PathBuf::from(home)));
        }
        let dirs = directories::ProjectDirs::from("io.github", "giacomoguaresi", "Ordo")
            .ok_or(Error::NoHomeDir)?;
        Ok(Self {
            config_dir: dirs.config_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
        })
    }

    /// Stores everything inside a single directory.
    pub fn in_dir(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        Self {
            config_dir: dir.clone(),
            data_dir: dir,
        }
    }

    pub fn rules_file(&self) -> PathBuf {
        self.config_dir.join("rules.json")
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn history_db(&self) -> PathBuf {
        self.data_dir.join("history.sqlite")
    }
}

/// Writes `contents` to `path` atomically (temp file + rename), creating the
/// parent directory if needed.
pub(crate) fn write_atomic(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, contents).map_err(|e| Error::io(&tmp, e))?;
    std::fs::rename(&tmp, path).map_err(|e| Error::io(path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_dir_puts_everything_together() {
        let p = AppPaths::in_dir("/x");
        assert_eq!(p.rules_file(), PathBuf::from("/x/rules.json"));
        assert_eq!(p.settings_file(), PathBuf::from("/x/settings.json"));
        assert_eq!(p.history_db(), PathBuf::from("/x/history.sqlite"));
    }

    #[test]
    fn write_atomic_creates_parent() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("a/b/c.json");
        write_atomic(&file, b"{}").unwrap();
        assert_eq!(std::fs::read(&file).unwrap(), b"{}");
        assert!(!file.with_extension("tmp").exists());
    }
}
