//! Sorting: applies the rules to the selected files and records a batch.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::fsops;
use crate::history::{History, NewEntry, StatusCounts};
use crate::model::{
    Conflict, ConflictResolver, ErrorCode, MoveStatus, OpError, Origin, Resolution, StickyResolver,
};
use crate::rules::CompiledRules;
use crate::scan::{self, ScanEntry};

/// Summary of a sort run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortReport {
    /// The recorded batch; `None` when there was nothing to sort.
    pub batch_id: Option<i64>,
    pub counts: StatusCounts,
    /// Files that could not be sorted.
    pub errors: Vec<FileError>,
}

impl SortReport {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// A failure on a specific file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileError {
    pub path: PathBuf,
    pub error: OpError,
}

fn same_dir(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Sorts the files designated by `paths` (files, or folders whose first-level
/// files are taken) and records the run as a batch in `history`.
pub fn sort(
    paths: &[PathBuf],
    rules: &CompiledRules,
    origin: Origin,
    resolver: &mut dyn ConflictResolver,
    history: &History,
) -> Result<SortReport> {
    let mut resolver = StickyResolver::new(resolver);
    let mut report = SortReport::default();
    for item in scan::collect(paths) {
        let entry = match item {
            ScanEntry::Error { path, message } => {
                let code = if path.exists() {
                    ErrorCode::Io
                } else {
                    ErrorCode::SourceMissing
                };
                NewEntry {
                    source: path,
                    destination: None,
                    rule_name: None,
                    status: MoveStatus::Error,
                    overwrote: false,
                    error: Some(OpError::new(code, message)),
                }
            }
            ScanEntry::File(path) => sort_file(path, rules, &mut resolver),
        };
        let batch_id = match report.batch_id {
            Some(id) => id,
            None => *report.batch_id.insert(history.create_batch(origin)?),
        };
        history.add_entry(batch_id, &entry)?;
        report.counts.add(entry.status, 1);
        if let Some(error) = entry.error {
            report.errors.push(FileError {
                path: entry.source,
                error,
            });
        }
    }
    Ok(report)
}

fn sort_file(
    source: PathBuf,
    rules: &CompiledRules,
    resolver: &mut StickyResolver<'_>,
) -> NewEntry {
    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    let mut entry = NewEntry {
        source,
        destination: None,
        rule_name: None,
        status: MoveStatus::Ignored,
        overwrote: false,
        error: None,
    };
    let Some(rule) = rules.first_match(&name) else {
        return entry;
    };
    entry.rule_name = Some(rule.name.clone());
    let dest_dir = &rule.destination;
    let mut target = dest_dir.join(&name);
    entry.destination = Some(target.clone());

    if entry.source.parent().is_some_and(|p| same_dir(p, dest_dir)) {
        entry.status = MoveStatus::AlreadyInPlace;
        return entry;
    }
    if let Err(e) = fsops::ensure_dir(dest_dir) {
        return failed(entry, e);
    }

    if let Ok(existing) = std::fs::symlink_metadata(&target) {
        if !existing.is_file() {
            // Never replace a folder or a link: keep both silently.
            target = fsops::unique_path(&target);
        } else {
            let conflict = Conflict {
                incoming: entry.source.clone(),
                existing: target.clone(),
                is_undo: false,
            };
            match resolver.resolve(&conflict) {
                Resolution::Skip => {
                    entry.status = MoveStatus::Skipped;
                    return entry;
                }
                Resolution::KeepBoth => target = fsops::unique_path(&target),
                Resolution::Overwrite => {
                    if let Err(e) = fsops::trash_file(&target) {
                        return failed(entry, e);
                    }
                    entry.overwrote = true;
                }
            }
        }
    }

    entry.destination = Some(target.clone());
    match fsops::move_file(&entry.source, &target) {
        Ok(()) => entry.status = MoveStatus::Done,
        Err(e) => return failed(entry, e),
    }
    entry
}

fn failed(mut entry: NewEntry, error: OpError) -> NewEntry {
    entry.status = MoveStatus::Error;
    entry.error = Some(error);
    entry
}
