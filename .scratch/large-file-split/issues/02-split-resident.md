# 02: Split `resident/mod.rs` and update callers

**Parent specification:** [Large File Split — Shelf and Resident](../spec.md)

**What to build:** Split Resident production code into `window_ops`, `drop_shelf`, `rail`, `fullscreen`, and `shake_session` submodules, move its 19 tests into `resident/tests/`, and update every caller to the new paths. Move-only apart from path updates.

**Blocked by:** 01

**Status:** ready-for-agent

**Testing seam:** Native build plus the existing native regression suite run directly from the produced `dropcove_lib-*.exe` (see spec Testing seam).

**Demo path:** Open `resident/` and find Edge Rail timing in `rail.rs` and Shake-to-open presentation in `shake_session.rs`; the largest production submodule stays under roughly 600 lines.

- [ ] `resident/mod.rs` keeps `Visibility`, the `ResidentState` struct and visibility access, `now_monotonic_ms`, `exit`, `publish_warning`, `synchronize`, and `repair_topmost`.
- [ ] Each responsibility in the spec's `resident/` table lives in its named submodule.
- [ ] Every caller across the crate uses the new submodule path for moved items (for example `resident::drop_shelf::show_shelf`, `resident::rail::RailTransientReason`).
- [ ] `resident/tests/mod.rs` holds `MockWindowOps` and `set_rail_presentation_for_test`; 19 tests are grouped into `drop_shelf`, `rail`, `fullscreen`, `shake_session`, and `lifecycle`.
- [ ] Tickets 01 and 02 together form one move-only commit.
- [ ] No logic, SQLite schema, IPC, or error-message change beyond what this ticket names.
- [ ] Widened visibility stops at `pub(super)`; no `pub use` re-export or alias exists.
- [ ] Every `#[cfg(test)]` / `#[cfg(windows)]` gate is preserved.
- [ ] `cargo xwin build` succeeds with no new warnings.
- [ ] Native suite: exactly 198 passed, 0 failed.
