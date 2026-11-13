//! Conflict resolution through the web UI.
//!
//! A [`DialogResolver`] runs on a worker thread. For each conflict it stores a
//! [`ConflictRequest`] addressed to a window, emits a `conflict` event to that
//! window and blocks until the UI calls `resolve_conflict` with the same id.
//! A window that opens late fetches outstanding requests with
//! `pending_conflicts`, so no request is lost.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};

use ordo_core::{
    Conflict, ConflictDetails, ConflictRequest, ConflictResolver, Decision, Resolution,
};
use tauri::{AppHandle, Emitter};

/// Event emitted to the target window for each conflict.
pub const CONFLICT_EVENT: &str = "conflict";

struct Pending {
    window: String,
    request: ConflictRequest,
    sender: Sender<Decision>,
}

/// Pending conflicts waiting for an answer from the UI.
#[derive(Default)]
pub struct ConflictBroker {
    next_id: AtomicU64,
    pending: Mutex<BTreeMap<u64, Pending>>,
}

const SKIP_ALL: Decision = Decision {
    resolution: Resolution::Skip,
    apply_to_all: true,
};

impl ConflictBroker {
    fn register(&self, window: &str, request: ConflictRequest, sender: Sender<Decision>) {
        self.pending.lock().expect("poisoned").insert(
            request.id,
            Pending {
                window: window.to_string(),
                request,
                sender,
            },
        );
    }

    /// Delivers the UI's answer. Returns `false` if nobody is waiting for `id`.
    pub fn answer(&self, id: u64, decision: Decision) -> bool {
        let pending = self.pending.lock().expect("poisoned").remove(&id);
        pending.is_some_and(|p| p.sender.send(decision).is_ok())
    }

    /// Requests addressed to `window` that are still unanswered, oldest first.
    pub fn pending_for(&self, window: &str) -> Vec<ConflictRequest> {
        self.pending
            .lock()
            .expect("poisoned")
            .values()
            .filter(|p| p.window == window)
            .map(|p| p.request.clone())
            .collect()
    }

    /// Answers "skip" to every request addressed to `window` (it was closed).
    pub fn cancel_for(&self, window: &str) {
        let mut pending = self.pending.lock().expect("poisoned");
        let ids: Vec<u64> = pending
            .iter()
            .filter(|(_, p)| p.window == window)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            if let Some(p) = pending.remove(&id) {
                let _ = p.sender.send(SKIP_ALL);
            }
        }
    }
}

/// Hook run before the first conflict is sent.
type Prepare<'a> = Box<dyn FnMut(&AppHandle) + Send + 'a>;

/// Asks a window of the UI how to resolve each conflict.
pub struct DialogResolver<'a> {
    app: AppHandle,
    broker: &'a ConflictBroker,
    window: String,
    /// Called before the first conflict is sent (e.g. to open the window).
    prepare: Option<Prepare<'a>>,
    shown: u32,
}

impl<'a> DialogResolver<'a> {
    pub fn new(app: AppHandle, broker: &'a ConflictBroker, window: impl Into<String>) -> Self {
        Self {
            app,
            broker,
            window: window.into(),
            prepare: None,
            shown: 0,
        }
    }

    pub fn with_prepare(mut self, f: impl FnMut(&AppHandle) + Send + 'a) -> Self {
        self.prepare = Some(Box::new(f));
        self
    }
}

impl ConflictResolver for DialogResolver<'_> {
    fn resolve(&mut self, conflict: &Conflict) -> Decision {
        if self.shown == 0
            && let Some(prepare) = self.prepare.as_mut()
        {
            prepare(&self.app);
        }
        let id = self.broker.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = channel();
        let request = ConflictRequest {
            id,
            index: self.shown,
            details: ConflictDetails::gather(conflict),
        };
        self.shown += 1;
        self.broker.register(&self.window, request.clone(), tx);
        if let Err(e) = self.app.emit_to(&self.window, CONFLICT_EVENT, &request) {
            log::warn!("cannot emit conflict event: {e}");
        }
        // If the window goes away, the request is cancelled with "skip":
        // the file stays where it is, which is always safe.
        rx.recv().unwrap_or(SKIP_ALL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::FileMeta;

    fn request(id: u64) -> ConflictRequest {
        let meta = FileMeta {
            path: "/a".into(),
            name: "a".into(),
            size: 0,
            modified: None,
        };
        ConflictRequest {
            id,
            index: 0,
            details: ConflictDetails {
                conflict: Conflict {
                    incoming: "/a".into(),
                    existing: "/b".into(),
                    is_undo: false,
                },
                existing: meta.clone(),
                incoming: meta,
                identical: false,
            },
        }
    }

    #[test]
    fn answer_unknown_id_is_false() {
        let b = ConflictBroker::default();
        assert!(!b.answer(1, SKIP_ALL));
    }

    #[test]
    fn pending_per_window_and_answer() {
        let b = ConflictBroker::default();
        let (tx1, rx1) = channel();
        let (tx2, _rx2) = channel();
        b.register("main", request(1), tx1);
        b.register("conflicts", request(2), tx2);
        assert_eq!(b.pending_for("main").len(), 1);
        assert_eq!(b.pending_for("conflicts")[0].id, 2);
        let keep = Decision {
            resolution: Resolution::KeepBoth,
            apply_to_all: false,
        };
        assert!(b.answer(1, keep));
        assert_eq!(rx1.recv().unwrap(), keep);
        assert!(b.pending_for("main").is_empty());
    }

    #[test]
    fn cancel_for_answers_skip_only_for_that_window() {
        let b = ConflictBroker::default();
        let (tx1, rx1) = channel();
        let (tx2, _rx2) = channel();
        b.register("summary", request(1), tx1);
        b.register("main", request(2), tx2);
        b.cancel_for("summary");
        assert_eq!(rx1.recv().unwrap().resolution, Resolution::Skip);
        assert_eq!(b.pending_for("main").len(), 1);
    }
}
