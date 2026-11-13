//! Creation of the app's windows.
//!
//! - `main`: rules, history and settings.
//! - `conflicts`: conflict questions during a sort started from the file manager.
//! - `summary`: toast-like summary shown after such a sort, with "Undo".

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const MAIN: &str = "main";
pub const CONFLICTS: &str = "conflicts";
pub const SUMMARY: &str = "summary";

/// Event sent to an existing main window to open a batch in the history.
pub const OPEN_BATCH_EVENT: &str = "open-batch";

fn focus(w: &WebviewWindow) {
    let _ = w.unminimize();
    let _ = w.show();
    let _ = w.set_focus();
}

/// Shows the main window, creating it if needed; optionally on a batch of the history.
pub fn show_main(app: &AppHandle, batch: Option<i64>) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(MAIN) {
        focus(&w);
        if let Some(id) = batch {
            w.emit(OPEN_BATCH_EVENT, id)?;
        }
        return Ok(());
    }
    let mut route = match batch {
        Some(id) => format!("index.html#/history/{id}"),
        None => "index.html".to_string(),
    };
    // Development aid: open a given page, e.g. ORDO_DEV_ROUTE=settings.
    if cfg!(debug_assertions)
        && batch.is_none()
        && let Ok(dev) = std::env::var("ORDO_DEV_ROUTE")
    {
        route = format!("index.html#/{dev}");
    }
    let w = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App(route.into()))
        .title("Ordo")
        .inner_size(960.0, 640.0)
        .min_inner_size(720.0, 480.0)
        .center()
        .build()?;
    focus(&w);
    Ok(())
}

/// Shows the conflict window, creating it if needed.
pub fn show_conflicts(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(CONFLICTS) {
        focus(&w);
        return Ok(());
    }
    let w = WebviewWindowBuilder::new(
        app,
        CONFLICTS,
        WebviewUrl::App("index.html#/conflicts".into()),
    )
    .title("Ordo")
    .inner_size(660.0, 520.0)
    .resizable(false)
    .minimizable(false)
    .always_on_top(true)
    .center()
    .build()?;
    focus(&w);
    Ok(())
}

pub fn close(app: &AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.close();
    }
}

/// Shows the summary of a sort near the bottom-right corner of the screen.
pub fn show_summary(app: &AppHandle, batch: Option<i64>) -> tauri::Result<()> {
    close(app, SUMMARY);
    const WIDTH: f64 = 380.0;
    const HEIGHT: f64 = 168.0;
    const MARGIN: f64 = 24.0;
    let route = match batch {
        Some(id) => format!("index.html#/summary/{id}"),
        None => "index.html#/summary".to_string(),
    };
    let mut builder = WebviewWindowBuilder::new(app, SUMMARY, WebviewUrl::App(route.into()))
        .title("Ordo")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false);
    if let Some(monitor) = app.primary_monitor()? {
        let scale = monitor.scale_factor();
        let area = monitor.work_area();
        let x =
            (area.position.x as f64 / scale) + (area.size.width as f64 / scale) - WIDTH - MARGIN;
        let y =
            (area.position.y as f64 / scale) + (area.size.height as f64 / scale) - HEIGHT - MARGIN;
        builder = builder.position(x, y);
    }
    builder.build()?;
    app.state::<SummaryTimer>()
        .set(Some(Duration::from_secs(SUMMARY_SECONDS)));
    let app = app.clone();
    std::thread::spawn(move || summary_watchdog(&app));
    Ok(())
}

/// Seconds before the summary closes by itself.
pub const SUMMARY_SECONDS: u64 = 10;

/// When the summary window must close. Driven from Rust so that it works even
/// when the web view throttles its timers (e.g. hidden or locked screen).
#[derive(Default)]
pub struct SummaryTimer(Mutex<Option<Instant>>);

impl SummaryTimer {
    /// `Some(d)`: close after `d`; `None`: keep open until told otherwise.
    pub fn set(&self, after: Option<Duration>) {
        *self.0.lock().expect("poisoned") = after.map(|d| Instant::now() + d);
    }

    fn expired(&self) -> bool {
        self.0
            .lock()
            .expect("poisoned")
            .is_some_and(|deadline| Instant::now() >= deadline)
    }
}

fn summary_watchdog(app: &AppHandle) {
    loop {
        std::thread::sleep(Duration::from_millis(250));
        let Some(window) = app.get_webview_window(SUMMARY) else {
            return;
        };
        if app.state::<SummaryTimer>().expired() {
            let _ = window.close();
            return;
        }
    }
}
