//! Integration with the operating system:
//!
//! - the "Sort with Ordo" entry in the file manager's context menu
//!   (macOS Quick Action, Windows Explorer verbs, Nautilus and Dolphin);
//! - the `ordo` command in the terminal's `PATH`.
//!
//! Everything is installed per user, without administrator rights where
//! possible.

pub mod command;
pub mod launcher;
pub mod linux;
pub mod macos;
#[cfg(windows)]
pub mod windows;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::lang::Lang;

/// Command-line flag understood by the desktop app to sort paths.
pub const SORT_FLAG: &str = "--sort";

/// One context-menu integration (for one file manager).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct IntegrationItem {
    /// Stable identifier: `macos-quick-action`, `windows-explorer`, `nautilus`, `dolphin`.
    pub id: String,
    /// Human-readable name of the file manager.
    pub name: String,
    /// The file manager is present on this system.
    pub available: bool,
    pub installed: bool,
    /// Where the integration lives (folder or registry key).
    pub location: String,
}

/// State of the context-menu integrations for the current platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct IntegrationStatus {
    pub items: Vec<IntegrationItem>,
}

impl IntegrationStatus {
    /// Whether every available integration is installed (and at least one is).
    pub fn fully_installed(&self) -> bool {
        let available: Vec<_> = self.items.iter().filter(|i| i.available).collect();
        !available.is_empty() && available.iter().all(|i| i.installed)
    }

    pub fn any_installed(&self) -> bool {
        self.items.iter().any(|i| i.installed)
    }
}

/// Label of the context-menu entry.
pub fn menu_label(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Sort with Ordo",
        Lang::It => "Ordina con Ordo",
    }
}

/// Context-menu entry and "Ordo" launcher for the current user.
#[derive(Debug, Clone)]
pub struct Integration {
    /// Executable of the desktop app, started with `--sort <paths…>`.
    pub app_exe: PathBuf,
    /// Language of the menu entry.
    pub lang: Lang,
    /// Home folder of the user (overridable for tests).
    pub home: PathBuf,
}

impl Integration {
    pub fn new(app_exe: impl Into<PathBuf>, lang: Lang) -> Result<Self> {
        let home = directories::BaseDirs::new()
            .ok_or(Error::NoHomeDir)?
            .home_dir()
            .to_path_buf();
        Ok(Self {
            app_exe: app_exe.into(),
            lang,
            home,
        })
    }

    pub fn status(&self) -> IntegrationStatus {
        let items = if cfg!(target_os = "macos") {
            vec![macos::status(&self.home)]
        } else if cfg!(windows) {
            #[cfg(windows)]
            {
                vec![windows::status(windows::CLASSES_ROOT)]
            }
            #[cfg(not(windows))]
            {
                Vec::new()
            }
        } else {
            linux::status(&self.home)
        };
        let mut items = items;
        items.push(launcher::status(&self.home));
        IntegrationStatus { items }
    }

    /// Installs the entry for every available file manager.
    pub fn install(&self) -> Result<IntegrationStatus> {
        if !self.app_exe.is_file() {
            return Err(Error::Integration(format!(
                "the app executable {} does not exist",
                self.app_exe.display()
            )));
        }
        if cfg!(target_os = "macos") {
            macos::install(&self.home, &self.app_exe, self.lang)?;
        } else if cfg!(windows) {
            #[cfg(windows)]
            windows::install(windows::CLASSES_ROOT, &self.app_exe, self.lang)?;
        } else {
            linux::install(&self.home, &self.app_exe, self.lang, &linux::detect())?;
        }
        launcher::install(&self.home, &self.app_exe)?;
        Ok(self.status())
    }

    pub fn uninstall(&self) -> Result<IntegrationStatus> {
        if cfg!(target_os = "macos") {
            macos::uninstall(&self.home)?;
        } else if cfg!(windows) {
            #[cfg(windows)]
            windows::uninstall(windows::CLASSES_ROOT)?;
        } else {
            linux::uninstall(&self.home)?;
        }
        launcher::uninstall(&self.home)?;
        Ok(self.status())
    }
}

/// Quotes a path for a POSIX shell script.
pub(crate) fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', r"'\''"))
}

pub(crate) fn write_file(path: &Path, contents: &str, executable: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    std::fs::write(path, contents).map_err(|e| Error::io(path, e))?;
    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| Error::io(path, e))?;
    }
    #[cfg(not(unix))]
    let _ = executable;
    Ok(())
}

pub(crate) fn remove_path(path: &Path) -> Result<()> {
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io(path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting() {
        assert_eq!(shell_quote(Path::new("/a b/c")), "'/a b/c'");
        assert_eq!(shell_quote(Path::new("/it's")), r"'/it'\''s'");
    }

    #[test]
    fn status_helpers() {
        let item = |available, installed| IntegrationItem {
            id: "x".into(),
            name: "X".into(),
            available,
            installed,
            location: String::new(),
        };
        let s = IntegrationStatus {
            items: vec![item(true, true), item(false, false)],
        };
        assert!(s.fully_installed());
        assert!(s.any_installed());
        let s = IntegrationStatus {
            items: vec![item(true, false)],
        };
        assert!(!s.fully_installed());
        assert!(!IntegrationStatus { items: vec![] }.fully_installed());
    }
}
