use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

// ─── Data types ───────────────────────────────────────────────────────────────

/// Persisted state for a single note window.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowSettings {
    pub id: String,
    pub file_path: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    /// Hex colour string, e.g. "#4ade80"
    pub accent_color: String,
}

/// Top-level settings.json schema.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub windows: Vec<WindowSettings>,
}

// ─── impl Settings ────────────────────────────────────────────────────────────

impl Settings {
    /// Load settings from `<app-data>/settings.json`.
    /// Returns `Settings::default()` when the file does not exist yet.
    pub fn load(app: &tauri::AppHandle) -> Result<Settings, String> {
        let path = settings_path(app)?;
        if !path.exists() {
            return Ok(Settings::default());
        }
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// Atomically save settings to `<app-data>/settings.json`.
    ///
    /// # Why atomic?
    /// Writing directly to the target file risks leaving a partial/corrupt file
    /// if the process is killed mid-write.  Instead we write to a `.tmp` sibling
    /// and then `rename` it over the real file — the rename is atomic on all
    /// major operating systems so readers always see either the old or the new
    /// complete file, never a half-written one.
    pub fn save(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let path = settings_path(app)?;
        let tmp = path.with_extension("tmp");

        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, &json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Returns the path `<app-data-dir>/settings.json`, creating the directory if
/// it does not exist yet.
fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}
