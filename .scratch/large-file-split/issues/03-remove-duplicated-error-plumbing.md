# 03: Remove duplicated error plumbing

**Parent specification:** [Large File Split — Shelf and Resident](../spec.md)

**What to build:** Replace repeated mutex-poison and filesystem error mappings with one private helper per module.

**Blocked by:** 02

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** `grep "lock is poisoned" src-tauri/src/resident` returns only the helper; `grep "ShelfError::FileSystem {" src-tauri/src/shelf` returns only the helper and the error definition.

- [ ] `ResidentState::lock_presentation()` replaces all 32 mutex-poison mappings; both former messages become `"resident presentation lock is poisoned"`.
- [ ] One private Shelf helper replaces the 26 hand-written `ShelfError::FileSystem { path, source }` mappings while preserving each reported `path` value.
- [ ] No other behavior or message changes.
- [ ] This ticket is its own commit, separate from tickets 01 and 02.
- [ ] No logic, SQLite schema, IPC, or error-message change beyond what this ticket names.
- [ ] Widened visibility stops at `pub(super)`; no `pub use` re-export or alias exists.
- [ ] Every `#[cfg(test)]` / `#[cfg(windows)]` gate is preserved.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
