# Rust Module Layout — Feature Directories

Status: ready-for-agent
Design confirmation: approved by the user through `grill-with-docs` on 2026-11-12 (Q1–Q13)
Testing seam: existing native regression suite — `cargo xwin build`, then `cargo xwin test --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --lib --no-run --locked`, then direct execution of the produced `dropcove_lib-*.exe`
Baseline: 198 passed, 0 failed (2026-11-12, before any move)
Release assignment: none — internal refactor, no version bump, no installer
Ticket decomposition: five sequential tickets `01 → 02 → 03 → 04 → 05`: [01 — settings/](issues/01-settings-directory.md), [02 — resident/](issues/02-resident-directory.md), [03 — shelf/](issues/03-shelf-directory.md), [04 — drag_drop/](issues/04-drag-drop-directory.md), [05 — surfaces/](issues/05-surfaces-directory.md)
Implementation authorization: separate user gate after ticket approval

## Problem Statement

All 29 native source files sit flat in `src-tauri/src/`. Fourteen carry a `windows_` prefix that distinguishes nothing on a Windows-only product. `lib.rs` is 1175 lines, of which roughly 950 are `#[tauri::command]` wrappers for every feature. Finding the code for one feature means scanning the whole directory and the whole `lib.rs`.

## Solution

Group modules into feature directories named after `GLOSSARY.md` concepts, move each feature's command wrappers next to the feature, and leave `lib.rs` as the composition root.

```text
src-tauri/src/
├── main.rs
├── lib.rs                   mod declarations, run(), generate_handler!, run_blocking, fatal_startup_error
├── fixtures/website-image.avif
├── resident/                mod.rs ← resident.rs; commands.rs; tray.rs ← resident_tray.rs;
│                            instance.rs ← windows_instance.rs; startup.rs ← windows_startup.rs;
│                            foreground.rs ← windows_foreground.rs; hotkey.rs ← windows_hotkey.rs;
│                            shake.rs ← windows_shake.rs
├── shelf/                   mod.rs ← shelf.rs; commands.rs; paths.rs ← windows_paths.rs;
│                            managed_files.rs; website_images.rs
├── drag_drop/               mod.rs (declarations only); commands.rs; drag.rs ← windows_drag.rs;
│                            drop.rs ← windows_drop.rs; virtual_files.rs ← windows_virtual_files.rs;
│                            virtual_files_tests.rs ← windows_virtual_files_tests.rs
├── surfaces/                mod.rs (declarations only); commands.rs; edge_rail.rs; manage_items.rs;
│                            clear_confirmation.rs; selection_confirmation.rs; about.rs; shelf_resize.rs;
│                            placement.rs ← windows_auxiliary.rs; item_actions.rs ← windows_actions.rs;
│                            shell_visuals.rs ← windows_visuals.rs
└── settings/                mod.rs ← settings.rs; commands.rs; store.rs ← settings_store.rs;
                             transaction.rs ← settings_transaction.rs; appearance.rs ← windows_theme.rs
```

### Command ownership

| File | Commands and helpers |
|---|---|
| `settings/commands.rs` | `open_settings`, `get_settings`, `get_appearance`, `set_appearance_mode`, `start_settings_recording`, `cancel_settings_recording`, `set_start_with_windows`, `record_settings_key`, `save_settings`, `cancel_settings`, `set_rail_placement`, `set_show_rail_over_fullscreen`, `set_shake` |
| `resident/commands.rs` | `dismiss_drop_shelf`, `get_rail`, `on_rail_body_pointer_enter`, `on_rail_button_pointer_enter`, `on_rail_pointer_leave`, `tick_rail`, `open_shelf_from_rail`, `set_rail_transient_hold` |
| `shelf/commands.rs` | `publish_shelf`, `get_shelf`, `acknowledge_shelf_warning`, `accept_drop`, `save_size`, `remove_batch`, `remove_item`, `clear_temporary`, `set_item_pinned`, `set_batch_pinned`, `set_batches_pinned`, `set_items_pinned`, `prepare_batches_removal`, `prepare_items_removal`, `remove_items`, `remove_batches`, `get_managed_text` |
| `drag_drop/commands.rs` | `storage_drag_on_window_thread`, `ItemDragResult`, `BatchDragResult`, `prepare_item_drag`, `cancel_item_drag`, `start_item_drag`, `prepare_batch_drag`, `cancel_batch_drag`, `start_batch_drag` |
| `surfaces/commands.rs` | `request_/confirm_/cancel_drop_shelf_clear_temporary`, `get_drop_shelf_clear_temporary_state`, `get_about_metadata`, `open_about`, `start_shelf_resize`, `get_shelf_resizing`, `prepare_shelf_move`, `open_manage_items`, `get_manage_items`, `get_manage_items_keyboard`, `get_manage_items_origin`, `close_manage_items`, `cancel_selection_removal`, `perform_item_action` |

Shell Visual commands (`get_shelf_visual`, `get_surface_generation`, `invalidate_surface_visuals`, `cancel_shelf_visual`) already live in their module and move with it.

## Decisions

1. Feature directories, not layers; Win32 code lives inside the feature that uses it.
2. Move-only: no logic change, no splitting of `shelf.rs`/`resident.rs`, no dependency untangling.
3. `#[tauri::command]` wrappers move to `<feature>/commands.rs`; `lib.rs` keeps only composition.
4. Directory modules use `mod.rs`.
5. The `windows_` prefix is dropped; names follow `GLOSSARY.md`.
6. Historical paths in `.scratch/**` and `docs/ROADMAP.md` stay unchanged.
7. `hotkey`, `shake`, `startup` belong to `resident/` (Resident Instance owns global integrations).
8. `windows_theme` becomes `settings/appearance.rs` (Appearance Mode is a Settings preference, ADR-0005).
9. `surfaces/` stays flat until a surface needs more than one file.
10. No `pub use` re-exports or aliases; every caller uses the new full path.
11. No release, no version bump.
12. Files move with `git mv`; no `cargo fmt` in this effort.
13. Five sequential tickets, one directory each; every ticket ends green.

## Invariants (every ticket)

- IPC command names, argument names, payload shapes, emitted event names, and window labels are unchanged; the frontend needs no edit.
- Every `#[cfg(windows)]` gate on a moved module or command is preserved.
- Visibility does not widen beyond what the new paths require (`pub(crate)` stays the ceiling).
- `include_bytes!` paths to `fixtures/website-image.avif` and the `#[path]` attribute for the virtual-file tests are updated to the new locations.
- `cargo xwin build` succeeds with no new warnings, and the native suite reports exactly 198 passed, 0 failed.
- No Git commit without explicit user approval.

## Out of Scope

- Splitting `shelf.rs` (6758 lines) or `resident.rs` (2966 lines) into submodules.
- Removing cross-feature calls (`publish_shelf` → surfaces/resident, `resident` ↔ `surfaces::edge_rail`).
- Formatting changes.
- Updating historical documentation paths.
