# 01: Group Settings into `settings/`

**Parent specification:** [Rust Module Layout — Feature Directories](../spec.md)

**What to build:** Move Settings and Appearance Mode code into `settings/` and move every Settings-owned command wrapper out of `lib.rs`. Move-only: no logic change.

**Blocked by:** None — first ticket.

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** Launch the built app, open Settings from the tray, change Appearance Mode, Save, reopen — the change persists exactly as before.

- [ ] `settings.rs` → `settings/mod.rs`; `settings_store.rs` → `settings/store.rs`; `settings_transaction.rs` → `settings/transaction.rs`; `windows_theme.rs` → `settings/appearance.rs`.
- [ ] `settings/commands.rs` owns every Settings command listed in the spec's Command ownership table; `lib.rs` registers them through their new paths.
- [ ] All callers across the crate use `crate::settings::{store, transaction, appearance}` paths.
- [ ] IPC command names, arguments, payloads, events, and window labels are unchanged; no frontend file changes.
- [ ] Every `#[cfg(windows)]` gate on moved modules and commands is preserved.
- [ ] No `pub use` re-export or alias of an old path remains; no visibility wider than `pub(crate)` is introduced.
- [ ] Files are moved with `git mv`; no unrelated formatting changes.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
