# 03: Group Shelf state into `shelf/`

**Parent specification:** [Rust Module Layout — Feature Directories](../spec.md)

**What to build:** Move Shelf state, its path rules, and Managed File support into `shelf/`, and move Shelf command wrappers plus `publish_shelf` out of `lib.rs`. Move-only: no logic change.

**Blocked by:** 02

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** Drop files and a text snippet on the Drop Shelf, pin one, restart the app — the Shelf restores identically and the Managed File opens.

- [ ] `shelf.rs` → `shelf/mod.rs`; `windows_paths.rs` → `shelf/paths.rs`; `managed_files.rs` → `shelf/managed_files.rs`; `website_images.rs` → `shelf/website_images.rs`.
- [ ] `shelf/commands.rs` owns `publish_shelf` and every Shelf command listed in the spec; every former `crate::publish_shelf` caller uses the new path.
- [ ] The `include_bytes!` fixture path in Shelf tests resolves `src-tauri/src/fixtures/website-image.avif`.
- [ ] IPC command names, arguments, payloads, events, and window labels are unchanged; no frontend file changes.
- [ ] Every `#[cfg(windows)]` gate on moved modules and commands is preserved.
- [ ] No `pub use` re-export or alias of an old path remains; no visibility wider than `pub(crate)` is introduced.
- [ ] Files are moved with `git mv`; no unrelated formatting changes.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
