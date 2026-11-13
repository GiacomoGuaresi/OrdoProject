//! Ordo core library: rules, matching, file moves, conflicts, history and undo.

pub mod error;
pub mod fsops;
pub mod lang;
pub mod model;
pub mod paths;
pub mod rules;
pub mod scan;
pub mod settings;

pub use error::{Error, Result};
pub use lang::Lang;
pub use model::{
    Conflict, ConflictResolver, Decision, ErrorCode, FixedResolver, MoveStatus, OpError, Origin,
    Resolution,
};
pub use paths::AppPaths;
pub use rules::{PatternKind, Rule, RuleSet};
pub use settings::{Settings, Theme};

/// Version of the Ordo core library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
