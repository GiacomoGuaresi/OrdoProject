//! End-to-end tests of sorting and undo on temporary folders.

#![allow(clippy::cloned_ref_to_slice_refs)]

use std::fs;
use std::path::{Path, PathBuf};

use ordo_core::model::{Conflict, ConflictResolver, Decision};
use ordo_core::rules::CompiledRules;
use ordo_core::*;

/// Tests that put files in the real system trash only run in CI or when
/// `ORDO_TEST_TRASH` is set, to keep the developer's trash clean.
fn trash_tests_enabled() -> bool {
    std::env::var_os("CI").is_some() || std::env::var_os("ORDO_TEST_TRASH").is_some()
}

struct Env {
    _tmp: tempfile::TempDir,
    src: PathBuf,
    docs: PathBuf,
    pics: PathBuf,
    history: History,
}

impl Env {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        let src = root.join("Desktop");
        fs::create_dir(&src).unwrap();
        Env {
            src,
            docs: root.join("Docs"),
            pics: root.join("Pics"),
            history: History::open(&root.join("history.sqlite")).unwrap(),
            _tmp: tmp,
        }
    }

    fn rules(&self) -> CompiledRules {
        RuleSet::new(vec![
            Rule::new("Docs", PatternKind::Glob, "*.{pdf,txt}", &self.docs),
            Rule::new("Pics", PatternKind::Regex, r"\.(jpe?g|png)$", &self.pics),
        ])
        .compile()
        .unwrap()
    }

    fn file(&self, name: &str, content: &str) -> PathBuf {
        let p = self.src.join(name);
        fs::write(&p, content).unwrap();
        p
    }

    fn sort(&self, paths: &[PathBuf], resolver: &mut dyn ConflictResolver) -> SortReport {
        sort(paths, &self.rules(), Origin::Cli, resolver, &self.history).unwrap()
    }
}

fn keep_both() -> FixedResolver {
    FixedResolver(Resolution::KeepBoth)
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap()
}

/// Records the conflicts it is asked about and answers with a fixed decision.
struct Recording {
    seen: Vec<Conflict>,
    decision: Decision,
}

impl ConflictResolver for Recording {
    fn resolve(&mut self, conflict: &Conflict) -> Decision {
        self.seen.push(conflict.clone());
        self.decision
    }
}

#[test]
fn sorts_folder_contents_and_counts_ignored() {
    let env = Env::new();
    env.file("a.pdf", "A");
    env.file("photo.JPG", "P");
    env.file("script.sh", "S");
    env.file(".hidden.pdf", "H");
    fs::create_dir(env.src.join("sub")).unwrap();
    fs::write(env.src.join("sub/inner.pdf"), "I").unwrap();

    let report = env.sort(&[env.src.clone()], &mut keep_both());

    assert_eq!(report.counts.done, 2);
    assert_eq!(report.counts.ignored, 1);
    assert_eq!(report.counts.total(), 3, "hidden files are not counted");
    assert!(!report.has_errors());
    assert_eq!(read(&env.docs.join("a.pdf")), "A");
    assert_eq!(read(&env.pics.join("photo.JPG")), "P");
    assert!(env.src.join("script.sh").exists());
    assert!(env.src.join(".hidden.pdf").exists());
    assert!(env.src.join("sub/inner.pdf").exists());

    let batch = env.history.batch(report.batch_id.unwrap()).unwrap();
    assert_eq!(batch.entries.len(), 3);
    let ignored = env.history.ignored_types().unwrap();
    assert_eq!(ignored.len(), 1);
    assert_eq!(ignored[0].ext, "sh");
    assert_eq!(ignored[0].example, "script.sh");
    assert_eq!(batch.summary.origin, Origin::Cli);
}

#[test]
fn nothing_to_sort_creates_no_batch() {
    let env = Env::new();
    env.file(".DS_Store", "");
    let report = env.sort(&[env.src.clone()], &mut keep_both());
    assert_eq!(report.batch_id, None);
    assert!(env.history.list_batches(10).unwrap().is_empty());
}

#[test]
fn missing_input_is_reported() {
    let env = Env::new();
    let report = env.sort(&[env.src.join("ghost.pdf")], &mut keep_both());
    assert_eq!(report.counts.error, 1);
    assert_eq!(report.errors[0].error.code, ErrorCode::SourceMissing);
}

#[test]
fn file_already_in_destination_is_left_alone() {
    let env = Env::new();
    fs::create_dir_all(&env.docs).unwrap();
    let p = env.docs.join("x.pdf");
    fs::write(&p, "X").unwrap();
    let report = env.sort(&[p.clone()], &mut keep_both());
    assert_eq!(report.counts.already_in_place, 1);
    assert!(p.exists());
    assert_eq!(env.history.latest_undoable_batch().unwrap(), None);
}

#[test]
fn conflict_keep_both_renames_incoming() {
    let env = Env::new();
    fs::create_dir_all(&env.docs).unwrap();
    fs::write(env.docs.join("a.pdf"), "OLD").unwrap();
    env.file("a.pdf", "NEW");

    let mut r = Recording {
        seen: vec![],
        decision: Decision {
            resolution: Resolution::KeepBoth,
            apply_to_all: false,
        },
    };
    let report = env.sort(&[env.src.join("a.pdf")], &mut r);

    assert_eq!(report.counts.done, 1);
    assert_eq!(r.seen.len(), 1);
    assert_eq!(r.seen[0].existing, env.docs.join("a.pdf"));
    assert!(!r.seen[0].is_undo);
    assert_eq!(read(&env.docs.join("a.pdf")), "OLD");
    assert_eq!(read(&env.docs.join("a (1).pdf")), "NEW");
}

#[test]
fn conflict_skip_leaves_file_and_apply_to_all_asks_once() {
    let env = Env::new();
    fs::create_dir_all(&env.docs).unwrap();
    for n in ["a.pdf", "b.pdf", "c.txt"] {
        fs::write(env.docs.join(n), "OLD").unwrap();
        env.file(n, "NEW");
    }
    let mut r = Recording {
        seen: vec![],
        decision: Decision {
            resolution: Resolution::Skip,
            apply_to_all: true,
        },
    };
    let report = env.sort(&[env.src.clone()], &mut r);
    assert_eq!(r.seen.len(), 1, "apply to all must stop further questions");
    assert_eq!(report.counts.skipped, 3);
    for n in ["a.pdf", "b.pdf", "c.txt"] {
        assert_eq!(read(&env.src.join(n)), "NEW");
        assert_eq!(read(&env.docs.join(n)), "OLD");
    }
}

#[test]
fn existing_folder_with_same_name_is_never_replaced() {
    let env = Env::new();
    fs::create_dir_all(env.docs.join("a.pdf")).unwrap();
    env.file("a.pdf", "NEW");
    let mut r = Recording {
        seen: vec![],
        decision: Decision {
            resolution: Resolution::Overwrite,
            apply_to_all: true,
        },
    };
    env.sort(&[env.src.join("a.pdf")], &mut r);
    assert!(r.seen.is_empty());
    assert!(env.docs.join("a.pdf").is_dir());
    assert_eq!(read(&env.docs.join("a (1).pdf")), "NEW");
}

#[test]
fn conflict_overwrite_trashes_existing() {
    if !trash_tests_enabled() {
        return;
    }
    let env = Env::new();
    fs::create_dir_all(&env.docs).unwrap();
    fs::write(env.docs.join("a.pdf"), "OLD").unwrap();
    env.file("a.pdf", "NEW");
    let report = env.sort(
        &[env.src.join("a.pdf")],
        &mut FixedResolver(Resolution::Overwrite),
    );
    assert_eq!(report.counts.done, 1);
    assert_eq!(read(&env.docs.join("a.pdf")), "NEW");
    let entry = &env.history.batch(report.batch_id.unwrap()).unwrap().entries[0];
    assert!(entry.overwrote);

    let undo = undo_batch(&env.history, report.batch_id.unwrap(), &mut keep_both()).unwrap();
    assert_eq!(undo.undone, 1);
    assert_eq!(read(&env.src.join("a.pdf")), "NEW");
    if cfg!(target_os = "macos") {
        assert_eq!(undo.manual_trash_restores, vec![env.docs.join("a.pdf")]);
    }
}

#[test]
fn missing_volume_is_an_error_and_nothing_is_created() {
    let env = Env::new();
    let ghost = if cfg!(windows) {
        let free = ('D'..='Z')
            .map(|c| format!("{c}:\\"))
            .find(|d| !Path::new(d).exists());
        match free {
            Some(d) => PathBuf::from(d).join("Photos"),
            None => return,
        }
    } else {
        PathBuf::from("/Volumes/ordo-test-unmounted/Photos")
    };
    let rules = RuleSet::new(vec![Rule::new("Ghost", PatternKind::Glob, "*", &ghost)])
        .compile()
        .unwrap();
    let f = env.file("a.txt", "A");
    let report = sort(
        &[f.clone()],
        &rules,
        Origin::Cli,
        &mut keep_both(),
        &env.history,
    )
    .unwrap();
    assert_eq!(report.counts.error, 1);
    assert_eq!(report.errors[0].error.code, ErrorCode::VolumeMissing);
    assert!(f.exists());
    assert!(!ghost.exists());
}

#[test]
fn undo_batch_restores_everything() {
    let env = Env::new();
    env.file("a.pdf", "A");
    env.file("b.png", "B");
    env.file("c.zip", "C");
    let report = env.sort(&[env.src.clone()], &mut keep_both());
    let id = report.batch_id.unwrap();

    let undo = undo_latest(&env.history, &mut keep_both())
        .unwrap()
        .unwrap();
    assert_eq!(undo.batch_id, id);
    assert_eq!(undo.undone, 2);
    assert!(undo.errors.is_empty());
    assert_eq!(read(&env.src.join("a.pdf")), "A");
    assert_eq!(read(&env.src.join("b.png")), "B");
    assert!(!env.docs.join("a.pdf").exists());

    let statuses: Vec<_> = env
        .history
        .batch(id)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.status)
        .collect();
    assert_eq!(
        statuses,
        [MoveStatus::Undone, MoveStatus::Undone, MoveStatus::Ignored]
    );
    // Nothing left to undo.
    assert!(
        undo_latest(&env.history, &mut keep_both())
            .unwrap()
            .is_none()
    );
}

#[test]
fn undo_single_move_gives_partial_batch() {
    let env = Env::new();
    env.file("a.pdf", "A");
    env.file("b.pdf", "B");
    let id = env
        .sort(&[env.src.clone()], &mut keep_both())
        .batch_id
        .unwrap();
    let entries = env.history.batch(id).unwrap().entries;

    let out = undo_move(&env.history, entries[0].id, &mut keep_both()).unwrap();
    assert!(matches!(out, UndoOutcome::Undone { .. }));
    assert!(env.src.join("a.pdf").exists());
    assert!(env.docs.join("b.pdf").exists());
    assert_eq!(
        undo_move(&env.history, entries[0].id, &mut keep_both()).unwrap(),
        UndoOutcome::NotApplicable
    );
    let counts = env.history.batch(id).unwrap().summary.counts;
    assert_eq!((counts.done, counts.undone), (1, 1));
}

#[test]
fn undo_when_file_vanished_marks_not_undoable() {
    let env = Env::new();
    env.file("a.pdf", "A");
    env.file("b.pdf", "B");
    let id = env
        .sort(&[env.src.clone()], &mut keep_both())
        .batch_id
        .unwrap();
    fs::remove_file(env.docs.join("a.pdf")).unwrap();

    let undo = undo_batch(&env.history, id, &mut keep_both()).unwrap();
    assert_eq!(undo.undone, 1);
    assert_eq!(undo.not_undoable, 1);
    let entries = env.history.batch(id).unwrap().entries;
    assert_eq!(entries[0].status, MoveStatus::NotUndoable);
    assert_eq!(
        entries[0].error.as_ref().unwrap().code,
        ErrorCode::DestinationMissing
    );
}

#[test]
fn undo_with_occupied_original_path_asks() {
    let env = Env::new();
    env.file("a.pdf", "A");
    let id = env
        .sort(&[env.src.clone()], &mut keep_both())
        .batch_id
        .unwrap();
    env.file("a.pdf", "SQUATTER");

    // Skip: the move stays done and can be retried.
    let mut skip = Recording {
        seen: vec![],
        decision: Decision {
            resolution: Resolution::Skip,
            apply_to_all: false,
        },
    };
    let undo = undo_batch(&env.history, id, &mut skip).unwrap();
    assert_eq!(undo.skipped, 1);
    assert!(skip.seen[0].is_undo);
    assert_eq!(skip.seen[0].incoming, env.docs.join("a.pdf"));
    assert_eq!(env.history.latest_undoable_batch().unwrap(), Some(id));

    // Keep both: restored next to the squatter.
    let undo = undo_batch(&env.history, id, &mut keep_both()).unwrap();
    assert_eq!(undo.undone, 1);
    assert_eq!(read(&env.src.join("a.pdf")), "SQUATTER");
    assert_eq!(read(&env.src.join("a (1).pdf")), "A");
    let entry = &env.history.batch(id).unwrap().entries[0];
    assert_eq!(entry.restored_to, Some(env.src.join("a (1).pdf")));
}

#[test]
fn undo_recreates_missing_source_folder() {
    let env = Env::new();
    env.file("a.pdf", "A");
    let id = env
        .sort(&[env.src.clone()], &mut keep_both())
        .batch_id
        .unwrap();
    fs::remove_dir(&env.src).unwrap();
    let undo = undo_batch(&env.history, id, &mut keep_both()).unwrap();
    assert_eq!(undo.undone, 1);
    assert_eq!(read(&env.src.join("a.pdf")), "A");
}
