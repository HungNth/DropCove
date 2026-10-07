# 01: Clear Temporary Items from the Edge Rail

**Parent specification:** [Compact Adaptive Edge Rail](../spec.md)

**What to build:** Add a behaviorally complete Clear Temporary Items action to the existing Edge Rail so users can remove temporary references without reopening the Drop Shelf. This ticket owns the command integration, confirmation, success refresh, empty-state transition, and failure reporting on the current rail shell; Ticket 02 owns the later final placement and visual recomposition.

**Blocked by:** None (can start immediately).

**Status:** needs-info

**Testing seam:** Existing `DropShelfManager` lifecycle interface with real temporary SQLite; actual isolated-profile Edge Rail for confirmation, refresh/hide and notification.

**Demo path:** Dock mixed content, expand the rail, cancel Clear, confirm Clear and inspect retained pinned content; repeat with all-Temporary content and an aborting SQLite trigger.

- [ ] The expanded Edge Rail exposes a bottom-left Clear Temporary Items control with the approved trash glyph, accessible name, tooltip, keyboard activation, and visible focus; the resting rail does not expose the control.
- [ ] The action uses the same confirmation title and message as the Drop Shelf, including the statement that source files on disk are not touched.
- [x] Canceling confirmation leaves every Shelf Batch and Shelf Item unchanged and performs no presentation refresh that could imply success.
- [x] Confirming the action calls the existing `DropShelfManager.ClearTemporaryItemsAsync` lifecycle and persistence seam; no rail-specific deletion implementation is added.
- [x] Every Temporary Item is removed across the shelf, every Pinned Item is retained, and any resulting empty Shelf Batch is pruned.
- [x] Clearing all remaining content transitions the shelf to Hidden, hides the Edge Rail immediately, and preserves remembered rail placement.
- [x] Clearing mixed content refreshes the surviving Edge Rail summaries and retains pinned indicators without reopening or activating the Drop Shelf.
- [x] `RemoveBatchAsync` remains a separate exact-batch action and is not called by the rail footer.
- [x] The confirmation title and message come from one shared application source so Drop Shelf and Edge Rail wording cannot drift.
- [ ] Expected persistence failure leaves current manager state and durable storage unchanged, skips success refresh, and reports an error through the existing resident notification path.
- [ ] The Drop Shelf's existing Clear Temporary Items behavior remains operational and uses the same manager semantics after the shared wording/failure handling is introduced.
- [ ] Manager and real temporary SQLite tests cover all-Temporary content, all-Pinned no-op, mixed content, multiple batches, empty-batch pruning, final-empty Hidden transition, restart persistence, and deterministic persistence failure.
- [ ] Installed Release smoke exercises confirmation cancel, successful mixed clear, successful all-Temporary clear, source-file preservation, rail refresh/hide, and visible failure reporting.
- [ ] Edge Rail geometry, Shelf Batch row layout, hover timing, no-activate behavior, drag behavior, flyouts, monitor placement, fullscreen policy, and scrolling remain unchanged in this ticket.
- [ ] No `ShelfCleanupCoordinator`, generic command bus, public deletion interface, compatibility wrapper, new dependency, telemetry, polling, or UI test framework is introduced.

## Comments

- Implementation adds shared confirmation wording, manager-owned rail Clear, success-only refresh, persistence-error reporting, and a confirmation/reentrancy guard holding the rail open.
- Release build: zero warnings/errors. Full suite passed 225/225; the clear-failure regression added afterward passes separately (1/1), preserving mixed references in memory and after reopening SQLite.
- Native Clear acceptance remains unobserved: Windows LockApp PID 8356 intercepts pointer input instead of the isolated application. Ticket remains open until unlocked-desktop qualification. No lock-screen bypass or normal-user process termination.

### Unlocked-desktop continuation

- Native Release now proves cancel/accept, mixed retention, all-Temporary hide, all-Pinned no-op, unchanged source fixtures, success refresh and failure state retention. The prior LockApp blocker is resolved.
- Smoke caught cleared rows remaining visible (five stale realized rows). Cache had aliased the manager's mutable batch list; caching the existing immutable summary projection fixes it without an additional array. Identical smoke fails before and passes after; see `artifacts/adaptive-rail-refresh-regression.json`.
- Final regression suite: 226 passed, zero failed/skipped. Release build/publish: zero warnings/errors. Notification presentation on persistence failure remains unobserved, so this acceptance is not marked complete.
