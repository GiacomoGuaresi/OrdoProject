//! Tauri commands exposed to the web UI.

use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;

use ordo_core::{
    AppPaths, BatchDetail, BatchSummary, ConflictRequest, Decision, History, Lang, Rule, RuleSet,
    Settings, UndoOutcome, UndoReport,
};
use tauri::{AppHandle, State, WebviewWindow};

use ordo_core::integration::command::{self, CommandStatus};
use ordo_core::integration::{Integration, IntegrationStatus};

use crate::conflicts::{ConflictBroker, DialogResolver};
use crate::instance::InstanceDir;
use crate::sortmode::SortQueue;
use crate::windows;

/// Error returned to the UI: a human-readable message.
type CmdResult<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Shared state of the app.
pub struct AppState {
    pub paths: AppPaths,
    pub broker: ConflictBroker,
    pub sort_queue: SortQueue,
    pub instance: InstanceDir,
    /// Lock held while this process is the primary instance.
    pub primary_lock: Mutex<Option<File>>,
}

impl AppState {
    fn settings(&self) -> CmdResult<Settings> {
        Settings::load(&self.paths.settings_file()).map_err(err)
    }

    fn history(&self) -> CmdResult<History> {
        History::open(&self.paths.history_db()).map_err(err)
    }
}

#[tauri::command]
pub fn system_lang() -> Lang {
    Lang::system()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CmdResult<Settings> {
    state.settings()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> CmdResult<()> {
    settings.save(&state.paths.settings_file()).map_err(err)?;
    state
        .history()?
        .apply_retention(settings.retention_days)
        .map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn get_rules(state: State<'_, AppState>) -> CmdResult<RuleSet> {
    let lang = state.settings()?.effective_lang();
    RuleSet::load_or_init(&state.paths.rules_file(), lang).map_err(err)
}

/// Checks a rule; returns the error message, or `None` when valid.
#[tauri::command]
pub fn validate_rule(rule: Rule) -> Option<String> {
    rule.validate().err().map(|e| e.to_string())
}

#[tauri::command]
pub fn save_rules(state: State<'_, AppState>, rules: RuleSet) -> CmdResult<()> {
    for rule in &rules.rules {
        rule.validate().map_err(err)?;
    }
    rules.save(&state.paths.rules_file()).map_err(err)
}

#[tauri::command]
pub fn import_rules(state: State<'_, AppState>, path: PathBuf) -> CmdResult<RuleSet> {
    let rules = RuleSet::import(&path).map_err(err)?;
    rules.save(&state.paths.rules_file()).map_err(err)?;
    Ok(rules)
}

#[tauri::command]
pub fn export_rules(state: State<'_, AppState>, path: PathBuf) -> CmdResult<()> {
    let lang = state.settings()?.effective_lang();
    RuleSet::load_or_init(&state.paths.rules_file(), lang)
        .and_then(|r| r.save(&path))
        .map_err(err)
}

#[tauri::command]
pub fn default_rules(state: State<'_, AppState>) -> CmdResult<Vec<Rule>> {
    let lang = state.settings()?.effective_lang();
    ordo_core::rules::default_rules(lang).map_err(err)
}

#[tauri::command]
pub fn list_batches(state: State<'_, AppState>, limit: usize) -> CmdResult<Vec<BatchSummary>> {
    let history = state.history()?;
    history
        .apply_retention(state.settings()?.retention_days)
        .map_err(err)?;
    history.list_batches(limit).map_err(err)
}

#[tauri::command]
pub fn get_batch(state: State<'_, AppState>, id: i64) -> CmdResult<BatchDetail> {
    state.history()?.batch(id).map_err(err)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> CmdResult<()> {
    state.history()?.clear().map_err(err)
}

/// Resolver for an undo started from `window`: conflicts are asked in the
/// same window, except for the small summary window, which delegates them to
/// the conflict window.
fn undo_resolver<'a>(app: &AppHandle, state: &'a AppState, window: &str) -> DialogResolver<'a> {
    if window == windows::SUMMARY {
        DialogResolver::new(app.clone(), &state.broker, windows::CONFLICTS).with_prepare(|app| {
            if let Err(e) = windows::show_conflicts(app) {
                log::error!("cannot open the conflict window: {e}");
            }
        })
    } else {
        DialogResolver::new(app.clone(), &state.broker, window)
    }
}

/// Undoes a batch.
#[tauri::command]
pub async fn undo_batch(app: AppHandle, window: WebviewWindow, id: i64) -> CmdResult<UndoReport> {
    let label = window.label().to_string();
    run_blocking(app, move |app, state| {
        let history = state.history()?;
        let mut resolver = undo_resolver(app, state, &label);
        let report = ordo_core::undo_batch(&history, id, &mut resolver).map_err(err);
        windows::close(app, windows::CONFLICTS);
        report
    })
    .await
}

/// Undoes a single move.
#[tauri::command]
pub async fn undo_move(app: AppHandle, window: WebviewWindow, id: i64) -> CmdResult<UndoOutcome> {
    let label = window.label().to_string();
    run_blocking(app, move |app, state| {
        let history = state.history()?;
        let mut resolver = undo_resolver(app, state, &label);
        ordo_core::undo_move(&history, id, &mut resolver).map_err(err)
    })
    .await
}

/// Conflicts addressed to the calling window that are still unanswered.
#[tauri::command]
pub fn pending_conflicts(
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Vec<ConflictRequest> {
    state.broker.pending_for(window.label())
}

/// Opens (or focuses) the main window, optionally on a batch of the history.
#[tauri::command]
pub fn open_main(app: AppHandle, batch: Option<i64>) -> CmdResult<()> {
    windows::show_main(&app, batch).map_err(err)
}

#[tauri::command]
pub fn resolve_conflict(state: State<'_, AppState>, id: u64, decision: Decision) -> bool {
    state.broker.answer(id, decision)
}

/// Shows a file (or, if it no longer exists, its folder) in the file manager.
#[tauri::command]
pub fn reveal_path(path: PathBuf) -> CmdResult<()> {
    let target = if path.exists() {
        path
    } else {
        path.ancestors()
            .find(|p| p.exists())
            .map(PathBuf::from)
            .ok_or_else(|| format!("{} does not exist", path.display()))?
    };
    tauri_plugin_opener::reveal_item_in_dir(&target).map_err(err)
}

/// Postpones (seconds) or suspends (`None`) the automatic close of the summary.
#[tauri::command]
pub fn summary_keep_open(app: AppHandle, seconds: Option<u64>) {
    use tauri::Manager;
    app.state::<windows::SummaryTimer>()
        .set(seconds.map(std::time::Duration::from_secs));
}

/// Largest image turned into a thumbnail.
const MAX_THUMBNAIL_BYTES: u64 = 15 * 1024 * 1024;

fn image_mime(path: &std::path::Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "avif" => "image/avif",
        _ => return None,
    })
}

/// A `data:` URL with the image, for the conflict dialog; `None` for other
/// files or images that are too large.
#[tauri::command]
pub async fn file_thumbnail(path: PathBuf) -> Option<String> {
    use base64::Engine;
    let mime = image_mime(&path)?;
    let meta = std::fs::metadata(&path).ok()?;
    if !meta.is_file() || meta.len() > MAX_THUMBNAIL_BYTES {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    let data = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{data}"))
}

/// Shows a quick preview: Quick Look on macOS, the default app elsewhere.
#[tauri::command]
pub fn preview_file(path: PathBuf) -> CmdResult<()> {
    if !path.is_file() {
        return Err(format!("{} does not exist", path.display()));
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("qlmanage")
            .arg("-p")
            .arg(&path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(err)
    }
    #[cfg(not(target_os = "macos"))]
    {
        tauri_plugin_opener::open_path(&path, None::<&str>).map_err(err)
    }
}

/// Executable that the file manager should start: the AppImage itself when
/// running from one (the binary lives in a temporary mount), otherwise this
/// process' executable.
fn app_executable() -> CmdResult<PathBuf> {
    if let Some(appimage) = std::env::var_os("APPIMAGE") {
        return Ok(PathBuf::from(appimage));
    }
    std::env::current_exe().map_err(err)
}

/// On start: create the "Ordo" launcher the first time and keep it pointing
/// at this app. Skipped in development builds.
pub fn ensure_launcher(state: &AppState) {
    if cfg!(debug_assertions) {
        return;
    }
    let result = (|| -> CmdResult<()> {
        let mut settings = state.settings()?;
        let integration = context_menu(state)?;
        integration
            .ensure_launcher(settings.launcher_offered)
            .map_err(err)?;
        if !settings.launcher_offered {
            settings.launcher_offered = true;
            settings.save(&state.paths.settings_file()).map_err(err)?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        log::warn!("cannot set up the Ordo launcher: {e}");
    }
}

fn context_menu(state: &AppState) -> CmdResult<Integration> {
    let lang = state.settings()?.effective_lang();
    Integration::new(app_executable()?, lang).map_err(err)
}

#[tauri::command]
pub fn integration_status(state: State<'_, AppState>) -> CmdResult<IntegrationStatus> {
    Ok(context_menu(&state)?.status())
}

#[tauri::command]
pub async fn integration_install(app: AppHandle) -> CmdResult<IntegrationStatus> {
    run_blocking(app, |_, state| context_menu(state)?.install().map_err(err)).await
}

#[tauri::command]
pub async fn integration_uninstall(app: AppHandle) -> CmdResult<IntegrationStatus> {
    run_blocking(app, |_, state| {
        context_menu(state)?.uninstall().map_err(err)
    })
    .await
}

#[tauri::command]
pub fn command_status() -> CmdResult<CommandStatus> {
    command::status().map_err(err)
}

/// Installs the `ordo` command (may show the system's administrator prompt).
#[tauri::command]
pub async fn command_install(app: AppHandle) -> CmdResult<CommandStatus> {
    run_blocking(app, |_, _| {
        let exe = std::env::current_exe().map_err(err)?;
        command::install(&command::cli_next_to(&exe)).map_err(err)
    })
    .await
}

#[tauri::command]
pub async fn command_uninstall(app: AppHandle) -> CmdResult<CommandStatus> {
    run_blocking(app, |_, _| command::uninstall().map_err(err)).await
}

/// Runs blocking core work (which may wait for conflict answers) off the
/// async runtime.
async fn run_blocking<T, F>(app: AppHandle, f: F) -> CmdResult<T>
where
    T: Send + 'static,
    F: FnOnce(&AppHandle, &AppState) -> CmdResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        use tauri::Manager;
        let state = app.state::<AppState>();
        f(&app, &state)
    })
    .await
    .map_err(err)?
}
