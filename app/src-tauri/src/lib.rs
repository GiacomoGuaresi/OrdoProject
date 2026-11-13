//! Ordo desktop app (Tauri).

mod commands;
mod conflicts;

use commands::AppState;
use ordo_core::AppPaths;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let paths = AppPaths::resolve()?;
            app.manage(AppState {
                paths,
                broker: Default::default(),
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // Never leave a worker blocked on a dialog that can't be answered.
                window.state::<AppState>().broker.cancel_all();
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
            commands::resolve_conflict,
            commands::reveal_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ordo");
}
