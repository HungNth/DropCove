# 04: Group drag and drop into `drag_drop/`

**Parent specification:** [Rust Module Layout — Feature Directories](../spec.md)

**What to build:** Move native drag-out, drop-target, and virtual-file intake into `drag_drop/`, and move drag command wrappers out of `lib.rs`. Move-only: no logic change.

**Blocked by:** 03

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** Drag a browser image onto the Shelf, then drag a batch and one Manage Items item out to Explorer — copies land as before.

- [ ] `drag_drop/mod.rs` only declares submodules; `windows_drag.rs` → `drag.rs`; `windows_drop.rs` → `drop.rs`; `windows_virtual_files.rs` → `virtual_files.rs`; `windows_virtual_files_tests.rs` → `virtual_files_tests.rs` with its `#[path]` attribute updated.
- [ ] `drag_drop/commands.rs` owns `storage_drag_on_window_thread`, `ItemDragResult`, `BatchDragResult`, and every drag command listed in the spec.
- [ ] The `include_bytes!` fixture path in the virtual-file tests resolves `src-tauri/src/fixtures/website-image.avif`.
- [ ] IPC command names, arguments, payloads, events, and window labels are unchanged; no frontend file changes.
- [ ] Every `#[cfg(windows)]` gate on moved modules and commands is preserved.
- [ ] No `pub use` re-export or alias of an old path remains; no visibility wider than `pub(crate)` is introduced.
- [ ] Files are moved with `git mv`; no unrelated formatting changes.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
