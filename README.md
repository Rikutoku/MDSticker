# MDSticker

Markdown sticky notes for desktop. Open any `.md` file and it floats as a frameless, always-readable note that **live-updates the moment you save the file**.

![screenshot placeholder](screenshot.png)

---

## Almost everything is vibecoded
---

## To do
- [x] ~~Startup fix~~
- [x] ~~Color theme~~
- [ ] HTML sanitization
- [ ] Add video and screenshots to project description
- [ ] Quick edit functionality
- [ ] Opening note in external editor

---

## Features

- 📄 Open any Markdown file as a sticky note window
- ⚡ Live reload — content updates instantly when the file changes on disk
- 🎨 Per-window accent colour picker
- 📌 Position and size are remembered across restarts
- 🚀 Launches automatically with the OS when at least one note is open
- 🪟 Multiple notes supported — one window per file
- 🌐 Completely offline — no internet connection required

---

## Requirements

- [Rust](https://rustup.rs/) + [Tauri CLI v2](https://tauri.app)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11; installer bundled for Windows 10)
- `marked.umd.js` placed in `src/` before building (see below)

---

## Getting started

**1. Get `marked.umd.js`**

It (or it's alternative) can be found on:
- [JsDelivr](https://www.jsdelivr.com/package/npm/marked)
- [marked releases page](https://github.com/markedjs/marked/releases)

Place it at:
```
src/marked.umd.js
```

**2. Run in development**
```bash
cargo tauri dev
```

**3. Build for production**
```bash
cargo tauri build
```
The installer is output to `src-tauri/target/release/bundle/`.

---

## Stack

- [Tauri 2.0](https://tauri.app) — Rust backend + WebView2 frontend
- Vanilla JS / HTML / CSS — no frameworks
- [marked](https://marked.js.org) — Markdown parsing (bundled locally)
