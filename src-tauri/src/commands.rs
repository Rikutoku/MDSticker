use crate::settings::{Settings, WindowSettings};
use crate::watcher;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// open_file_dialog
// ═══════════════════════════════════════════════════════════════════════════════

/// Opens a native OS file-open dialog filtered to `.md` files.
/// Returns the chosen absolute path, or `None` if the user cancelled.
#[tauri::command]
pub async fn open_file_dialog(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use tokio::sync::oneshot;

    // The dialog plugin uses a callback API; bridge it to async with a oneshot.
    let (tx, rx) = oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Markdown Files", &["md"])
        .pick_file(move |path| {
            let _ = tx.send(path);
        });

    let result = rx.await.map_err(|e| e.to_string())?;
    // `FilePath::to_string()` gives the absolute path on desktop.
    Ok(result.map(|p| p.to_string()))
}

// ═══════════════════════════════════════════════════════════════════════════════
// read_file
// ═══════════════════════════════════════════════════════════════════════════════

/// Reads and returns the full text content of `path`.
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// open_note_window
// ═══════════════════════════════════════════════════════════════════════════════

/// Core implementation — called by the Tauri command **and** by the startup
/// setup in `lib.rs` so the logic is not duplicated.
pub async fn open_note_window_impl(app: &AppHandle, path: String) -> Result<String, String> {
    // ── Guard: reject empty files ────────────────────────────────────────────
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    if content.trim().is_empty() {
        return Err("empty_file".to_string());
    }

    let mut settings = Settings::load(app).unwrap_or_default();

    // ── Duplicate-path detection ─────────────────────────────────────────────
    // Case-insensitive on Windows (NTFS), case-sensitive elsewhere.
    #[cfg(windows)]
    let existing_id = {
        let lower = path.to_lowercase();
        settings
            .windows
            .iter()
            .find(|w| w.file_path.to_lowercase() == lower)
            .map(|w| w.id.clone())
    };
    #[cfg(not(windows))]
    let existing_id = settings
        .windows
        .iter()
        .find(|w| w.file_path == path)
        .map(|w| w.id.clone());

    if let Some(id) = existing_id {
        // Focus the already-open window and return its ID without creating a new one.
        if let Some(win) = app.get_webview_window(&id) {
            let _ = win.set_focus();
        }
        return Ok(id);
    }

    // ── Compute initial position ─────────────────────────────────────────────
    // Try to centre the default 320×480 window on the primary monitor;
    // fall back to (120, 80) if monitor info is unavailable.
    let (init_x, init_y): (f64, f64) = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| {
            let pos = m.position();
            let sz = m.size();
            (
                pos.x as f64 + sz.width as f64 / 2.0 - 160.0,
                pos.y as f64 + sz.height as f64 / 2.0 - 240.0,
            )
        })
        .unwrap_or((120.0, 80.0));

    // ── Build the window ─────────────────────────────────────────────────────
    let window_id = Uuid::new_v4().to_string();
    let filename = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Note")
        .to_string();

    let window = WebviewWindowBuilder::new(app, &window_id, WebviewUrl::App("index.html".into()))
        .title(&filename)
        .decorations(false) // frameless — the frontend draws its own chrome
        .fullscreen(false)
        .maximizable(false)
        .resizable(true)
        .min_inner_size(200.0, 200.0)
        .always_on_top(false)
        .transparent(true) // lets the frontend handle border-radius / shadow
        .inner_size(320.0, 480.0)
        .position(init_x, init_y)
        .build()
        .map_err(|e| e.to_string())?;

    // ── Intercept OS close (Alt+F4, taskbar right-click → Close) ────────────
    // We let the window close naturally (no prevent_default) and run our
    // settings / watcher cleanup in a spawned task.  The `win.close()` call
    // inside `close_note_window_impl` will silently no-op if the window is
    // already destroyed by then.
    {
        let app_c = app.clone();
        let wid_c = window_id.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let a = app_c.clone();
                let w = wid_c.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = close_note_window_impl(&a, w).await;
                });
            }
        });
    }

    // ── Persist settings ─────────────────────────────────────────────────────
    // Read the window's *actual* physical size and position from the OS after
    // creation.  `inner_size()` / `position()` on the builder work in logical
    // pixels, so on HiDPI displays the real physical dimensions differ from the
    // values we passed.  Saving the logical guesses (320/480) would cause the
    // restore path to use them as if they were physical, making the window grow
    // on every restart.  Reading back the truth from the OS keeps the stored
    // values in sync with what JS `outerSize()` / `outerPosition()` will see.
    let phys_size = window
        .outer_size()
        .unwrap_or(tauri::PhysicalSize { width: 320, height: 480 });
    let phys_pos = window
        .outer_position()
        .unwrap_or(tauri::PhysicalPosition { x: init_x as i32, y: init_y as i32 });

    let is_first = settings.windows.is_empty();
    settings.windows.push(WindowSettings {
        id: window_id.clone(),
        file_path: path.clone(),
        x: phys_pos.x,
        y: phys_pos.y,
        width: phys_size.width,
        height: phys_size.height,
        accent_color: "#4ade80".to_string(),
    });
    settings.save(app)?;

    // ── File watcher ─────────────────────────────────────────────────────────
    watcher::start_watcher(app.clone(), window_id.clone(), path);

    // ── Autostart ────────────────────────────────────────────────────────────
    // Enable OS autostart the moment the first note window is created.
    if is_first {
        use tauri_plugin_autostart::ManagerExt;
        let _ = app.autolaunch().enable();
    }

    Ok(window_id)
}

/// Tauri command wrapper around `open_note_window_impl`.
#[tauri::command]
pub async fn open_note_window(app: AppHandle, path: String) -> Result<String, String> {
    open_note_window_impl(&app, path).await
}

// ═══════════════════════════════════════════════════════════════════════════════
// close_note_window
// ═══════════════════════════════════════════════════════════════════════════════

/// Core implementation — called by the Tauri command **and** by the
/// `CloseRequested` event handler attached in `open_note_window_impl` /
/// `window_mgr::restore_single_window`.
pub async fn close_note_window_impl(app: &AppHandle, window_id: String) -> Result<(), String> {
    // Remove from persisted settings first so a crash after this point
    // doesn't leave a ghost entry.
    let mut settings = Settings::load(app).unwrap_or_default();
    settings.windows.retain(|w| w.id != window_id);
    settings.save(app)?;

    // Stop the directory watcher (drops it, freeing the OS handle).
    let ws = app.state::<watcher::WatcherState>();
    watcher::stop_watcher(&ws, &window_id);

    // Close the WebviewWindow.
    if let Some(win) = app.get_webview_window(&window_id) {
        let _ = win.close();
    }

    // When the last window is gone: disable autostart and exit the process.
    // ALLOW_EXIT must be set to true BEFORE app.exit(0) so that the
    // RunEvent::ExitRequested handler in lib.rs knows not to prevent this
    // intentional exit (see the ALLOW_EXIT docs in lib.rs).
    if settings.windows.is_empty() {
        use tauri_plugin_autostart::ManagerExt;
        let _ = app.autolaunch().disable();
        crate::ALLOW_EXIT.store(true, std::sync::atomic::Ordering::SeqCst);
        app.exit(0);
    }

    Ok(())
}

/// Tauri command wrapper around `close_note_window_impl`.
#[tauri::command]
pub async fn close_note_window(app: AppHandle, window_id: String) -> Result<(), String> {
    close_note_window_impl(&app, window_id).await
}

// ═══════════════════════════════════════════════════════════════════════════════
// update_window_state
// ═══════════════════════════════════════════════════════════════════════════════

/// Persists the window's current position and size (called on move/resize).
#[tauri::command]
pub async fn update_window_state(
    app: AppHandle,
    window_id: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let mut settings = Settings::load(&app).unwrap_or_default();
    if let Some(w) = settings.windows.iter_mut().find(|w| w.id == window_id) {
        w.x = x;
        w.y = y;
        w.width = width;
        w.height = height;
    }
    settings.save(&app)
}

// ═══════════════════════════════════════════════════════════════════════════════
// update_accent_color
// ═══════════════════════════════════════════════════════════════════════════════

/// Persists the accent colour chosen by the user for a specific window.
#[tauri::command]
pub async fn update_accent_color(
    app: AppHandle,
    window_id: String,
    color: String,
) -> Result<(), String> {
    let mut settings = Settings::load(&app).unwrap_or_default();
    if let Some(w) = settings.windows.iter_mut().find(|w| w.id == window_id) {
        w.accent_color = color;
    }
    settings.save(&app)
}

// ═══════════════════════════════════════════════════════════════════════════════
// get_window_settings
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns the full `WindowSettings` for a window.
/// The frontend calls this on `DOMContentLoaded` to obtain `file_path` and
/// `accent_color` for the current window.
#[tauri::command]
pub fn get_window_settings(app: AppHandle, window_id: String) -> Result<WindowSettings, String> {
    let settings = Settings::load(&app)?;
    settings
        .windows
        .into_iter()
        .find(|w| w.id == window_id)
        .ok_or_else(|| format!("No settings found for window '{window_id}'"))
}
