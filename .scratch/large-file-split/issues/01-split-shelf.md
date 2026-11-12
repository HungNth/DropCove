# 01: Split `shelf/mod.rs` into responsibility submodules

**Parent specification:** [Large File Split — Shelf and Resident](../spec.md)

**What to build:** Split Shelf production code into `store`, `snapshot`, `intake`, `drag`, `pinning`, `removal`, `managed_lifecycle`, and `recovery` submodules, and move its 91 tests into `shelf/tests/`. Move-only.

**Blocked by:** None — first ticket.

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** Open `shelf/` and find intake, removal, or recovery code by file name; each production submodule stays under roughly 550 lines.

- [ ] `shelf/mod.rs` keeps only shared projection types, `ShelfError`, the `ShelfState` struct, its store guard, and `close`.
- [ ] Each responsibility in the spec's `shelf/` table lives in its named submodule, including both the `ShelfState` entry points and the `ShelfStore` persistence they delegate to.
- [ ] `shelf/tests/mod.rs` holds `TestDirectory`; 91 tests are grouped into files mirroring the production submodules plus `sizing` and `item_actions`.
- [ ] The `include_bytes!` fixture path in moved tests resolves `src-tauri/src/fixtures/website-image.avif`.
- [ ] No file outside `shelf/` changes.
- [ ] No logic, SQLite schema, IPC, or error-message change beyond what this ticket names.
- [ ] Widened visibility stops at `pub(super)`; no `pub use` re-export or alias exists.
- [ ] Every `#[cfg(test)]` / `#[cfg(windows)]` gate is preserved.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
