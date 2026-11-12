# 05: Group presentation surfaces into `surfaces/`

**Parent specification:** [Rust Module Layout — Feature Directories](../spec.md)

**What to build:** Move the remaining window/popup modules into a flat `surfaces/` directory, move their command wrappers out of `lib.rs`, and leave `lib.rs` as composition only. Move-only: no logic change.

**Blocked by:** 04

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** Dock to the Edge Rail, expand it, open Manage Items, run Clear Temporary with confirmation, open About, resize the Drop Shelf — all behave as before.

- [ ] `surfaces/mod.rs` only declares submodules; `edge_rail`, `manage_items`, `clear_confirmation`, `selection_confirmation`, `about`, `shelf_resize` move under their own names; `windows_auxiliary.rs` → `placement.rs`; `windows_actions.rs` → `item_actions.rs`; `windows_visuals.rs` → `shell_visuals.rs`.
- [ ] `surfaces/commands.rs` owns every surface command listed in the spec.
- [ ] `lib.rs` contains only module declarations, `run_blocking`, `fatal_startup_error`, and `run()` with `generate_handler!`; no `#[tauri::command]` remains in `lib.rs`.
- [ ] No `.rs` file other than `main.rs` and `lib.rs` remains directly under `src-tauri/src/`.
- [ ] IPC command names, arguments, payloads, events, and window labels are unchanged; no frontend file changes.
- [ ] Every `#[cfg(windows)]` gate on moved modules and commands is preserved.
- [ ] No `pub use` re-export or alias of an old path remains; no visibility wider than `pub(crate)` is introduced.
- [ ] Files are moved with `git mv`; no unrelated formatting changes.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
