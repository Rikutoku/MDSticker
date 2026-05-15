mod commands;
mod settings;
mod watcher;
mod window_mgr;

use std::sync::atomic::{AtomicBool, Ordering};

/// Set to `true` immediately before any intentional `app.exit()` / `handle.exit()` call.
///
/// # Why this exists
/// Tauri fires `RunEvent::ExitRequested` in two distinct situations:
///   1. The last window is closed by the OS (we want to PREVENT this and handle
///      cleanup ourselves via `close_note_window_impl`).
///   2. Our own code calls `app.exit(0)` — which internally also fires the
///      same event before the process terminates.
///
/// Without a flag, the unconditional `api.prevent_exit()` in the run-loop
/// catches case 2 as well and the process never actually exits (stays in Task
/// Manager).  Flipping this bool to `true` right before every explicit exit
/// call tells the handler to step aside.
pub static ALLOW_EXIT: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ── Plugins ──────────────────────────────────────────────────────────
        // Single-instance guard: if the user launches a second copy of the app
        // (double-clicking the icon while it's already running, autostart race,
        // etc.) the second process exits immediately and this callback runs in
        // the FIRST (already-running) process so we can bring it to the front.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            use tauri::Manager;
            for (_, win) in app.webview_windows() {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            // On macOS we register a LaunchAgent; this argument is ignored
            // on Windows and Linux.
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        // ── Managed state ────────────────────────────────────────────────────
        .manage(watcher::WatcherState::default())
        // ── IPC commands ─────────────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            commands::open_file_dialog,
            commands::read_file,
            commands::open_note_window,
            commands::close_note_window,
            commands::update_window_state,
            commands::update_accent_color,
            commands::get_window_settings,
        ])
        // setup is intentionally empty — window creation happens in
        // RunEvent::Ready (see below) so that WebView2 is fully initialised
        // before we ask it to load anything.
        .setup(|_app| Ok(()))
        // ── Build + run ───────────────────────────────────────────────────────
        .build(tauri::generate_context!())
        .expect("error while building MDSticker")
        .run(|app_handle, event| {
            match event {
                // ── RunEvent::Ready ──────────────────────────────────────────
                // Fires once, after the Tauri event loop has fully started and
                // WebView2 is ready to create windows.  This is the correct
                // place to restore saved windows or show the initial dialog,
                // because on Windows autostart during system boot WebView2 may
                // not be initialised yet when `setup` runs — moving here
                // eliminates the "webpage not available" blank-window bug.
                tauri::RunEvent::Ready => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        startup(handle).await;
                    });
                }

                // ── RunEvent::ExitRequested ──────────────────────────────────
                // Fired both when the last window is closed by the OS AND when
                // our own code calls `app.exit()`.  We only want to prevent it
                // in the first case (to keep the process alive so our async
                // cleanup can run and eventually call `app.exit(0)` itself).
                // ALLOW_EXIT is flipped to `true` right before every deliberate
                // `app.exit()` call, so this handler steps aside then.
                tauri::RunEvent::ExitRequested { api, .. } => {
                    if !ALLOW_EXIT.load(Ordering::SeqCst) {
                        api.prevent_exit();
                    }
                }

                _ => {}
            }
        });
}

// ── Startup logic ─────────────────────────────────────────────────────────────

async fn startup(handle: tauri::AppHandle) {
    let saved = settings::Settings::load(&handle).unwrap_or_default();

    if saved.windows.is_empty() {
        // First launch (or all windows were previously closed cleanly):
        // show a file-open dialog.  If the user cancels, exit.
        use tauri_plugin_dialog::DialogExt;
        use tokio::sync::oneshot;

        let (tx, rx) = oneshot::channel();
        handle
            .dialog()
            .file()
            .add_filter("Markdown Files", &["md"])
            .pick_file(move |p| {
                let _ = tx.send(p);
            });

        match rx.await {
            Ok(Some(file_path)) => {
                if let Err(e) =
                    commands::open_note_window_impl(&handle, file_path.to_string()).await
                {
                    eprintln!("[startup] Could not open initial window: {e}");
                    ALLOW_EXIT.store(true, Ordering::SeqCst);
                    handle.exit(1);
                }
            }
            _ => {
                // User dismissed the dialog — nothing to show, exit cleanly.
                ALLOW_EXIT.store(true, Ordering::SeqCst);
                handle.exit(0);
            }
        }
    } else {
        // Restore every window saved from the previous session.
        window_mgr::restore_windows(&handle, &saved);
    }
}
