<div align="center">

<img src="src-tauri/icons/128x128.png" alt="DropCove Logo" width="96" height="96" />

# DropCove

**A fast, persistent drag-and-drop shelf for Windows.**

Temporary holding area for files, images, and text snippets while you work across windows and desktops.

[![Platform](https://img.shields.io/badge/platform-Windows%2011%20(x64)-blue.svg)](#requirements)
[![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)](LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%20v2-24C8DB.svg)](https://tauri.app/)

</div>

---

DropCove stays resident in your notification area and provides a quick workspace when collecting files from different locations before moving them to their destination. Instead of juggling side-by-side File Explorer windows or cluttering your Desktop, drop items onto DropCove, switch contexts, and drag them out when you are ready.

> [!NOTE]
> DropCove is strictly built for **Windows 11 (x64)**. It is a temporary holding shelf — not a file manager, clipboard history, or cloud sync tool. It never alters, moves, or deletes original external files.

---

## Features

### 🗂️ Drag-In from Anywhere
- **Files & Folders**: Drop any local or network paths. Grouped automatically into ordered batches.
- **Website Images**: Drag images straight from web browsers; DropCove turns them into Managed Files ready for filesystem drag-out.
- **Text Snippets**: Drop highlighted text from documents or browsers to generate self-contained text files instantly.
- **Shake to Open**: Wiggle your cursor while dragging files to pop open the shelf automatically.

### 🪟 Floating Drop Shelf & Edge Rail
- **Drop Shelf**: A floating, resizable holding area that stays top-most when needed.
- **Edge Rail**: When dismissed, resting batches tuck neatly into a subtle strip along the screen edge without grabbing foreground focus.
- **Automatic Sizing**: Automatically adapts height to your content while respecting manual resizing preferences.

### 🎯 Multi-Select & Batch Drag-Out
- **Single & Multi-Batch Drag**: Drag an entire batch or multi-select (`Ctrl`+click, `Shift`+click) multiple batches to drag out at once.
- **Manage Items Inspection**: Expand multi-item batches in a compact grid popup to view, multi-select, or drag out individual items.
- **Occurrence-Safe & Mixed Availability**: Safely skips missing items while preserving temporary items if drag-and-out fails or is canceled.

### 📌 Pinned & Temporary Lifecycles
- **Temporary Items (Default)**: Items are automatically cleaned up as soon as they are successfully dragged and copied into their target destination.
- **Pin Items**: Mark essential items or frequently used templates to retain them indefinitely on the shelf.
- **Privacy & Safety**: Context menu actions let you inspect, reveal in File Explorer, copy paths, pin, or delete managed files without exposing raw storage internals.

### ⚙️ Resident & Persistent
- **Persistent Storage**: Saved in a local SQLite database (`shelf.db`) surviving application restarts and system reboots.
- **Hotkeys & Tray**: Summon or dismiss the shelf anytime with configurable global hotkeys or the system tray icon.
- **Native Appearance**: Fully supports System, Dark, and Light appearance modes as well as Windows High Contrast themes.

---

## Requirements

- **Operating System**: Windows 11, build 22000 or later (x64)
- **Runtime**: [Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe)
- **Node.js**: `>= 22` (for building from source)
- **Rust Toolchain**: `cargo-xwin` targeting `x86_64-pc-windows-msvc`

---

## Development

DropCove is built using [Tauri v2](https://tauri.app/), [Rust](https://www.rust-lang.org/), [Vue 3](https://vuejs.org/), [TypeScript](https://www.typescriptlang.org/), and [Tailwind CSS v4](https://tailwindcss.com/).

### Prerequisites

Install dependencies using `pnpm`:

```bash
pnpm install
```

Ensure `cargo-xwin` is installed:

```bash
cargo install cargo-xwin
```

### Common Commands

| Command | Description |
| --- | --- |
| `make dev` | Start the development server and launch the desktop app |
| `make build` | Build frontend and compile the NSIS installer via `cargo-xwin` |
| `pnpm test` | Run frontend test suite (`node --test tests/*.test.mjs`) |
| `make sync-version` | Synchronize `VERSION` across `package.json`, `Cargo.toml`, and `tauri.conf.json` |
| `make clean` | Clean up `dist/`, `build/`, and cargo target directories |
