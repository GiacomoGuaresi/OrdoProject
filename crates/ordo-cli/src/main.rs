//! `ordo` — command-line interface of Ordo.

mod i18n;
mod prompt;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ordo_core::history::BatchSummary;
use ordo_core::{
    AppPaths, ConflictResolver, FixedResolver, History, Lang, MoveStatus, Origin, PatternKind,
    Resolution, Rule, RuleSet, Settings, TrashOutcome, UndoOutcome,
};
use serde::Serialize;

use crate::i18n::Tr;
use crate::prompt::TtyResolver;

/// Exit codes.
mod exit {
    pub const OK: u8 = 0;
    pub const ERROR: u8 = 1;
    // 2 is used by clap for usage errors.
    pub const PARTIAL: u8 = 3;
}

/// Ordo — sort files into folders using simple rules.
#[derive(Parser, Debug)]
#[command(name = "ordo", version = ordo_core::VERSION, about, propagate_version = true)]
struct Cli {
    /// Print machine-readable JSON on stdout.
    #[arg(long, global = true)]
    json: bool,

    /// Language of the messages (default: settings, then system).
    #[arg(long, global = true, value_enum)]
    lang: Option<LangArg>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LangArg {
    En,
    It,
}

impl From<LangArg> for Lang {
    fn from(l: LangArg) -> Self {
        match l {
            LangArg::En => Lang::En,
            LangArg::It => Lang::It,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OnConflict {
    KeepBoth,
    Overwrite,
    Skip,
}

impl From<OnConflict> for Resolution {
    fn from(c: OnConflict) -> Self {
        match c {
            OnConflict::KeepBoth => Resolution::KeepBoth,
            OnConflict::Overwrite => Resolution::Overwrite,
            OnConflict::Skip => Resolution::Skip,
        }
    }
}

#[derive(Debug, Args)]
struct ConflictArgs {
    /// How to resolve name conflicts. Without it, an interactive terminal is
    /// asked for each conflict; otherwise `keep-both` is used.
    #[arg(long, value_enum)]
    on_conflict: Option<OnConflict>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Sort files, or the files at the first level of folders.
    Sort {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        conflict: ConflictArgs,
    },
    /// Undo a batch (default: the latest one) or a single move.
    Undo {
        /// Batch to undo.
        batch: Option<i64>,
        /// Undo only this move (see `ordo history <batch>`).
        #[arg(long = "move", conflicts_with = "batch")]
        move_id: Option<i64>,
        #[command(flatten)]
        conflict: ConflictArgs,
    },
    /// Show recent batches, or the details of one batch.
    History {
        batch: Option<i64>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Manage the sorting rules.
    #[command(subcommand)]
    Rules(RulesCommand),
    /// Install or remove the "Sort with Ordo" context-menu entry.
    #[command(subcommand)]
    Integration(IntegrationCommand),
}

#[derive(Debug, Subcommand)]
enum IntegrationCommand {
    /// Add the entry to the file manager's context menu.
    Install {
        /// Path of the Ordo desktop app (default: next to this command).
        #[arg(long)]
        app: Option<PathBuf>,
    },
    /// Remove the entry from the context menu.
    Uninstall,
    /// Show whether the entry is installed.
    Status,
}

#[derive(Debug, Subcommand)]
enum RulesCommand {
    /// List the rules in order.
    List,
    /// Add a rule.
    Add {
        #[arg(long)]
        name: String,
        /// Glob on the file name, e.g. "*.{jpg,png}".
        #[arg(long, conflicts_with = "regex", required_unless_present = "regex")]
        glob: Option<String>,
        /// Regular expression on the file name.
        #[arg(long)]
        regex: Option<String>,
        /// Destination folder.
        #[arg(long)]
        dest: PathBuf,
        #[arg(long)]
        case_sensitive: bool,
        /// Add the rule disabled.
        #[arg(long)]
        disabled: bool,
        /// 1-based position (default: last).
        #[arg(long)]
        position: Option<usize>,
    },
    /// Remove a rule (by position or id).
    Remove { rule: String },
    /// Move a rule to a new 1-based position.
    Move { rule: String, position: usize },
    /// Enable a rule.
    Enable { rule: String },
    /// Disable a rule.
    Disable { rule: String },
    /// Replace the rules with those of a file exported by Ordo.
    Import { file: PathBuf },
    /// Export the rules to a file.
    Export { file: PathBuf },
}

struct Ctx {
    paths: AppPaths,
    settings: Settings,
    tr: Tr,
    json: bool,
}

impl Ctx {
    fn lang(&self) -> Lang {
        self.tr.0
    }

    fn rules(&self) -> Result<RuleSet> {
        Ok(RuleSet::load_or_init(
            &self.paths.rules_file(),
            self.lang(),
        )?)
    }

    fn save_rules(&self, rules: &RuleSet) -> Result<()> {
        Ok(rules.save(&self.paths.rules_file())?)
    }

    fn history(&self) -> Result<History> {
        let h = History::open(&self.paths.history_db())?;
        h.apply_retention(self.settings.retention_days)?;
        Ok(h)
    }

    fn resolver(&self, args: &ConflictArgs) -> Box<dyn ConflictResolver> {
        let interactive =
            !self.json && std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
        match args.on_conflict {
            Some(c) => Box::new(FixedResolver(c.into())),
            None if interactive => Box::new(TtyResolver { tr: self.tr }),
            None => Box::new(FixedResolver(Resolution::KeepBoth)),
        }
    }

    fn print_json<T: Serialize>(&self, value: &T) {
        println!(
            "{}",
            serde_json::to_string_pretty(value).expect("serializable")
        );
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            if json {
                println!("{}", serde_json::json!({ "error": format!("{e:#}") }));
            }
            eprintln!("ordo: {e:#}");
            ExitCode::from(exit::ERROR)
        }
    }
}

fn run(cli: Cli) -> Result<u8> {
    let paths = AppPaths::resolve()?;
    let settings = Settings::load(&paths.settings_file())?;
    let lang = cli
        .lang
        .map(Lang::from)
        .unwrap_or_else(|| settings.effective_lang());
    let ctx = Ctx {
        paths,
        settings,
        tr: Tr(lang),
        json: cli.json,
    };
    match cli.command {
        Command::Sort { paths, conflict } => cmd_sort(&ctx, &paths, &conflict),
        Command::Undo {
            batch,
            move_id,
            conflict,
        } => cmd_undo(&ctx, batch, move_id, &conflict),
        Command::History { batch, limit } => cmd_history(&ctx, batch, limit),
        Command::Rules(cmd) => cmd_rules(&ctx, cmd),
        Command::Integration(cmd) => cmd_integration(&ctx, cmd),
    }
}

fn file_name(p: &std::path::Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.display().to_string())
}

fn cmd_sort(ctx: &Ctx, paths: &[PathBuf], conflict: &ConflictArgs) -> Result<u8> {
    let rules = ctx.rules()?.compile()?;
    let history = ctx.history()?;
    let mut resolver = ctx.resolver(conflict);
    let report = ordo_core::sort(paths, &rules, Origin::Cli, resolver.as_mut(), &history)?;
    let code = if report.has_errors() {
        if report.counts.done > 0 {
            exit::PARTIAL
        } else {
            exit::ERROR
        }
    } else {
        exit::OK
    };

    if ctx.json {
        ctx.print_json(&report);
        return Ok(code);
    }
    let tr = ctx.tr;
    let Some(batch_id) = report.batch_id else {
        println!("{}", tr.t("sort.nothing", &[]));
        return Ok(code);
    };
    for e in history.batch(batch_id)?.entries {
        if e.status == MoveStatus::Done
            && let Some(dest) = &e.destination
        {
            println!("  {} → {}", file_name(&e.source), dest.display());
        }
    }
    let c = report.counts;
    let mut parts = vec![tr.t("sort.moved", &[&c.done])];
    if c.ignored > 0 {
        parts.push(tr.t("sort.ignored", &[&c.ignored]));
    }
    if c.skipped > 0 {
        parts.push(tr.t("sort.skipped", &[&c.skipped]));
    }
    if c.already_in_place > 0 {
        parts.push(tr.t("sort.in_place", &[&c.already_in_place]));
    }
    if c.error > 0 {
        parts.push(tr.t("sort.errors", &[&c.error]));
    }
    println!("{}", parts.join(", "));
    for e in &report.errors {
        eprintln!(
            "  ✗ {}: {} ({})",
            e.path.display(),
            tr.error_code(e.error.code),
            e.error.detail
        );
    }
    if c.done > 0 {
        println!("{}", tr.t("sort.batch", &[&batch_id, &batch_id]));
    }
    Ok(code)
}

fn cmd_undo(
    ctx: &Ctx,
    batch: Option<i64>,
    move_id: Option<i64>,
    conflict: &ConflictArgs,
) -> Result<u8> {
    let history = ctx.history()?;
    let mut resolver = ctx.resolver(conflict);
    let tr = ctx.tr;

    if let Some(move_id) = move_id {
        let outcome = ordo_core::undo_move(&history, move_id, resolver.as_mut())?;
        let code = match outcome {
            UndoOutcome::Failed { .. } => exit::ERROR,
            _ => exit::OK,
        };
        if ctx.json {
            ctx.print_json(&outcome);
            return Ok(code);
        }
        match outcome {
            UndoOutcome::Undone { restored_to, trash } => {
                println!("{}", tr.t("undo.move_done", &[&restored_to.display()]));
                print_trash(tr, trash.as_ref());
            }
            UndoOutcome::NotUndoable => {
                let entry = history.entry(move_id)?;
                let dest = entry.destination.unwrap_or_default();
                println!("{}", tr.t("undo.move_not_undoable", &[&dest.display()]));
            }
            UndoOutcome::Skipped => println!("{}", tr.t("undo.move_skipped", &[])),
            UndoOutcome::NotApplicable => {
                println!("{}", tr.t("undo.move_not_applicable", &[]))
            }
            UndoOutcome::Failed { error } => {
                eprintln!("✗ {} ({})", tr.error_code(error.code), error.detail)
            }
        }
        return Ok(code);
    }

    let report = match batch {
        Some(id) => Some(ordo_core::undo_batch(&history, id, resolver.as_mut())?),
        None => ordo_core::undo_latest(&history, resolver.as_mut())?,
    };
    let Some(report) = report else {
        if ctx.json {
            ctx.print_json(&serde_json::Value::Null);
        } else {
            println!("{}", tr.t("undo.nothing", &[]));
        }
        return Ok(exit::OK);
    };
    let code = match (report.errors.is_empty(), report.undone) {
        (true, _) => exit::OK,
        (false, 0) => exit::ERROR,
        (false, _) => exit::PARTIAL,
    };
    if ctx.json {
        ctx.print_json(&report);
        return Ok(code);
    }
    let mut parts = vec![tr.t("undo.done", &[&report.batch_id, &report.undone])];
    if report.not_undoable > 0 {
        parts.push(tr.t("undo.not_undoable", &[&report.not_undoable]));
    }
    if report.skipped > 0 {
        parts.push(tr.t("undo.skipped", &[&report.skipped]));
    }
    if !report.errors.is_empty() {
        parts.push(tr.t("undo.errors", &[&report.errors.len()]));
    }
    println!("{}", parts.join(", "));
    for e in &report.errors {
        eprintln!(
            "  ✗ {}: {} ({})",
            e.path.display(),
            tr.error_code(e.error.code),
            e.error.detail
        );
    }
    for path in &report.manual_trash_restores {
        println!("{}", tr.t("undo.trash_manual", &[&path.display()]));
    }
    Ok(code)
}

fn print_trash(tr: Tr, trash: Option<&TrashOutcome>) {
    match trash {
        Some(TrashOutcome::Restored) => println!("{}", tr.t("undo.trash_restored", &[])),
        Some(TrashOutcome::Manual { path }) => {
            println!("{}", tr.t("undo.trash_manual", &[&path.display()]))
        }
        None => {}
    }
}

fn origin_label(lang: Lang, b: &BatchSummary) -> &'static str {
    let it = lang == Lang::It;
    match b.origin {
        Origin::Cli => "cli",
        Origin::ContextMenu if it => "menu contest.",
        Origin::ContextMenu => "context menu",
        Origin::App => "app",
        Origin::Launcher => "launcher",
    }
}

fn cmd_history(ctx: &Ctx, batch: Option<i64>, limit: usize) -> Result<u8> {
    let history = ctx.history()?;
    let tr = ctx.tr;
    if let Some(id) = batch {
        let detail = history.batch(id)?;
        if ctx.json {
            ctx.print_json(&detail);
            return Ok(exit::OK);
        }
        let date = detail.summary.created_at.with_timezone(&Local);
        println!(
            "Batch #{} — {} — {}",
            id,
            date.format("%Y-%m-%d %H:%M:%S"),
            origin_label(ctx.lang(), &detail.summary)
        );
        for e in detail.entries {
            let target = e
                .destination
                .as_ref()
                .map(|d| format!(" → {}", d.display()))
                .unwrap_or_default();
            let err = e
                .error
                .as_ref()
                .map(|x| format!(" ({})", tr.error_code(x.code)))
                .unwrap_or_default();
            println!(
                "  [{:>5}] {:<16} {}{}{}",
                e.id,
                tr.status(e.status),
                e.source.display(),
                target,
                err
            );
        }
        return Ok(exit::OK);
    }

    let batches = history.list_batches(limit)?;
    if ctx.json {
        ctx.print_json(&batches);
        return Ok(exit::OK);
    }
    if batches.is_empty() {
        println!("{}", tr.t("history.empty", &[]));
        return Ok(exit::OK);
    }
    println!("{}", tr.t("history.header", &[]));
    for b in &batches {
        let c = b.counts;
        println!(
            "{:<7} {:<17} {:<13} {:>5}  {:>6}  {:>7}  {:>6}",
            b.id,
            b.created_at.with_timezone(&Local).format("%Y-%m-%d %H:%M"),
            origin_label(ctx.lang(), b),
            c.moved(),
            c.undone,
            c.ignored,
            c.error
        );
    }
    Ok(exit::OK)
}

fn cmd_rules(ctx: &Ctx, cmd: RulesCommand) -> Result<u8> {
    let tr = ctx.tr;
    let mut set = ctx.rules()?;
    let msg = match cmd {
        RulesCommand::List => {
            if ctx.json {
                ctx.print_json(&set);
                return Ok(exit::OK);
            }
            if set.rules.is_empty() {
                println!("{}", tr.t("rules.empty", &[]));
            }
            for (i, r) in set.rules.iter().enumerate() {
                let kind = match r.kind {
                    PatternKind::Glob => "glob",
                    PatternKind::Regex => "regex",
                };
                let mut flags = Vec::new();
                if r.case_sensitive {
                    flags.push(tr.t("rules.case_sensitive", &[]));
                }
                if !r.enabled {
                    flags.push(tr.t("rules.off", &[]));
                }
                let flags = if flags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", flags.join(", "))
                };
                println!(
                    "{:>3}. {}{}\n     {} {}  →  {}",
                    i + 1,
                    r.name,
                    flags,
                    kind,
                    r.pattern,
                    r.destination.display()
                );
            }
            return Ok(exit::OK);
        }
        RulesCommand::Add {
            name,
            glob,
            regex,
            dest,
            case_sensitive,
            disabled,
            position,
        } => {
            let (kind, pattern) = match (glob, regex) {
                (Some(g), _) => (PatternKind::Glob, g),
                (None, Some(r)) => (PatternKind::Regex, r),
                (None, None) => bail!("either --glob or --regex is required"),
            };
            let dest = std::path::absolute(&dest)
                .with_context(|| format!("invalid destination {}", dest.display()))?;
            let mut rule = Rule::new(name, kind, pattern, dest);
            rule.case_sensitive = case_sensitive;
            rule.enabled = !disabled;
            rule.validate()?;
            let index = position
                .map(|p| p.clamp(1, set.rules.len() + 1) - 1)
                .unwrap_or(set.rules.len());
            let msg = tr.t("rules.added", &[&rule.name, &(index + 1)]);
            set.rules.insert(index, rule);
            msg
        }
        RulesCommand::Remove { rule } => {
            let i = set.find_index(&rule)?;
            let removed = set.rules.remove(i);
            tr.t("rules.removed", &[&removed.name])
        }
        RulesCommand::Move { rule, position } => {
            let from = set.find_index(&rule)?;
            if position == 0 || position > set.rules.len() {
                bail!("position must be between 1 and {}", set.rules.len());
            }
            set.move_rule(from, position - 1)?;
            tr.t("rules.moved", &[&set.rules[position - 1].name, &position])
        }
        RulesCommand::Enable { rule } => {
            let i = set.find_index(&rule)?;
            set.rules[i].enabled = true;
            tr.t("rules.enabled", &[&set.rules[i].name])
        }
        RulesCommand::Disable { rule } => {
            let i = set.find_index(&rule)?;
            set.rules[i].enabled = false;
            tr.t("rules.disabled", &[&set.rules[i].name])
        }
        RulesCommand::Import { file } => {
            set = RuleSet::import(&file)?;
            tr.t("rules.imported", &[&set.rules.len()])
        }
        RulesCommand::Export { file } => {
            set.save(&file)?;
            let msg = tr.t("rules.exported", &[&set.rules.len(), &file.display()]);
            if ctx.json {
                ctx.print_json(&serde_json::json!({ "message": msg }));
            } else {
                println!("{msg}");
            }
            return Ok(exit::OK);
        }
    };
    ctx.save_rules(&set)?;
    if ctx.json {
        ctx.print_json(&serde_json::json!({ "message": msg, "rules": set.rules }));
    } else {
        println!("{msg}");
    }
    Ok(exit::OK)
}

/// Names of the desktop app executable, looked up next to the CLI.
const APP_NAMES: &[&str] = if cfg!(windows) {
    &["ordo-app.exe", "Ordo.exe"]
} else {
    &["ordo-app", "Ordo"]
};

fn find_app(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(app) = explicit {
        return Ok(std::path::absolute(&app)?);
    }
    let exe = std::env::current_exe().context("cannot locate the ordo executable")?;
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    APP_NAMES
        .iter()
        .map(|name| exe.with_file_name(name))
        .find(|p| p.is_file())
        .with_context(|| {
            format!(
                "the Ordo app was not found next to {}; pass it with --app",
                exe.display()
            )
        })
}

fn cmd_integration(ctx: &Ctx, cmd: IntegrationCommand) -> Result<u8> {
    use ordo_core::integration::Integration;
    let tr = ctx.tr;
    let status = match cmd {
        IntegrationCommand::Install { app } => {
            Integration::new(find_app(app)?, ctx.lang())?.install()?
        }
        IntegrationCommand::Uninstall => {
            // The app path is not needed to remove the entries.
            Integration::new(PathBuf::new(), ctx.lang())?.uninstall()?
        }
        IntegrationCommand::Status => Integration::new(PathBuf::new(), ctx.lang())?.status(),
    };
    if ctx.json {
        ctx.print_json(&status);
        return Ok(exit::OK);
    }
    for item in &status.items {
        let state = match (item.available, item.installed) {
            (_, true) => tr.t("integration.installed", &[]),
            (true, false) => tr.t("integration.not_installed", &[]),
            (false, false) => tr.t("integration.unavailable", &[]),
        };
        println!("{:<24} {:<16} {}", item.name, state, item.location);
    }
    Ok(exit::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
