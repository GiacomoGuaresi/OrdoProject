//! Undo of a whole batch or of a single move.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::fsops::{self, TrashRestore};
use crate::history::{Entry, History};
use crate::model::{
    Conflict, ConflictResolver, ErrorCode, MoveStatus, OpError, Resolution, StickyResolver,
};
use crate::sorter::FileError;

/// Result of undoing one move.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UndoOutcome {
    /// The file is back. `restored_to` is where it was put (it differs from the
    /// original path when the user chose "keep both").
    Undone {
        restored_to: PathBuf,
        /// When the move had overwritten a file: whether that file was put back
        /// from the trash, or must be restored manually.
        trash: Option<TrashOutcome>,
    },
    /// The moved file is no longer at its destination.
    NotUndoable,
    /// The user skipped it because the original path is occupied.
    Skipped,
    /// The entry is not in the `done` state (already undone, ignored, …).
    NotApplicable,
    Failed {
        error: OpError,
    },
}

/// What happened to a file that had been overwritten (and trashed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TrashOutcome {
    Restored,
    /// The overwritten file is still in the trash; its original location was `path`.
    Manual {
        path: PathBuf,
    },
}

/// Summary of a batch undo.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoReport {
    pub batch_id: i64,
    pub undone: u32,
    pub not_undoable: u32,
    pub skipped: u32,
    pub errors: Vec<FileError>,
    /// Overwritten files that the user must restore manually from the trash.
    pub manual_trash_restores: Vec<PathBuf>,
}

impl UndoReport {
    fn record(&mut self, entry: &Entry, outcome: UndoOutcome) {
        match outcome {
            UndoOutcome::Undone { trash, .. } => {
                self.undone += 1;
                if let Some(TrashOutcome::Manual { path }) = trash {
                    self.manual_trash_restores.push(path);
                }
            }
            UndoOutcome::NotUndoable => self.not_undoable += 1,
            UndoOutcome::Skipped => self.skipped += 1,
            UndoOutcome::NotApplicable => {}
            UndoOutcome::Failed { error } => self.errors.push(FileError {
                path: entry.source.clone(),
                error,
            }),
        }
    }
}

/// Undoes every move of a batch that is still `done`, most recent first.
pub fn undo_batch(
    history: &History,
    batch_id: i64,
    resolver: &mut dyn ConflictResolver,
) -> Result<UndoReport> {
    let detail = history.batch(batch_id)?;
    let mut resolver = StickyResolver::new(resolver);
    let mut report = UndoReport {
        batch_id,
        ..Default::default()
    };
    for entry in detail.entries.iter().rev() {
        let outcome = undo_entry(history, entry, &mut resolver)?;
        report.record(entry, outcome);
    }
    Ok(report)
}

/// Undoes a single move.
pub fn undo_move(
    history: &History,
    move_id: i64,
    resolver: &mut dyn ConflictResolver,
) -> Result<UndoOutcome> {
    let entry = history.entry(move_id)?;
    let mut resolver = StickyResolver::new(resolver);
    undo_entry(history, &entry, &mut resolver)
}

fn undo_entry(
    history: &History,
    entry: &Entry,
    resolver: &mut StickyResolver<'_>,
) -> Result<UndoOutcome> {
    if entry.status != MoveStatus::Done {
        return Ok(UndoOutcome::NotApplicable);
    }
    let Some(moved) = entry.destination.as_deref() else {
        return Ok(UndoOutcome::NotApplicable);
    };
    if !moved.is_file() {
        let err = OpError::new(ErrorCode::DestinationMissing, moved.display().to_string());
        history.set_status(entry.id, MoveStatus::NotUndoable, Some(&err), None)?;
        return Ok(UndoOutcome::NotUndoable);
    }

    let original = entry.source.as_path();
    if let Some(parent) = original.parent()
        && let Err(error) = fsops::ensure_dir(parent)
    {
        return Ok(UndoOutcome::Failed { error });
    }

    let mut target = original.to_path_buf();
    if let Ok(existing) = std::fs::symlink_metadata(&target) {
        if !existing.is_file() {
            target = fsops::unique_path(&target);
        } else {
            let conflict = Conflict {
                incoming: moved.to_path_buf(),
                existing: target.clone(),
                is_undo: true,
            };
            match resolver.resolve(&conflict) {
                Resolution::Skip => return Ok(UndoOutcome::Skipped),
                Resolution::KeepBoth => target = fsops::unique_path(&target),
                Resolution::Overwrite => {
                    if let Err(error) = fsops::trash_file(&target) {
                        return Ok(UndoOutcome::Failed { error });
                    }
                }
            }
        }
    }

    if let Err(error) = fsops::move_file(moved, &target) {
        return Ok(UndoOutcome::Failed { error });
    }

    let trash = entry.overwrote.then(|| restore_overwritten(moved));
    let restored_to = (target != original).then_some(target.as_path());
    history.set_status(entry.id, MoveStatus::Undone, None, restored_to)?;
    Ok(UndoOutcome::Undone {
        restored_to: target,
        trash,
    })
}

fn restore_overwritten(path: &Path) -> TrashOutcome {
    match fsops::restore_from_trash(path) {
        TrashRestore::Restored => TrashOutcome::Restored,
        TrashRestore::Manual => TrashOutcome::Manual {
            path: path.to_path_buf(),
        },
    }
}

/// Undoes the most recent batch that still has undoable moves.
pub fn undo_latest(
    history: &History,
    resolver: &mut dyn ConflictResolver,
) -> Result<Option<UndoReport>> {
    match history.latest_undoable_batch()? {
        Some(id) => undo_batch(history, id, resolver).map(Some),
        None => Ok(None),
    }
}
