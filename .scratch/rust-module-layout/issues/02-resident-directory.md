# 02: Group Resident Instance into `resident/`

**Parent specification:** [Rust Module Layout — Feature Directories](../spec.md)

**What to build:** Move the Resident Instance coordinator and its global integrations into `resident/`, and move Rail/dismiss command wrappers out of `lib.rs`. Move-only: no logic change.

**Blocked by:** 01

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** Launch the built app twice: the second launch activates the existing Shelf; the global hotkey and Shake-to-open still show the Shelf; tray Exit closes cleanly.

- [ ] `resident.rs` → `resident/mod.rs`; `resident_tray.rs` → `resident/tray.rs`; `windows_instance.rs` → `resident/instance.rs`; `windows_startup.rs` → `resident/startup.rs`; `windows_foreground.rs` → `resident/foreground.rs`; `windows_hotkey.rs` → `resident/hotkey.rs`; `windows_shake.rs` → `resident/shake.rs`.
- [ ] `resident/commands.rs` owns every Resident command listed in the spec; `lib.rs` registers them through their new paths.
- [ ] All callers use the new `crate::resident::…` paths.
- [ ] IPC command names, arguments, payloads, events, and window labels are unchanged; no frontend file changes.
- [ ] Every `#[cfg(windows)]` gate on moved modules and commands is preserved.
- [ ] No `pub use` re-export or alias of an old path remains; no visibility wider than `pub(crate)` is introduced.
- [ ] Files are moved with `git mv`; no unrelated formatting changes.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
