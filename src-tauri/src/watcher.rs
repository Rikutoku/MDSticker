use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

// ─── State ────────────────────────────────────────────────────────────────────

/// Tauri-managed state: one active `RecommendedWatcher` per note window,
/// keyed by the window's label (== the UUID window_id).
pub struct WatcherState(pub Mutex<HashMap<String, RecommendedWatcher>>);

impl Default for WatcherState {
    fn default() -> Self {
        WatcherState(Mutex::new(HashMap::new()))
    }
}

// ─── start_watcher ───────────────────────────────────────────────────────────

/// Starts a non-recursive directory watcher for the *parent* of `file_path`
/// and stores it in `WatcherState` under `window_id`.
///
/// # Why watch the parent directory?
///
/// Many editors (VS Code, Vim, Neovim, etc.) do **not** modify a file in-place.
/// Instead they write to a temporary sibling file and then atomically rename it
/// over the target.  On most platforms the `notify` crate cannot see a `Modify`
/// event on the original path during a rename-over — but it *does* see a
/// `Create` event on the parent directory when the new inode appears.
///
/// Watching the parent directory in `NonRecursive` mode and filtering events
/// by filename catches all three cases:
///   1. Direct in-place writes (Classic editors / `echo > file`)
///   2. Atomic rename-over (VS Code, Vim, Helix, …)
///   3. File recreation (delete + write)
pub fn start_watcher(app: tauri::AppHandle, window_id: String, file_path: String) {
    let path = PathBuf::from(&file_path);

    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => {
            eprintln!("[watcher] No parent directory for: {file_path}");
            return;
        }
    };

    let target_name = match path.file_name() {
        Some(n) => n.to_os_string(),
        None => {
            eprintln!("[watcher] Cannot extract filename from: {file_path}");
            return;
        }
    };

    // Clones needed to move into the closure (the closure lives on a notify thread).
    let app_c = app.clone();
    let wid_c = window_id.clone();
    let path_c = path.clone();

    let callback = move |res: notify::Result<notify::Event>| {
        let event = match res {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[watcher] Watch error: {e}");
                return;
            }
        };

        // Filter: only act when the event involves our specific file.
        let is_our_file = event
            .paths
            .iter()
            .any(|p| p.file_name() == Some(&target_name));
        if !is_our_file {
            return;
        }

        match event.kind {
            notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                match std::fs::read_to_string(&path_c) {
                    Ok(content) => {
                        if let Some(win) = app_c.get_webview_window(&wid_c) {
                            let _ =
                                win.emit("file-updated", serde_json::json!({ "content": content }));
                        }
                    }
                    Err(_) => {
                        // File was deleted / moved right after the event.
                        if let Some(win) = app_c.get_webview_window(&wid_c) {
                            let _ = win.emit("file-deleted", ());
                        }
                    }
                }
            }
            _ => {} // Ignore Access, Remove, Other, etc.
        }
    };

    match notify::recommended_watcher(callback) {
        Ok(mut watcher) => {
            if let Err(e) = watcher.watch(&parent, RecursiveMode::NonRecursive) {
                eprintln!("[watcher] Failed to watch {parent:?}: {e}");
                return;
            }
            let state = app.state::<WatcherState>();
            if let Ok(mut map) = state.0.lock() {
                map.insert(window_id, watcher);
            }; // semicolon ensures the MutexGuard temporary is dropped before `state`
        }
        Err(e) => eprintln!("[watcher] Failed to create watcher: {e}"),
    }
}

// ─── stop_watcher ─────────────────────────────────────────────────────────────

/// Removes the watcher for `window_id` from the map, which drops it and stops
/// the underlying OS watch handle.
pub fn stop_watcher(state: &WatcherState, window_id: &str) {
    if let Ok(mut map) = state.0.lock() {
        map.remove(window_id);
    }
}
