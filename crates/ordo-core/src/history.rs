//! Sort history stored in SQLite: batches and their entries.
//!
//! The database runs in WAL mode so that the CLI and several app processes
//! can use it at the same time.

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::{ErrorCode, MoveStatus, OpError, Origin};

const SCHEMA_VERSION: i32 = 1;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS batches (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at  INTEGER NOT NULL,
    origin      TEXT    NOT NULL
);
CREATE TABLE IF NOT EXISTS moves (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_id     INTEGER NOT NULL REFERENCES batches(id) ON DELETE CASCADE,
    source       TEXT    NOT NULL,
    destination  TEXT,
    rule_name    TEXT,
    status       TEXT    NOT NULL,
    overwrote    INTEGER NOT NULL DEFAULT 0,
    error_code   TEXT,
    error_detail TEXT,
    restored_to  TEXT,
    updated_at   INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS moves_batch ON moves(batch_id);
CREATE INDEX IF NOT EXISTS batches_created ON batches(created_at);
";

fn status_str(s: MoveStatus) -> &'static str {
    match s {
        MoveStatus::Done => "done",
        MoveStatus::Undone => "undone",
        MoveStatus::NotUndoable => "not_undoable",
        MoveStatus::Skipped => "skipped",
        MoveStatus::Ignored => "ignored",
        MoveStatus::AlreadyInPlace => "already_in_place",
        MoveStatus::Error => "error",
    }
}

fn parse_status(s: &str) -> MoveStatus {
    match s {
        "done" => MoveStatus::Done,
        "undone" => MoveStatus::Undone,
        "not_undoable" => MoveStatus::NotUndoable,
        "skipped" => MoveStatus::Skipped,
        "ignored" => MoveStatus::Ignored,
        "already_in_place" => MoveStatus::AlreadyInPlace,
        _ => MoveStatus::Error,
    }
}

fn origin_str(o: Origin) -> &'static str {
    match o {
        Origin::Cli => "cli",
        Origin::ContextMenu => "context_menu",
        Origin::App => "app",
    }
}

fn parse_origin(s: &str) -> Origin {
    match s {
        "context_menu" => Origin::ContextMenu,
        "app" => Origin::App,
        _ => Origin::Cli,
    }
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

fn from_ms(ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms).single().unwrap_or_default()
}

fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// A new entry to record in a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    pub source: PathBuf,
    pub destination: Option<PathBuf>,
    pub rule_name: Option<String>,
    pub status: MoveStatus,
    /// An existing file at the destination was moved to the trash.
    pub overwrote: bool,
    pub error: Option<OpError>,
}

/// A recorded entry of a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: i64,
    pub batch_id: i64,
    pub source: PathBuf,
    pub destination: Option<PathBuf>,
    pub rule_name: Option<String>,
    pub status: MoveStatus,
    pub overwrote: bool,
    pub error: Option<OpError>,
    /// Where the file was put back by an undo, when different from `source`.
    pub restored_to: Option<PathBuf>,
    pub updated_at: DateTime<Utc>,
}

impl Entry {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let error_code: Option<String> = row.get("error_code")?;
        let error_detail: Option<String> = row.get("error_detail")?;
        Ok(Entry {
            id: row.get("id")?,
            batch_id: row.get("batch_id")?,
            source: PathBuf::from(row.get::<_, String>("source")?),
            destination: row
                .get::<_, Option<String>>("destination")?
                .map(PathBuf::from),
            rule_name: row.get("rule_name")?,
            status: parse_status(&row.get::<_, String>("status")?),
            overwrote: row.get("overwrote")?,
            error: error_code
                .map(|c| OpError::new(ErrorCode::parse(&c), error_detail.unwrap_or_default())),
            restored_to: row
                .get::<_, Option<String>>("restored_to")?
                .map(PathBuf::from),
            updated_at: from_ms(row.get("updated_at")?),
        })
    }
}

/// Number of entries per status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct StatusCounts {
    pub done: u32,
    pub undone: u32,
    pub not_undoable: u32,
    pub skipped: u32,
    pub ignored: u32,
    pub already_in_place: u32,
    pub error: u32,
}

impl StatusCounts {
    pub fn add(&mut self, status: MoveStatus, n: u32) {
        let slot = match status {
            MoveStatus::Done => &mut self.done,
            MoveStatus::Undone => &mut self.undone,
            MoveStatus::NotUndoable => &mut self.not_undoable,
            MoveStatus::Skipped => &mut self.skipped,
            MoveStatus::Ignored => &mut self.ignored,
            MoveStatus::AlreadyInPlace => &mut self.already_in_place,
            MoveStatus::Error => &mut self.error,
        };
        *slot += n;
    }

    pub fn total(&self) -> u32 {
        self.done
            + self.undone
            + self.not_undoable
            + self.skipped
            + self.ignored
            + self.already_in_place
            + self.error
    }

    /// Files moved by the batch, including those later undone.
    pub fn moved(&self) -> u32 {
        self.done + self.undone + self.not_undoable
    }
}

/// A batch with its entry counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub origin: Origin,
    pub counts: StatusCounts,
}

/// A batch with all its entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct BatchDetail {
    #[serde(flatten)]
    pub summary: BatchSummary,
    pub entries: Vec<Entry>,
}

/// Handle to the history database.
pub struct History {
    conn: Connection,
}

impl History {
    /// Opens (and creates/migrates if needed) the database at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(10))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Self::init(conn)
    }

    /// Opens a private in-memory database (tests).
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", true)?;
        let version: i32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version < SCHEMA_VERSION {
            conn.execute_batch(SCHEMA)?;
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        Ok(Self { conn })
    }

    /// Creates an empty batch and returns its id.
    pub fn create_batch(&self, origin: Origin) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO batches (created_at, origin) VALUES (?1, ?2)",
            params![now_ms(), origin_str(origin)],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Records an entry in a batch and returns its id.
    pub fn add_entry(&self, batch_id: i64, e: &NewEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO moves (batch_id, source, destination, rule_name, status, overwrote,
                                error_code, error_detail, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                batch_id,
                path_str(&e.source),
                e.destination.as_deref().map(path_str),
                e.rule_name,
                status_str(e.status),
                e.overwrote,
                e.error.as_ref().map(|x| x.code.as_str()),
                e.error.as_ref().map(|x| x.detail.as_str()),
                now_ms(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Updates the status of an entry (undo). `error` replaces the stored error.
    pub fn set_status(
        &self,
        move_id: i64,
        status: MoveStatus,
        error: Option<&OpError>,
        restored_to: Option<&Path>,
    ) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE moves SET status = ?2, error_code = ?3, error_detail = ?4,
                              restored_to = COALESCE(?5, restored_to), updated_at = ?6
             WHERE id = ?1",
            params![
                move_id,
                status_str(status),
                error.map(|x| x.code.as_str()),
                error.map(|x| x.detail.as_str()),
                restored_to.map(path_str),
                now_ms(),
            ],
        )?;
        if n == 0 {
            return Err(Error::MoveNotFound(move_id));
        }
        Ok(())
    }

    fn counts(&self, batch_id: i64) -> Result<StatusCounts> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT status, COUNT(*) FROM moves WHERE batch_id = ?1 GROUP BY status",
        )?;
        let mut counts = StatusCounts::default();
        let rows = stmt.query_map([batch_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?))
        })?;
        for row in rows {
            let (status, n) = row?;
            counts.add(parse_status(&status), n);
        }
        Ok(counts)
    }

    fn summary_from_row(&self, row: (i64, i64, String)) -> Result<BatchSummary> {
        let (id, created, origin) = row;
        Ok(BatchSummary {
            id,
            created_at: from_ms(created),
            origin: parse_origin(&origin),
            counts: self.counts(id)?,
        })
    }

    /// Most recent batches first.
    pub fn list_batches(&self, limit: usize) -> Result<Vec<BatchSummary>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, created_at, origin FROM batches ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt
            .query_map([limit as i64], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter().map(|r| self.summary_from_row(r)).collect()
    }

    /// A batch with its entries, in the order they were recorded.
    pub fn batch(&self, batch_id: i64) -> Result<BatchDetail> {
        let row = self
            .conn
            .query_row(
                "SELECT id, created_at, origin FROM batches WHERE id = ?1",
                [batch_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?
            .ok_or(Error::BatchNotFound(batch_id))?;
        let summary = self.summary_from_row(row)?;
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM moves WHERE batch_id = ?1 ORDER BY id")?;
        let entries = stmt
            .query_map([batch_id], Entry::from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(BatchDetail { summary, entries })
    }

    /// A single entry.
    pub fn entry(&self, move_id: i64) -> Result<Entry> {
        self.conn
            .query_row(
                "SELECT * FROM moves WHERE id = ?1",
                [move_id],
                Entry::from_row,
            )
            .optional()?
            .ok_or(Error::MoveNotFound(move_id))
    }

    /// The most recent batch that still has undoable moves.
    pub fn latest_undoable_batch(&self) -> Result<Option<i64>> {
        Ok(self
            .conn
            .query_row(
                "SELECT batch_id FROM moves WHERE status = 'done' ORDER BY batch_id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?)
    }

    /// Deletes batches older than `days` days. Returns the number of deleted batches.
    pub fn purge_older_than(&self, days: u32) -> Result<usize> {
        let cutoff = now_ms() - i64::from(days) * 24 * 60 * 60 * 1000;
        self.purge_before(cutoff)
    }

    fn purge_before(&self, cutoff_ms: i64) -> Result<usize> {
        Ok(self
            .conn
            .execute("DELETE FROM batches WHERE created_at < ?1", [cutoff_ms])?)
    }

    /// Applies the retention setting (`None` keeps everything).
    pub fn apply_retention(&self, retention_days: Option<u32>) -> Result<usize> {
        match retention_days {
            Some(days) => self.purge_older_than(days),
            None => Ok(0),
        }
    }

    /// Deletes the whole history.
    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM batches", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(status: MoveStatus) -> NewEntry {
        NewEntry {
            source: PathBuf::from("/src/a.txt"),
            destination: Some(PathBuf::from("/dst/a.txt")),
            rule_name: Some("Docs".into()),
            status,
            overwrote: false,
            error: None,
        }
    }

    #[test]
    fn records_and_reads_batches() {
        let h = History::open_in_memory().unwrap();
        let b = h.create_batch(Origin::ContextMenu).unwrap();
        let m1 = h.add_entry(b, &entry(MoveStatus::Done)).unwrap();
        h.add_entry(b, &entry(MoveStatus::Ignored)).unwrap();
        let mut failed = entry(MoveStatus::Error);
        failed.error = Some(OpError::new(ErrorCode::VolumeMissing, "/Volumes/X"));
        h.add_entry(b, &failed).unwrap();

        let detail = h.batch(b).unwrap();
        assert_eq!(detail.summary.origin, Origin::ContextMenu);
        assert_eq!(detail.entries.len(), 3);
        assert_eq!(detail.entries[0].id, m1);
        assert_eq!(detail.summary.counts.done, 1);
        assert_eq!(detail.summary.counts.ignored, 1);
        assert_eq!(detail.summary.counts.error, 1);
        assert_eq!(detail.summary.counts.total(), 3);
        assert_eq!(
            detail.entries[2].error.as_ref().unwrap().code,
            ErrorCode::VolumeMissing
        );
    }

    #[test]
    fn status_updates_and_latest_undoable() {
        let h = History::open_in_memory().unwrap();
        assert_eq!(h.latest_undoable_batch().unwrap(), None);
        let b1 = h.create_batch(Origin::Cli).unwrap();
        let m1 = h.add_entry(b1, &entry(MoveStatus::Done)).unwrap();
        let b2 = h.create_batch(Origin::Cli).unwrap();
        let m2 = h.add_entry(b2, &entry(MoveStatus::Done)).unwrap();
        assert_eq!(h.latest_undoable_batch().unwrap(), Some(b2));

        h.set_status(
            m2,
            MoveStatus::Undone,
            None,
            Some(Path::new("/src/a (1).txt")),
        )
        .unwrap();
        assert_eq!(h.latest_undoable_batch().unwrap(), Some(b1));
        let e = h.entry(m2).unwrap();
        assert_eq!(e.status, MoveStatus::Undone);
        assert_eq!(e.restored_to, Some(PathBuf::from("/src/a (1).txt")));

        h.set_status(m1, MoveStatus::NotUndoable, None, None)
            .unwrap();
        assert_eq!(h.latest_undoable_batch().unwrap(), None);
        assert!(matches!(
            h.set_status(999, MoveStatus::Done, None, None),
            Err(Error::MoveNotFound(999))
        ));
    }

    #[test]
    fn list_is_most_recent_first_and_limited() {
        let h = History::open_in_memory().unwrap();
        let ids: Vec<_> = (0..5)
            .map(|_| h.create_batch(Origin::App).unwrap())
            .collect();
        let list = h.list_batches(3).unwrap();
        assert_eq!(
            list.iter().map(|b| b.id).collect::<Vec<_>>(),
            vec![ids[4], ids[3], ids[2]]
        );
        assert!(matches!(h.batch(12345), Err(Error::BatchNotFound(_))));
    }

    #[test]
    fn retention_and_clear() {
        let h = History::open_in_memory().unwrap();
        let old = h.create_batch(Origin::Cli).unwrap();
        h.add_entry(old, &entry(MoveStatus::Done)).unwrap();
        h.conn
            .execute(
                "UPDATE batches SET created_at = ?1 WHERE id = ?2",
                params![now_ms() - 100 * 24 * 3600 * 1000, old],
            )
            .unwrap();
        let recent = h.create_batch(Origin::Cli).unwrap();

        assert_eq!(h.apply_retention(None).unwrap(), 0);
        assert_eq!(h.apply_retention(Some(90)).unwrap(), 1);
        assert!(h.batch(old).is_err());
        // Entries are removed together with their batch.
        let orphans: i64 = h
            .conn
            .query_row("SELECT COUNT(*) FROM moves", [], |r| r.get(0))
            .unwrap();
        assert_eq!(orphans, 0);
        assert!(h.batch(recent).is_ok());

        h.clear().unwrap();
        assert!(h.list_batches(10).unwrap().is_empty());
    }

    #[test]
    fn file_database_is_shared_between_handles() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("data/history.sqlite");
        let a = History::open(&path).unwrap();
        let b = History::open(&path).unwrap();
        let id = a.create_batch(Origin::Cli).unwrap();
        assert_eq!(b.list_batches(1).unwrap()[0].id, id);
    }
}
