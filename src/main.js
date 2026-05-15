// MDSticker – frontend entry point
// Runs inside every note window (each window loads the same index.html but
// gets a different UUID label, which is used to look up its own settings).

// ── Accent colour table ───────────────────────────────────────────────────────
// Each entry is one colour family.  Index 0-5 matches the six .color-swatch
// buttons in order.  Two shades per family: muted for light mode, richer for
// dark mode.  Both values are recognised when loading a stored accent so the
// app can migrate between themes automatically.
const ACCENT_PAIRS = [
  { dark: "#2d5e5c", light: "#528c8b" }, // Teal
  { dark: "#8c6f36", light: "#bd9b5b" }, // Gold
  { dark: "#4d5e3c", light: "#828c61" }, // Green
  { dark: "#3a5675", light: "#6382a1" }, // Blue
  { dark: "#6b5687", light: "#9c89b8" }, // Purple
  { dark: "#8a4336", light: "#b56a5c" }, // Red
];

const darkMQ = window.matchMedia("(prefers-color-scheme: dark)");
const isDark = () => darkMQ.matches;

// Return the theme-correct hex for a family index.
const familyColor = (i) =>
  isDark() ? ACCENT_PAIRS[i].dark : ACCENT_PAIRS[i].light;

// Return the family index (0-5) that contains `color` (either shade), or -1.
const findFamily = (color) =>
  ACCENT_PAIRS.findIndex((p) => p.light === color || p.dark === color);

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

  // ── Swatch initialisation ──────────────────────────────────────────────
  const allSwatches = document.querySelectorAll(".color-swatch");

  // Paint every swatch with the colour that matches the current OS theme.
  function updateSwatchColors() {
    allSwatches.forEach((s, i) => {
      s.style.background = isDark()
        ? ACCENT_PAIRS[i].dark
        : ACCENT_PAIRS[i].light;
    });
  }
  updateSwatchColors();

  // If the stored accent belongs to a known family, migrate it to the current
  // theme's shade (e.g. a dark-mode blue stored from a previous session will
  // automatically become the light-mode blue when the OS is in light mode).
  const familyIdx = findFamily(accentColor);
  if (familyIdx !== -1) {
    accentColor = familyColor(familyIdx);
    // Persist quietly — don't await so startup isn't delayed.
    invoke("update_accent_color", {
      windowId: label,
      color: accentColor,
    }).catch(() => {});
  }

  // Apply the (possibly migrated) accent and mark the matching swatch active.
  document.documentElement.style.setProperty("--accent", accentColor);
  allSwatches.forEach((s, i) => s.classList.toggle("active", i === familyIdx));

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

  // ── Accent colour palette ──────────────────────────────────────────────────
  const palette = document.getElementById("color-palette");

  document.getElementById("btn-color").addEventListener("click", (e) => {
    e.stopPropagation();
    palette.hidden = !palette.hidden;
  });

  // Each swatch click picks the theme-appropriate shade for that family.
  allSwatches.forEach((swatch, i) => {
    swatch.addEventListener("click", async () => {
      const color = familyColor(i);
      document.documentElement.style.setProperty("--accent", color);
      allSwatches.forEach((s) => s.classList.remove("active"));
      swatch.classList.add("active");
      palette.hidden = true;
      try {
        await invoke("update_accent_color", { windowId: label, color });
      } catch (err) {
        console.error("[MDSticker] update_accent_color error:", err);
      }
    });
  });

  // Close palette on outside click.
  document.addEventListener("click", () => {
    palette.hidden = true;
  });

  // ── OS theme change ────────────────────────────────────────────────────────
  // When the system switches light ↔ dark: repaint swatches and swap the
  // current accent to the same family's other shade, then persist.
  darkMQ.addEventListener("change", async () => {
    updateSwatchColors();
    const current = getComputedStyle(document.documentElement)
      .getPropertyValue("--accent")
      .trim();
    const fi = findFamily(current);
    if (fi === -1) return; // unknown / custom colour — leave it
    const newColor = familyColor(fi);
    document.documentElement.style.setProperty("--accent", newColor);
    allSwatches.forEach((s, i) => s.classList.toggle("active", i === fi));
    try {
      await invoke("update_accent_color", { windowId: label, color: newColor });
    } catch (err) {
      console.error("[MDSticker] theme-change accent error:", err);
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
