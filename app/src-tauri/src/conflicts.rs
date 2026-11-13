//! Conflict resolution through the web UI.
//!
//! A [`DialogResolver`] runs on a worker thread: for each conflict it emits a
//! `conflict` event with a [`ConflictRequest`] and blocks until the UI calls
//! the `resolve_conflict` command with the same id.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};

use ordo_core::{
    Conflict, ConflictDetails, ConflictRequest, ConflictResolver, Decision, Resolution,
};
use tauri::{AppHandle, Emitter};

/// Event emitted to the UI for each conflict.
pub const CONFLICT_EVENT: &str = "conflict";

/// Pending conflicts waiting for an answer from the UI.
#[derive(Default)]
pub struct ConflictBroker {
    next_id: AtomicU64,
    pending: Mutex<HashMap<u64, Sender<Decision>>>,
}

impl ConflictBroker {
    /// Delivers the UI's answer. Returns `false` if nobody is waiting for `id`.
    pub fn answer(&self, id: u64, decision: Decision) -> bool {
        let sender = self.pending.lock().expect("poisoned").remove(&id);
        sender.is_some_and(|s| s.send(decision).is_ok())
    }

    /// Answers every pending conflict with "skip" (e.g. the window was closed).
    pub fn cancel_all(&self) {
        let pending: Vec<_> = self.pending.lock().expect("poisoned").drain().collect();
        for (_, sender) in pending {
            let _ = sender.send(Decision {
                resolution: Resolution::Skip,
                apply_to_all: true,
            });
        }
    }
}

/// Asks the UI how to resolve each conflict.
pub struct DialogResolver<'a> {
    pub app: AppHandle,
    pub broker: &'a ConflictBroker,
    pub shown: u32,
}

impl<'a> DialogResolver<'a> {
    pub fn new(app: AppHandle, broker: &'a ConflictBroker) -> Self {
        Self {
            app,
            broker,
            shown: 0,
        }
    }
}

impl ConflictResolver for DialogResolver<'_> {
    fn resolve(&mut self, conflict: &Conflict) -> Decision {
        let skip = Decision {
            resolution: Resolution::Skip,
            apply_to_all: false,
        };
        let id = self.broker.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = channel();
        self.broker.pending.lock().expect("poisoned").insert(id, tx);
        let request = ConflictRequest {
            id,
            index: self.shown,
            details: ConflictDetails::gather(conflict),
        };
        self.shown += 1;
        if let Err(e) = self.app.emit(CONFLICT_EVENT, &request) {
            log::error!("cannot emit conflict event: {e}");
            self.broker.pending.lock().expect("poisoned").remove(&id);
            return skip;
        }
        // If the UI goes away, the sender is dropped and we fall back to skip:
        // the file stays where it is, which is always safe.
        rx.recv().unwrap_or(skip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answer_unknown_id_is_false() {
        let b = ConflictBroker::default();
        assert!(!b.answer(
            1,
            Decision {
                resolution: Resolution::Skip,
                apply_to_all: false
            }
        ));
    }

    #[test]
    fn cancel_all_answers_skip() {
        let b = ConflictBroker::default();
        let (tx, rx) = channel();
        b.pending.lock().unwrap().insert(7, tx);
        b.cancel_all();
        assert_eq!(rx.recv().unwrap().resolution, Resolution::Skip);
    }
}
