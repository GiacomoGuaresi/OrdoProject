//! Integration tests of the `ordo` binary.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

struct Env {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Env {
    /// An isolated Ordo home with an empty rule list.
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(tmp.path()).unwrap();
        fs::create_dir_all(root.join("home")).unwrap();
        fs::write(root.join("home/rules.json"), r#"{"version":1,"rules":[]}"#).unwrap();
        fs::create_dir_all(root.join("in")).unwrap();
        Env { _tmp: tmp, root }
    }

    fn ordo(&self) -> Command {
        let mut cmd = Command::cargo_bin("ordo").unwrap();
        cmd.env("ORDO_HOME", self.root.join("home"))
            .current_dir(&self.root)
            .arg("--lang")
            .arg("en");
        cmd
    }

    fn json(&self, args: &[&str]) -> Value {
        let out = self.ordo().arg("--json").args(args).output().unwrap();
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
            panic!(
                "invalid JSON ({e}): {}\nstderr: {}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        })
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn file(&self, rel: &str, content: &str) -> PathBuf {
        let p = self.path(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, content).unwrap();
        p
    }

    fn add_rules(&self) {
        self.ordo()
            .args([
                "rules", "add", "--name", "Docs", "--glob", "*.pdf", "--dest", "out/docs",
            ])
            .assert()
            .success();
        self.ordo()
            .args([
                "rules",
                "add",
                "--name",
                "Pics",
                "--regex",
                r"\.jpe?g$",
                "--dest",
                "out/pics",
            ])
            .assert()
            .success();
    }
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap()
}

#[test]
fn help_and_version() {
    let env = Env::new();
    env.ordo()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("sort"));
    env.ordo()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
    env.ordo().arg("bogus").assert().code(2);
}

#[test]
fn default_rules_created_on_first_use() {
    let env = Env::new();
    fs::remove_file(env.path("home/rules.json")).unwrap();
    let rules = env.json(&["rules", "list"]);
    assert!(rules["rules"].as_array().unwrap().len() >= 5);
    assert!(env.path("home/rules.json").exists());
}

#[test]
fn rules_management() {
    let env = Env::new();
    env.add_rules();
    env.ordo()
        .args([
            "rules", "add", "--name", "Bad", "--regex", "(", "--dest", "x",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("invalid pattern"));

    env.ordo()
        .args(["rules", "move", "2", "1"])
        .assert()
        .success();
    env.ordo()
        .args(["rules", "disable", "1"])
        .assert()
        .success();
    let set = env.json(&["rules", "list"]);
    let rules = set["rules"].as_array().unwrap();
    assert_eq!(rules[0]["name"], "Pics");
    assert_eq!(rules[0]["enabled"], false);
    assert!(Path::new(rules[1]["destination"].as_str().unwrap()).is_absolute());

    env.ordo()
        .args(["rules", "export", "export.json"])
        .assert()
        .success();
    env.ordo().args(["rules", "remove", "1"]).assert().success();
    env.ordo().args(["rules", "remove", "9"]).assert().code(1);
    assert_eq!(
        env.json(&["rules", "list"])["rules"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    env.ordo()
        .args(["rules", "import", "export.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 rules imported"));
}

#[test]
fn sort_history_and_undo() {
    let env = Env::new();
    env.add_rules();
    env.file("in/a.pdf", "A");
    env.file("in/b.JPG", "B");
    env.file("in/c.sh", "C");

    env.ordo()
        .args(["sort", "in"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2 sorted, 1 ignored"));
    assert_eq!(read(&env.path("out/docs/a.pdf")), "A");
    assert_eq!(read(&env.path("out/pics/b.JPG")), "B");

    let history = env.json(&["history"]);
    assert_eq!(history[0]["counts"]["done"], 2);
    assert_eq!(history[0]["origin"], "cli");
    let batch = env.json(&["history", "1"]);
    assert_eq!(batch["entries"].as_array().unwrap().len(), 3);

    env.ordo()
        .args(["undo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch #1: 2 restored"));
    assert_eq!(read(&env.path("in/a.pdf")), "A");
    env.ordo()
        .args(["undo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to undo"));
}

#[test]
fn undo_single_move() {
    let env = Env::new();
    env.add_rules();
    env.file("in/a.pdf", "A");
    env.file("in/b.pdf", "B");
    env.ordo().args(["sort", "in"]).assert().success();
    let first = env.json(&["history", "1"])["entries"][0]["id"]
        .as_i64()
        .unwrap();

    let out = env.json(&["undo", "--move", &first.to_string()]);
    assert_eq!(out["kind"], "undone");
    assert!(env.path("in/a.pdf").exists());
    assert!(env.path("out/docs/b.pdf").exists());
}

#[test]
fn conflicts_default_to_keep_both_when_not_interactive() {
    let env = Env::new();
    env.add_rules();
    env.file("out/docs/a.pdf", "OLD");
    env.file("in/a.pdf", "NEW");
    env.ordo().args(["sort", "in/a.pdf"]).assert().success();
    assert_eq!(read(&env.path("out/docs/a.pdf")), "OLD");
    assert_eq!(read(&env.path("out/docs/a (1).pdf")), "NEW");
}

#[test]
fn conflicts_skip_flag() {
    let env = Env::new();
    env.add_rules();
    env.file("out/docs/a.pdf", "OLD");
    env.file("in/a.pdf", "NEW");
    let report = env.json(&["sort", "in", "--on-conflict", "skip"]);
    assert_eq!(report["counts"]["skipped"], 1);
    assert_eq!(read(&env.path("in/a.pdf")), "NEW");
}

#[test]
fn exit_codes_for_errors_and_partial_success() {
    let env = Env::new();
    env.add_rules();
    env.ordo()
        .args(["sort", "missing.pdf"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("file not found"));

    env.file("in/a.pdf", "A");
    env.ordo()
        .args(["sort", "in/a.pdf", "missing.pdf"])
        .assert()
        .code(3);

    env.ordo().args(["history", "999"]).assert().code(1);
}

#[test]
fn json_errors_are_reported_on_stdout() {
    let env = Env::new();
    let v = env.json(&["history", "999"]);
    assert!(v["error"].as_str().unwrap().contains("999"));
}

#[test]
fn italian_messages() {
    let env = Env::new();
    env.add_rules();
    env.file("in/a.pdf", "A");
    let mut cmd = Command::cargo_bin("ordo").unwrap();
    cmd.env("ORDO_HOME", env.path("home"))
        .current_dir(&env.root)
        .args(["--lang", "it", "sort", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 ordinati"));
}

#[test]
fn nothing_to_sort() {
    let env = Env::new();
    env.file("in/.hidden", "");
    env.ordo()
        .args(["sort", "in"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to sort"));
}

#[test]
fn integration_status_and_missing_app() {
    let env = Env::new();
    let status = env.json(&["integration", "status"]);
    assert!(status["items"].is_array());
    env.ordo()
        .args(["integration", "install", "--app", "does-not-exist"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn ignored_types_list() {
    let env = Env::new();
    env.add_rules();
    env.file("in/a.sh", "A");
    env.file("in/b.SH", "B");
    env.file("in/c.pdf", "C");
    env.ordo().args(["sort", "in"]).assert().success();
    let list = env.json(&["ignored"]);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["ext"], "sh");
    assert_eq!(list[0]["count"], 2);
    // A new rule covering the extension removes it from the list.
    env.ordo()
        .args([
            "rules", "add", "--name", "Scripts", "--glob", "*.sh", "--dest", "out/sh",
        ])
        .assert()
        .success();
    assert!(env.json(&["ignored"]).as_array().unwrap().is_empty());
}
