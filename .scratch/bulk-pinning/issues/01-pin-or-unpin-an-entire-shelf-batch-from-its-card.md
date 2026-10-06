# 01: Pin or unpin an entire Shelf Batch from its card

**Parent specification:** [Bulk Pinning for Shelf Batches Specification](../spec.md)

**What to build:** Add one-click Bulk Pinning to every multi-item Shelf Batch card. The card must derive Off, On, or Mixed from its Shelf Items and atomically set all of those items to the Pinned or Temporary lifecycle state without creating batch-owned pin state. Persistence failure must leave memory and storage unchanged and report that nothing changed.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Testing seam:** Use the existing shelf manager with real temporary SQLite as the highest automated seam. Exercise deterministic persistence failure through SQLite itself rather than adding a persistence abstraction. Use an installed self-contained Release smoke for the real card control, accessibility state, status message, and stable window bounds.

**Demo path:** Create a multi-item Shelf Batch, use the card control to Pin All from Off, Unpin All from On, and Pin All from Mixed; restart between operations; then inject a persistence failure and show that every item and the visible card state remain unchanged.

- [x] Every multi-item Shelf Batch card exposes a `24 × 24` bulk pin toggle before Manage and Remove; single-item cards retain their existing binary Pin/Unpin action and no popup-only behavior is moved onto them.
- [x] The card derives Off when all Shelf Items are Temporary, On when all are Pinned, and Mixed when both lifecycle states are present; no aggregate pin state is added to Shelf Batch or persistence.
- [x] Activating Off or Mixed pins every Shelf Item in the target Shelf Batch, while activating On unpins every Shelf Item; no confirmation dialog appears.
- [x] Bulk Pinning includes Available, Unavailable, and confirmed-Missing references still present in the Shelf Batch without changing availability or removing references.
- [x] One manager mutation owns lookup, mutation serialization, persistence ordering, and in-memory replacement for card callers.
- [x] A real state change uses one atomic SQLite mutation keyed by Shelf Batch identity rather than one write per Shelf Item; no database schema change or batch pin column is introduced.
- [x] A missing Shelf Batch reports that it was not changed, while an already-satisfied target state succeeds without rewriting unchanged rows.
- [x] Persistence completes before in-memory replacement. Failure or cancellation before commit leaves every current and persisted Shelf Item in its previous lifecycle state, including a pre-existing Mixed state.
- [x] The card handler catches persistence failure, restores the manager-derived state, shows “Couldn’t update pinning. Nothing changed.” through the existing status surface, and does not retry automatically.
- [x] Successful Bulk Pinning updates the card immediately without a success toast or status message and survives application restart.
- [x] Individual Pin/Unpin, item removal, accepted drag-out, and restored persistence recompute the card aggregate instead of retaining stale Off, On, or Mixed state.
- [x] Successful drag-out semantics remain unchanged: participating Temporary Items are consumed, participating Pinned Items are retained, and cancellation, rejection, or failure retains valid references.
- [ ] The Mixed visual uses the pin icon plus a distinct dash or badge shape and remains distinguishable in light, dark, and high-contrast presentation without relying only on opacity.
- [x] Automation exposes Off, On, and Indeterminate accurately. Accessible action names are count-aware and item-centric, including the number currently pinned when Mixed; UI strings do not describe a Pinned Batch.
- [x] Bulk Pinning does not change Shelf Batch order, preferred shelf size, Manual Height Override, Automatic Shelf Growth, actual window bounds, or Edge Rail behavior.
- [x] Manager and real-SQLite tests cover Off → On, Mixed → On, On → Off, restart restoration, all availability classifications, missing/no-op targets, unchanged drag lifecycle, and deterministic transaction abort with unchanged memory and storage.
- [x] Installed Release smoke proves the card action, tri-state visual and automation state, count-aware naming, failure status, unchanged window bounds, and unchanged single-item card behavior.

## Comments

### 2026-10-06 — Card implementation and focused smoke

- TDD red: the atomic abort/manual-retry test failed compilation because the bulk manager API did not exist. Green: the same test passed after adding one serialized manager mutation and one atomic SQLite statement. No schema, package, or persistence abstraction added.
- `dotnet test tests/DropCove.Tests/DropCove.Tests.csproj -c Release --no-restore --filter FullyQualifiedName~DropShelfPersistenceTests`: 39 passed, 0 failed/skipped, including 14 new Bulk Pinning cases for transitions, restart/reference invariants, availability, no-op/missing targets, cancellation, atomic abort, and drag participation/lifecycle.
- Release build and side-by-side publish: 0 warnings/errors. The installed-directory executable uses an isolated test profile; the normal resident process and live profile were not replaced.
- [Card smoke evidence](../../../artifacts/bulk-pinning-card-smoke.json): actual UIA TogglePattern exercises Mixed → On, On → Off, Off → On, restart after pin and unpin, per-item normalization, deterministic abort with every persisted reference unchanged, outside-popup closure, and the single-item binary action. Off/On/Indeterminate and exact count-aware names were observed. Shelf bounds stayed `350 × 348`.
- [Mixed visual](../../../artifacts/bulk-pinning-card-mixed.png) and [failure visual](../../../artifacts/bulk-pinning-card-failure.png) are actual dark-mode screenshots. Light/high-contrast and integrated release qualification remain acceptance work in Ticket 03; no release-ready claim yet.

