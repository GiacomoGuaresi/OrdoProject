//! Types shared by sorting, history and undo.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Where a sort was started from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Cli,
    ContextMenu,
    App,
}

/// State of a single entry of a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum MoveStatus {
    /// The file was moved.
    Done,
    /// The move was undone.
    Undone,
    /// The move cannot be undone (e.g. the file is no longer at the destination).
    NotUndoable,
    /// The user chose to skip the file because of a name conflict.
    Skipped,
    /// No rule matched: the file was left in place.
    Ignored,
    /// The file was already in the rule's destination folder.
    AlreadyInPlace,
    /// The move failed.
    Error,
}

/// Machine-readable reason of a failure, localized by the front-ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The source file does not exist (anymore).
    SourceMissing,
    /// The volume of the destination is not mounted.
    VolumeMissing,
    /// The moved file is no longer at its destination (undo).
    DestinationMissing,
    /// The overwritten file could not be moved to the trash.
    Trash,
    /// Any other I/O error.
    Io,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::SourceMissing => "source_missing",
            ErrorCode::VolumeMissing => "volume_missing",
            ErrorCode::DestinationMissing => "destination_missing",
            ErrorCode::Trash => "trash",
            ErrorCode::Io => "io",
        }
    }

    pub fn parse(s: &str) -> ErrorCode {
        match s {
            "source_missing" => ErrorCode::SourceMissing,
            "volume_missing" => ErrorCode::VolumeMissing,
            "destination_missing" => ErrorCode::DestinationMissing,
            "trash" => ErrorCode::Trash,
            _ => ErrorCode::Io,
        }
    }
}

/// A failure on a single file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
pub struct OpError {
    pub code: ErrorCode,
    /// Technical detail (OS error message, path, …).
    pub detail: String,
}

impl OpError {
    pub fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub(crate) fn io(err: &std::io::Error) -> Self {
        let code = if err.kind() == std::io::ErrorKind::NotFound {
            ErrorCode::SourceMissing
        } else {
            ErrorCode::Io
        };
        Self::new(code, err.to_string())
    }
}

impl fmt::Display for OpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for OpError {}

/// How to resolve a name conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "kebab-case")]
pub enum Resolution {
    /// Replace the existing file (which goes to the trash).
    Overwrite,
    /// Keep both: the incoming file gets a free name such as `name (1).ext`.
    KeepBoth,
    /// Leave the incoming file where it is.
    Skip,
}

/// A name conflict, presented to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    /// The file that would be moved.
    pub incoming: PathBuf,
    /// The file already occupying the target path.
    pub existing: PathBuf,
    /// Whether this conflict comes from an undo (restoring a file).
    pub is_undo: bool,
}

/// Basic metadata of a file, shown in the conflict dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct FileMeta {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
}

impl FileMeta {
    pub fn read(path: &std::path::Path) -> Self {
        let meta = std::fs::metadata(path).ok();
        Self {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            modified: meta
                .and_then(|m| m.modified().ok())
                .map(chrono::DateTime::<chrono::Utc>::from),
        }
    }
}

/// A conflict with the information needed to present it to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct ConflictDetails {
    pub conflict: Conflict,
    pub existing: FileMeta,
    pub incoming: FileMeta,
    /// Both files have the same content.
    pub identical: bool,
}

impl ConflictDetails {
    pub fn gather(conflict: &Conflict) -> Self {
        Self {
            existing: FileMeta::read(&conflict.existing),
            incoming: FileMeta::read(&conflict.incoming),
            identical: crate::fsops::files_identical(&conflict.existing, &conflict.incoming)
                .unwrap_or(false),
            conflict: conflict.clone(),
        }
    }
}

/// A conflict sent to a graphical front-end, which answers by `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct ConflictRequest {
    pub id: u64,
    /// How many conflicts were already shown in the same operation.
    pub index: u32,
    #[serde(flatten)]
    pub details: ConflictDetails,
}

/// The user's answer to a conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    pub resolution: Resolution,
    /// Apply the same resolution to all remaining conflicts of the operation.
    pub apply_to_all: bool,
}

/// Asks how to resolve name conflicts (dialog, TTY prompt or fixed policy).
pub trait ConflictResolver {
    fn resolve(&mut self, conflict: &Conflict) -> Decision;
}

/// Resolves every conflict the same way (CLI `--on-conflict`, non-interactive use).
#[derive(Debug, Clone, Copy)]
pub struct FixedResolver(pub Resolution);

impl ConflictResolver for FixedResolver {
    fn resolve(&mut self, _conflict: &Conflict) -> Decision {
        Decision {
            resolution: self.0,
            apply_to_all: true,
        }
    }
}

/// Wraps a resolver and remembers an "apply to all" decision.
pub(crate) struct StickyResolver<'a> {
    inner: &'a mut dyn ConflictResolver,
    sticky: Option<Resolution>,
}

impl<'a> StickyResolver<'a> {
    pub(crate) fn new(inner: &'a mut dyn ConflictResolver) -> Self {
        Self {
            inner,
            sticky: None,
        }
    }

    pub(crate) fn resolve(&mut self, conflict: &Conflict) -> Resolution {
        if let Some(r) = self.sticky {
            return r;
        }
        let d = self.inner.resolve(conflict);
        if d.apply_to_all {
            self.sticky = Some(d.resolution);
        }
        d.resolution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_details() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a.txt");
        let b = tmp.path().join("b.txt");
        std::fs::write(&a, "same").unwrap();
        std::fs::write(&b, "same").unwrap();
        let d = ConflictDetails::gather(&Conflict {
            incoming: a.clone(),
            existing: b,
            is_undo: false,
        });
        assert!(d.identical);
        assert_eq!(d.incoming.name, "a.txt");
        assert_eq!(d.incoming.size, 4);
        assert!(d.incoming.modified.is_some());
    }

    #[test]
    fn error_code_roundtrip() {
        for c in [
            ErrorCode::SourceMissing,
            ErrorCode::VolumeMissing,
            ErrorCode::DestinationMissing,
            ErrorCode::Trash,
            ErrorCode::Io,
        ] {
            assert_eq!(ErrorCode::parse(c.as_str()), c);
            assert_eq!(
                serde_json::to_string(&c).unwrap(),
                format!("\"{}\"", c.as_str())
            );
        }
    }

    #[test]
    fn resolution_serializes_kebab_case() {
        assert_eq!(
            serde_json::to_string(&Resolution::KeepBoth).unwrap(),
            "\"keep-both\""
        );
    }

    struct Counting(u32, Decision);
    impl ConflictResolver for Counting {
        fn resolve(&mut self, _: &Conflict) -> Decision {
            self.0 += 1;
            self.1
        }
    }

    #[test]
    fn sticky_resolver_remembers_apply_to_all() {
        let c = Conflict {
            incoming: "a".into(),
            existing: "b".into(),
            is_undo: false,
        };
        let mut inner = Counting(
            0,
            Decision {
                resolution: Resolution::Skip,
                apply_to_all: false,
            },
        );
        {
            let mut s = StickyResolver::new(&mut inner);
            s.resolve(&c);
            s.resolve(&c);
        }
        assert_eq!(inner.0, 2);

        inner.1.apply_to_all = true;
        inner.0 = 0;
        {
            let mut s = StickyResolver::new(&mut inner);
            assert_eq!(s.resolve(&c), Resolution::Skip);
            assert_eq!(s.resolve(&c), Resolution::Skip);
        }
        assert_eq!(inner.0, 1);
    }
}
