//! Single instance with message passing, safe against simultaneous launches.
//!
//! A file manager may start one process per selected file, all at the same
//! moment. Every process first drops its request into an inbox folder
//! (written atomically), then tries to take an exclusive lock:
//!
//! - the process that gets the lock is the *primary*: it runs the app and
//!   keeps draining the inbox;
//! - the others exit immediately: their request is already in the inbox.
//!
//! When the primary is about to exit, it releases the lock and checks the
//! inbox once more. A request written before the release is seen by that
//! check; one written after it finds the lock free and its sender becomes the
//! new primary. Either way no request is lost.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A request sent to the primary instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Message {
    /// Sort these (absolute) paths.
    Sort { paths: Vec<PathBuf> },
    /// Show the main window.
    Show,
}

/// Folder layout of the instance mechanism.
#[derive(Debug, Clone)]
pub struct InstanceDir {
    root: PathBuf,
}

impl InstanceDir {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            root: data_dir.join("instance"),
        }
    }

    fn inbox(&self) -> PathBuf {
        self.root.join("inbox")
    }

    fn lock_file(&self) -> PathBuf {
        self.root.join("primary.lock")
    }

    /// Writes a message to the inbox atomically.
    pub fn send(&self, message: &Message) -> io::Result<()> {
        let inbox = self.inbox();
        fs::create_dir_all(&inbox)?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        // Time first so that sorting by name keeps the arrival order.
        let name = format!("{nanos:032}-{}-{seq:08}", std::process::id());
        let tmp = inbox.join(format!(".{name}.tmp"));
        fs::write(&tmp, serde_json::to_vec(message)?)?;
        fs::rename(&tmp, inbox.join(format!("{name}.json")))
    }

    /// Removes and returns the pending messages, oldest first.
    pub fn drain(&self) -> Vec<Message> {
        let Ok(entries) = fs::read_dir(self.inbox()) else {
            return Vec::new();
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        files.sort();
        files
            .into_iter()
            .filter_map(|path| {
                let data = fs::read(&path).ok();
                let _ = fs::remove_file(&path);
                data.and_then(|d| serde_json::from_slice(&d).ok())
            })
            .collect()
    }

    /// Whether messages are waiting.
    pub fn has_pending(&self) -> bool {
        fs::read_dir(self.inbox()).is_ok_and(|mut d| {
            d.any(|e| {
                e.ok()
                    .is_some_and(|e| e.path().extension().is_some_and(|x| x == "json"))
            })
        })
    }

    /// Tries to become the primary instance. The lock is held while the
    /// returned file is alive.
    pub fn try_acquire(&self) -> io::Result<Option<File>> {
        fs::create_dir_all(&self.root)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.lock_file())?;
        match file.try_lock() {
            Ok(()) => Ok(Some(file)),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(e)) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_roundtrip_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = InstanceDir::new(tmp.path());
        assert!(dir.drain().is_empty());
        assert!(!dir.has_pending());
        let a = Message::Sort {
            paths: vec!["/a".into()],
        };
        dir.send(&a).unwrap();
        dir.send(&Message::Show).unwrap();
        assert!(dir.has_pending());
        assert_eq!(dir.drain(), vec![a, Message::Show]);
        assert!(!dir.has_pending());
    }

    #[test]
    fn only_one_primary() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = InstanceDir::new(tmp.path());
        let first = dir.try_acquire().unwrap();
        assert!(first.is_some());
        assert!(dir.try_acquire().unwrap().is_none());
        drop(first);
        assert!(dir.try_acquire().unwrap().is_some());
    }

    #[test]
    fn concurrent_senders_are_all_received() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = InstanceDir::new(tmp.path());
        let handles: Vec<_> = (0..16)
            .map(|i| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    dir.send(&Message::Sort {
                        paths: vec![format!("/f{i}").into()],
                    })
                    .unwrap()
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(dir.drain().len(), 16);
    }
}
