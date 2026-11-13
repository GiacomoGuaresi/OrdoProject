//! Ordo core library: rules, matching, file moves, conflicts, history and undo.

pub mod error;
pub mod fsops;
pub mod history;
pub mod lang;
pub mod model;
pub mod paths;
pub mod rules;
pub mod scan;
pub mod settings;
pub mod sorter;
pub mod undo;

pub use error::{Error, Result};
pub use history::{BatchDetail, BatchSummary, Entry, History, StatusCounts};
pub use lang::Lang;
pub use model::{
    Conflict, ConflictDetails, ConflictRequest, ConflictResolver, Decision, ErrorCode, FileMeta,
    FixedResolver, MoveStatus, OpError, Origin, Resolution,
};
pub use paths::AppPaths;
pub use rules::{PatternKind, Rule, RuleSet};
pub use settings::{Settings, Theme};
pub use sorter::{FileError, SortReport, sort};
pub use undo::{TrashOutcome, UndoOutcome, UndoReport, undo_batch, undo_latest, undo_move};

/// Version of the Ordo core library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
