//! User settings stored in `settings.json`.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::lang::Lang;
use crate::paths::write_atomic;

/// Current version of the `settings.json` schema.
pub const SETTINGS_FILE_VERSION: u32 = 1;

/// Default history retention in days.
pub const DEFAULT_RETENTION_DAYS: u32 = 90;

/// Color theme of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// User settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub version: u32,
    /// Language; `None` follows the operating system.
    pub language: Option<Lang>,
    pub theme: Theme,
    /// History retention in days; `None` keeps history forever.
    pub retention_days: Option<u32>,
    /// Check GitHub Releases for new versions when the app starts.
    pub check_updates: bool,
    /// Whether the first-launch onboarding has been completed.
    pub onboarding_done: bool,
    /// Folders sorted by the "Ordo" launcher; empty means the Desktop.
    pub quick_sort_folders: Vec<std::path::PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_FILE_VERSION,
            language: None,
            theme: Theme::System,
            retention_days: Some(DEFAULT_RETENTION_DAYS),
            check_updates: true,
            onboarding_done: false,
            quick_sort_folders: Vec::new(),
        }
    }
}

impl Settings {
    /// Loads settings from `path`; a missing file yields the defaults.
    pub fn load(path: &Path) -> Result<Self> {
        let json = match std::fs::read_to_string(path) {
            Ok(json) => json,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(Error::io(path, e)),
        };
        let settings: Settings = serde_json::from_str(&json).map_err(|source| Error::Json {
            path: path.to_path_buf(),
            source,
        })?;
        if settings.version > SETTINGS_FILE_VERSION {
            return Err(Error::UnsupportedVersion {
                path: path.to_path_buf(),
                found: settings.version,
                supported: SETTINGS_FILE_VERSION,
            });
        }
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).expect("settings are always serializable");
        write_atomic(path, json.as_bytes())
    }

    /// The effective language (configured or system).
    pub fn effective_lang(&self) -> Lang {
        self.language.unwrap_or_else(Lang::system)
    }

    /// Folders sorted by the "Ordo" launcher: the configured ones, or the Desktop.
    pub fn quick_sort_targets(&self) -> Vec<std::path::PathBuf> {
        if !self.quick_sort_folders.is_empty() {
            return self.quick_sort_folders.clone();
        }
        directories::UserDirs::new()
            .map(|d| {
                d.desktop_dir()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| d.home_dir().join("Desktop"))
            })
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let s = Settings::load(&tmp.path().join("settings.json")).unwrap();
        assert_eq!(s, Settings::default());
        assert_eq!(s.retention_days, Some(90));
    }

    #[test]
    fn roundtrip_and_partial_files() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("settings.json");
        let s = Settings {
            language: Some(Lang::It),
            theme: Theme::Dark,
            retention_days: None,
            ..Settings::default()
        };
        s.save(&file).unwrap();
        assert_eq!(Settings::load(&file).unwrap(), s);

        std::fs::write(&file, r#"{"theme":"light"}"#).unwrap();
        let s = Settings::load(&file).unwrap();
        assert_eq!(s.theme, Theme::Light);
        assert!(s.check_updates);
        assert_eq!(s.effective_lang(), s.language.unwrap_or_else(Lang::system));
    }

    #[test]
    fn quick_sort_targets_default_to_desktop() {
        let s = Settings::default();
        let targets = s.quick_sort_targets();
        assert_eq!(targets.len(), 1);
        let custom = Settings {
            quick_sort_folders: vec!["/a".into(), "/b".into()],
            ..Settings::default()
        };
        assert_eq!(
            custom.quick_sort_targets(),
            vec![std::path::PathBuf::from("/a"), "/b".into()]
        );
    }

    #[test]
    fn invalid_json_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("settings.json");
        std::fs::write(&file, "{").unwrap();
        assert!(matches!(Settings::load(&file), Err(Error::Json { .. })));
    }
}
