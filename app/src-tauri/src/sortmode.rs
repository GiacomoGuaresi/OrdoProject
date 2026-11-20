//! Sorting started from the file manager (`ordo-app --sort <paths…>`).
//!
//! The file manager may start one process per selected file. Thanks to the
//! single-instance plugin, later processes forward their paths to the first
//! one, which collects them for a short debounce window and then sorts all of
//! them as a single batch.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use ordo_core::{History, Origin, RuleSet, Settings};
use tauri::{AppHandle, Manager};

use crate::commands::AppState;
use crate::conflicts::DialogResolver;
use crate::windows;

/// How long to wait for more paths before starting the batch.
pub const DEBOUNCE: Duration = Duration::from_millis(350);

/// Command-line flag that switches the app to sort mode.
pub const SORT_FLAG: &str = "--sort";

/// Extracts the paths following `--sort`, resolved against `cwd`.
/// Returns `None` when the flag is absent.
pub fn parse_sort_args<S: AsRef<str>>(args: &[S], cwd: &Path) -> Option<Vec<PathBuf>> {
    let pos = args.iter().position(|a| a.as_ref() == SORT_FLAG)?;
    Some(
        args[pos + 1..]
            .iter()
            .map(|a| a.as_ref())
            .filter(|a| !a.is_empty() && !a.starts_with("--"))
            .map(|a| {
                let p = PathBuf::from(a);
                if p.is_absolute() { p } else { cwd.join(p) }
            })
            .collect(),
    )
}

/// Collects paths and runs debounced batches, one at a time.
#[derive(Default)]
pub struct SortQueue {
    pending: Mutex<Vec<PathBuf>>,
    /// Origin recorded for the next batch.
    origin: Mutex<Option<Origin>>,
    generation: AtomicU64,
    /// A batch is collecting or running.
    busy: AtomicBool,
    /// Serializes batches.
    running: Mutex<()>,
}

impl SortQueue {
    /// Whether a batch is waiting or running.
    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }

    /// Adds paths and (re)starts the debounce timer.
    pub fn add(&self, app: &AppHandle, paths: Vec<PathBuf>, origin: Origin) {
        if paths.is_empty() {
            return;
        }
        self.pending.lock().expect("poisoned").extend(paths);
        *self.origin.lock().expect("poisoned") = Some(origin);
        self.busy.store(true, Ordering::SeqCst);
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(DEBOUNCE);
            let state = app.state::<AppState>();
            let queue = &state.sort_queue;
            if queue.generation.load(Ordering::SeqCst) != generation {
                return; // more paths arrived: a later thread will run the batch
            }
            let _guard = queue.running.lock().expect("poisoned");
            let paths = std::mem::take(&mut *queue.pending.lock().expect("poisoned"));
            if !paths.is_empty() {
                let origin = queue
                    .origin
                    .lock()
                    .expect("poisoned")
                    .take()
                    .unwrap_or(Origin::ContextMenu);
                run_batch(&app, &state, &paths, origin);
            }
            if queue.generation.load(Ordering::SeqCst) == generation {
                queue.busy.store(false, Ordering::SeqCst);
            }
            crate::exit_if_idle(&app);
        });
    }
}

fn run_batch(app: &AppHandle, state: &AppState, paths: &[PathBuf], origin: Origin) {
    let result = (|| -> ordo_core::Result<Option<i64>> {
        let settings = Settings::load(&state.paths.settings_file())?;
        let rules = RuleSet::load_or_init(&state.paths.rules_file(), settings.effective_lang())?
            .compile()?;
        let history = History::open(&state.paths.history_db())?;
        history.apply_retention(settings.retention_days)?;
        let mut resolver = DialogResolver::new(app.clone(), &state.broker, windows::CONFLICTS)
            .with_prepare(|app| {
                if let Err(e) = windows::show_conflicts(app) {
                    log::error!("cannot open the conflict window: {e}");
                }
            });
        let report = ordo_core::sort(paths, &rules, origin, &mut resolver, &history)?;
        Ok(report.batch_id)
    })();
    windows::close(app, windows::CONFLICTS);
    let batch = match result {
        Ok(batch) => batch,
        Err(e) => {
            log::error!("sort failed: {e}");
            None
        }
    };
    if let Err(e) = windows::show_summary(app, batch) {
        log::error!("cannot open the summary window: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sort_args() {
        let cwd = if cfg!(windows) {
            PathBuf::from(r"C:\cwd")
        } else {
            PathBuf::from("/cwd")
        };
        let abs = if cfg!(windows) {
            r"C:\x\a.pdf"
        } else {
            "/x/a.pdf"
        };
        let args = ["ordo-app", "--sort", abs, "b.txt", ""];
        assert_eq!(
            parse_sort_args(&args, &cwd).unwrap(),
            vec![PathBuf::from(abs), cwd.join("b.txt")]
        );
        assert!(parse_sort_args(&["ordo-app"], &cwd).is_none());
        assert_eq!(
            parse_sort_args(&["ordo-app", "--sort"], &cwd).unwrap(),
            Vec::<PathBuf>::new()
        );
    }
}
