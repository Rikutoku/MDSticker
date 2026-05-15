// MDSticker – frontend entry point
// Runs inside every note window (each window loads the same index.html but
// gets a different UUID label, which is used to look up its own settings).

window.addEventListener("DOMContentLoaded", async () => {
  // ── Tauri API references ──────────────────────────────────────────────────
  // `withGlobalTauri: true` in tauri.conf.json injects window.__TAURI__.
  const { invoke } = window.__TAURI__.core;
  const { getCurrentWindow } = window.__TAURI__.window;

  const win = getCurrentWindow(); // this window's WebviewWindow proxy
  const label = win.label; // UUID that identifies this window

  // ── Load persisted settings ───────────────────────────────────────────────
  let filePath, accentColor;
  try {
    const ws = await invoke("get_window_settings", { windowId: label });
    filePath = ws.file_path;
    accentColor = ws.accent_color;
  } catch (err) {
    console.error("[MDSticker] get_window_settings failed:", err);
    return; // Nothing useful to show — bail out.
  }

  // Apply stored accent colour immediately (before any repaint).
  document.documentElement.style.setProperty("--accent", accentColor);

  // Keep the colour-picker swatch in sync.
  document.getElementById("color-input").value = accentColor;

  // ── Render initial file content ───────────────────────────────────────────
  async function renderFile() {
    try {
      const raw = await invoke("read_file", { path: filePath });
      document.getElementById("content").innerHTML = marked.parse(raw);
    } catch (err) {
      console.error("[MDSticker] read_file failed:", err);
    }
  }
  await renderFile();

  // ── Live-update events from the Rust file watcher ────────────────────────

  // Rust emits "file-updated" with payload { content: String } to this window.
  await win.listen("file-updated", (event) => {
    document.getElementById("content").innerHTML = marked.parse(
      event.payload.content,
    );
  });

  // Rust emits "file-deleted" when the watched file disappears.
  await win.listen("file-deleted", () => {
    if (!document.querySelector(".deleted-banner")) {
      const banner = document.createElement("div");
      banner.className = "deleted-banner";
      banner.textContent = "⚠ The source file has been deleted or moved.";
      document.getElementById("content").prepend(banner);
    }
  });

  // ── Header auto-collapse ──────────────────────────────────────────────────
  // Hide the header when the window loses focus so it stays out of the way
  // while the user works in another app.  Restore it on focus.
  const header = document.getElementById("header");
  window.addEventListener("blur", () => header.classList.add("collapsed"));
  window.addEventListener("focus", () => header.classList.remove("collapsed"));

  // ── [+] Open a new note ───────────────────────────────────────────────────
  document.getElementById("btn-open").addEventListener("click", async () => {
    let chosenPath;
    try {
      chosenPath = await invoke("open_file_dialog");
    } catch (err) {
      console.error("[MDSticker] open_file_dialog error:", err);
      return;
    }
    if (!chosenPath) return; // user cancelled

    try {
      await invoke("open_note_window", { path: chosenPath });
    } catch (err) {
      if (err === "empty_file") {
        showToast("Cannot open an empty file.");
      } else {
        console.error("[MDSticker] open_note_window error:", err);
      }
    }
  });

  // ── [✕] Close this window ─────────────────────────────────────────────────
  document.getElementById("btn-close").addEventListener("click", async () => {
    try {
      await invoke("close_note_window", { windowId: label });
    } catch (err) {
      console.error("[MDSticker] close_note_window error:", err);
    }
  });

  // ── [···] Accent colour picker ────────────────────────────────────────────
  const colorInput = document.getElementById("color-input");

  document.getElementById("btn-color").addEventListener("click", (e) => {
    // Only open the picker when the button itself (not the input) is clicked,
    // to prevent double-firing on browsers that forward button clicks to
    // child inputs automatically.
    if (e.target !== colorInput) {
      colorInput.click();
    }
  });

  // Use 'input' for live preview while dragging the colour wheel, then persist.
  colorInput.addEventListener("input", async () => {
    const color = colorInput.value;
    document.documentElement.style.setProperty("--accent", color);
    try {
      await invoke("update_accent_color", { windowId: label, color });
    } catch (err) {
      console.error("[MDSticker] update_accent_color error:", err);
    }
  });

  // ── Persist window position / size ────────────────────────────────────────
  // Debounce at 300 ms so we don't hammer the settings file on every pixel
  // of a drag or resize.
  let debounceTimer = null;

  async function saveWindowState() {
    try {
      // outerPosition / outerSize give the physical (non-DPI-scaled) values.
      const pos = await win.outerPosition();
      const size = await win.outerSize();
      await invoke("update_window_state", {
        windowId: label,
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
      });
    } catch (err) {
      console.error("[MDSticker] update_window_state error:", err);
    }
  }

  function debouncedSave() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(saveWindowState, 300);
  }

  // tauri://move and tauri://resize are window-scoped events fired by Tauri
  // whenever the OS reports a position or size change.
  await win.listen("tauri://move", debouncedSave);
  await win.listen("tauri://resize", debouncedSave);
}); // end DOMContentLoaded

// ── Toast helper ─────────────────────────────────────────────────────────────
function showToast(message) {
  const toast = document.createElement("div");
  toast.className = "toast";
  toast.textContent = message;
  document.body.appendChild(toast);
  setTimeout(() => toast.remove(), 3000);
}
