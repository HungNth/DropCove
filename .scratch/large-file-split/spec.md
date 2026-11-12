# Large File Split — Shelf and Resident

Status: ready-for-agent
Design confirmation: approved by the user through `grill-with-docs` on 2026-11-12 (Q1–Q9)
Testing seam: existing native regression suite — `cargo xwin build`, then `cargo xwin test --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --lib --no-run --locked`, then direct execution of the produced `dropcove_lib-*.exe`
Baseline: 198 passed, 0 failed (commit `9381142`)
Release assignment: none — internal refactor, no version bump, no installer
Ticket decomposition: three sequential tickets `01 → 02 → 03`: [01 — split shelf/](issues/01-split-shelf.md), [02 — split resident/](issues/02-split-resident.md), [03 — remove duplicated error plumbing](issues/03-remove-duplicated-error-plumbing.md)
Commit plan: tickets 01 + 02 form the move-only commit; ticket 03 is a separate commit
Implementation authorization: separate user gate after ticket approval

## Problem Statement

After the feature-directory refactor, two modules remain too large to navigate: `shelf/mod.rs` (6763 lines; 3086 production, 91 tests) and `resident/mod.rs` (2974 lines; 1779 production, 19 tests). Each mixes several unrelated responsibilities and keeps every test in one inline module. Both also repeat the same error plumbing: 32 hand-written mutex-poison mappings in `resident/mod.rs` and 26 hand-written `ShelfError::FileSystem` mappings in `shelf/mod.rs`.

## Solution

Split each module into responsibility-named submodules inside its existing directory, move tests into a `tests/` directory mirroring those submodules, and then replace duplicated error plumbing with one private helper per module.

### `shelf/`

| File | Responsibility |
|---|---|
| `mod.rs` | Shared projection types (`ShelfSnapshot`, `ShelfBatch`, `ShelfItem`, `DragOutcome`, …), `ShelfError`, `ShelfState` struct, store guard, `close` |
| `store.rs` | `ShelfStore` struct, database open and schema, `save_size`, small shared helpers (`random_id`, `automatic_height`, validation, token fields) |
| `snapshot.rs` | Read side: snapshot, Manage Items batch, Shell Visual target, item action resolution, Managed File text read |
| `intake.rs` | Path, text, and image intake into a new Shelf Batch |
| `drag.rs` | Batch and Item drag preparation, start, completion, occurrence consumption |
| `pinning.rs` | Item, batch, multi-batch, and multi-item pinning |
| `removal.rs` | Remove, Clear Temporary, removal preview and token, confirmed removal |
| `managed_lifecycle.rs` | Pending Managed File Deletion queueing, cleanup, startup reconciliation |
| `recovery.rs` | Database validation, SHM staging, Recovery Backup, `open_or_recover` |

`commands.rs`, `managed_files.rs`, `paths.rs`, `website_images.rs` are unchanged.

### `resident/`

| File | Responsibility |
|---|---|
| `mod.rs` | `Visibility`, `ResidentState` struct and visibility access, `now_monotonic_ms`, `exit`, `publish_warning`, `synchronize`, `repair_topmost` |
| `window_ops.rs` | `WorkArea`, `PhysicalPlacement`, `calculate_placement`, `WindowOps`, `NativeWindowOps` |
| `drop_shelf.rs` | Show, hide, dock, and dismiss the Drop Shelf |
| `rail.rs` | `RailPresentation`, `RailTransientReason`, expansion/collapse timing, Rail settings application, pointer classification |
| `fullscreen.rs` | Rail hiding and restoration under fullscreen coverage |
| `shake_session.rs` | Shake-to-open presentation session: begin, restore, promote on accepted drop |

`commands.rs`, `foreground.rs`, `hotkey.rs`, `instance.rs`, `shake.rs` (low-level gesture hook), `startup.rs`, `tray.rs` are unchanged.

### Tests

`shelf/tests/` and `resident/tests/` each contain `mod.rs` (shared fixtures: `TestDirectory`, `MockWindowOps`, `set_rail_presentation_for_test`) and one file per production submodule it primarily asserts. Cross-cutting tests live with their primary assertion. Shelf also gets `tests/sizing.rs` and `tests/item_actions.rs`; Resident gets `tests/lifecycle.rs` for exit and foreground behavior.

## Decisions

1. Scope is `shelf/mod.rs` and `resident/mod.rs` only.
2. No logic, SQLite schema, IPC, or error-message changes, except decision 9.
3. Tests move to `<module>/tests/`, grouped by the production submodule they assert.
4. Two phases: move-only first, duplication removal second; each phase is one commit and ends at 198 passed, 0 failed.
5. `shelf/` split as in the table above.
6. `resident/` split as in the table above; `shake_session.rs` (presentation state) stays separate from `shake.rs` (input hook).
7. Callers of moved `resident` items switch to the new submodule paths; no `pub use` re-exports or aliases. `publish_warning` stays in `resident/mod.rs`.
8. Test files mirror production submodule names; shared fixtures live in `tests/mod.rs`.
9. One `lock_presentation()` helper replaces all 32 mutex-poison mappings in `resident/`, unifying the two messages to `"resident presentation lock is poisoned"`.

## Invariants (every ticket)

- `ShelfState` and `ResidentState` remain single types; their `pub(crate)` methods keep their names and signatures.
- Fields and test-only hooks needed across submodules widen only to `pub(super)`, never beyond the owning module.
- Every `#[cfg(test)]` / `#[cfg(windows)]` gate is preserved.
- `cargo xwin build` succeeds with no new warnings, and the native suite reports exactly 198 passed, 0 failed.
- No Git commit without explicit user approval.

## Out of Scope

- Splitting `settings/store.rs`, `surfaces/edge_rail.rs`, or any other file.
- Removing cross-feature calls (`resident` ↔ `surfaces`, `shelf::commands::publish_shelf`).
- Formatting changes beyond the touched lines.
- Glossary or ADR changes.
