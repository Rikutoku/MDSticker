use crate::settings::{Settings, WindowSettings};
use crate::watcher;
use tauri::{AppHandle, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

/// Re-opens every window stored in `settings` — called on app startup when
/// there are saved windows from a previous session.
pub fn restore_windows(app: &AppHandle, settings: &Settings) {
    for win in &settings.windows {
        restore_single_window(app, win);
    }
}

fn restore_single_window(app: &AppHandle, win: &WindowSettings) {
    let filename = std::path::Path::new(&win.file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Note")
        .to_string();

    // Build the window INVISIBLE so there is no visual flash when we apply the
    // correct physical-pixel size and position below.  The builder's
    // `inner_size()` and `position()` methods work in *logical* pixels, which
    // diverges from our stored *physical* pixel values on HiDPI displays —
    // that mismatch was causing the window to grow a little on every restart.
    // Applying the values through the physical-pixel API after creation is the
    // only reliable way to guarantee exact restoration regardless of DPI.
    let result = WebviewWindowBuilder::new(app, &win.id, WebviewUrl::App("index.html".into()))
        .title(&filename)
        .decorations(false)
        .fullscreen(false)
        .maximizable(false)
        .resizable(true)
        .min_inner_size(200.0, 200.0)
        .always_on_top(false)
        .transparent(true)
        .visible(false) // show only after size/position are set correctly
        .build();

    match result {
        Ok(window) => {
            // Apply exact physical-pixel dimensions saved by the JS frontend.
            let _ = window.set_size(PhysicalSize::new(win.width, win.height));
            let _ = window.set_position(PhysicalPosition::new(win.x, win.y));

            // Reveal the window now that it has the right geometry.
            let _ = window.show();

            // OS-close cleanup handler (Alt+F4, taskbar).
            let app_c = app.clone();
            let wid_c = win.id.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    let a = app_c.clone();
                    let w = wid_c.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::close_note_window_impl(&a, w).await;
                    });
                }
            });

            watcher::start_watcher(app.clone(), win.id.clone(), win.file_path.clone());
        }
        Err(e) => eprintln!("[window_mgr] Failed to restore '{}': {e}", win.id),
    }
}
