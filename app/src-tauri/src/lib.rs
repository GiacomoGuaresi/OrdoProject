//! Ordo desktop app (Tauri).
//!
//! Started without arguments it opens the main window. Started with
//! `--sort <paths…>` (from the file manager's context menu) it opens no main
//! window: it sorts the paths, asks about conflicts only if needed, shows a
//! summary with "Undo" and exits.

mod commands;
mod conflicts;
mod instance;
mod sortmode;
mod windows;

use std::sync::Mutex;
use std::time::Duration;

use commands::AppState;
use instance::{InstanceDir, Message};
use ordo_core::AppPaths;
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};

/// How often the primary instance checks for requests from other processes.
const INBOX_POLL: Duration = Duration::from_millis(100);

/// Exits when no window is open and no sort is pending.
///
/// The primary lock is released first and the inbox checked again, so that a
/// request sent at the same moment is never lost (see [`instance`]).
pub(crate) fn exit_if_idle(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !app.webview_windows().is_empty() || state.sort_queue.is_busy() {
        return;
    }
    let mut lock = state.primary_lock.lock().expect("poisoned");
    drop(lock.take());
    if state.instance.has_pending()
        && let Ok(Some(file)) = state.instance.try_acquire()
    {
        *lock = Some(file);
        return; // keep running: the inbox watcher will handle the request
    }
    app.exit(0);
}

fn dispatch(app: &AppHandle, message: Message) {
    match message {
        Message::Sort { paths } => app.state::<AppState>().sort_queue.add(app, paths),
        Message::Show => {
            if let Err(e) = windows::show_main(app, None) {
                log::error!("cannot open the main window: {e}");
            }
        }
    }
}

/// Handles requests from other processes while this one is the primary.
fn watch_inbox(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            let messages = {
                let state = app.state::<AppState>();
                let lock = state.primary_lock.lock().expect("poisoned");
                if lock.is_none() {
                    // We are shutting down: leave requests to the next primary.
                    Vec::new()
                } else {
                    state.instance.drain()
                }
            };
            for m in messages {
                dispatch(&app, m);
            }
            std::thread::sleep(INBOX_POLL);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = match AppPaths::resolve() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ordo: {e}");
            std::process::exit(1);
        }
    };
    let instance = InstanceDir::new(&paths.data_dir);
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir().unwrap_or_default();
    let message = match sortmode::parse_sort_args(&args, &cwd) {
        Some(paths) => Message::Sort { paths },
        None => Message::Show,
    };

    // Hand the request over first, then find out whether we are the primary.
    if let Err(e) = instance.send(&message) {
        eprintln!("ordo: cannot queue the request: {e}");
        std::process::exit(1);
    }
    let lock = match instance.try_acquire() {
        Ok(Some(lock)) => lock,
        // Another instance is running and will pick up the request.
        Ok(None) => return,
        Err(e) => {
            eprintln!("ordo: cannot check for a running instance: {e}");
            std::process::exit(1);
        }
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(AppState {
                paths,
                broker: Default::default(),
                sort_queue: Default::default(),
                instance,
                primary_lock: Mutex::new(Some(lock)),
            });
            app.manage(windows::SummaryTimer::default());
            watch_inbox(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                // Never leave a worker blocked on a question nobody can answer.
                window.state::<AppState>().broker.cancel_for(window.label());
                let app = window.app_handle().clone();
                // Let Tauri drop the window first, then check whether we're done.
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(200));
                    exit_if_idle(&app);
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::system_lang,
            commands::get_settings,
            commands::save_settings,
            commands::get_rules,
            commands::validate_rule,
            commands::save_rules,
            commands::import_rules,
            commands::export_rules,
            commands::default_rules,
            commands::list_batches,
            commands::get_batch,
            commands::clear_history,
            commands::undo_batch,
            commands::undo_move,
            commands::pending_conflicts,
            commands::resolve_conflict,
            commands::open_main,
            commands::reveal_path,
            commands::summary_keep_open,
            commands::file_thumbnail,
            commands::preview_file,
            commands::integration_status,
            commands::integration_install,
            commands::integration_uninstall,
            commands::command_status,
            commands::command_install,
            commands::command_uninstall,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Ordo");

    app.run(|app, event| match event {
        // Keep running while a sort is pending, even without windows.
        RunEvent::ExitRequested {
            code: None, api, ..
        } if app.state::<AppState>().sort_queue.is_busy() => {
            api.prevent_exit();
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } => {
            let _ = windows::show_main(app, None);
        }
        _ => {}
    });
}
