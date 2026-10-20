# 01: Exact Automatic Shelf Sizing

**What to build:** Make the Drop Shelf automatically fit its current responsive Shelf Batch rows in both directions whenever no Manual Height Override exists. The surface must use the approved `180/230/282/334` tiers, grow or shrink after width and content changes, preserve explicit user height while content remains, and retain the established final-empty reset.

**Parent specification:** [DropCove 0.0.16 — Drag Placement Feedback and Drop Shelf Fit](../spec.md)

**Testing seam:** Authoritative Shelf State behavior regressions supported by the actual Windows Resident Instance and real WebView2 Drop Shelf window bounds.

**Demo path:** Populate an automatically sized Drop Shelf through each responsive row tier, widen and narrow it across column boundaries, remove content through each supported lifecycle, then repeat with a Manual Height Override and final-empty removal.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] Zero through two responsive rows use `180` logical pixels; three use `230`; four use `282`; five or more use `334`; six or more rows scroll with the final Shelf Batch reachable.
- [x] Automatic height is derived from preferred width and authoritative retained Shelf Batch count rather than retained as a larger historical automatic height.
- [x] Width-only resizing without a Manual Height Override recalculates height in both directions when responsive column count changes.
- [x] Width-only resizing with an existing Manual Height Override preserves the chosen height.
- [x] A vertical or corner resize that actually changes height remains the only operation that creates or replaces a Manual Height Override.
- [x] Remove Batch immediately shrinks an automatically sized Drop Shelf when responsive row count decreases.
- [x] Removing a non-final Shelf Item from a surviving Shelf Batch leaves the automatic tier unchanged; removing that batch's final Shelf Item prunes the batch and may shrink the Drop Shelf when responsive row count decreases.
- [x] Clear Temporary Items recalculates automatic height from surviving Shelf Batches and preserves a Manual Height Override while content remains.
- [x] Successful Drag-Out consumption for Temporary Items and Shelf Batches recalculates automatic height; canceled, rejected, or failed drag-out leaves content and height unchanged.
- [x] Confirmed-Missing cleanup recalculates automatic height when it prunes content, while persistence failure retains the prior coherent Shelf and sizing state.
- [x] Reopen and restart derive the exact automatic tier from retained width and content instead of restoring stale excess height; manual height restores unchanged.
- [x] Automatic height changes keep the current top-left Drop Shelf origin and retain existing monitor work-area and Per-Monitor V2 DPI clamping.
- [x] Removing the final Shelf Item clears retained sizing intent and resets the next opening to `191 × 180`.
- [x] Duplicate-only and unsupported drops do not resize, reorder, scroll, or persist Shelf state.
- [x] Pin and Unpin operations that do not change Shelf Batch layout retain the same exact automatic tier.
- [x] Existing Edge Rail height tiers and no-live-shrink behavior remain unchanged.
- [x] Historical `236/292/348` and Drop Shelf no-live-shrink regression expectations are replaced with operation-level tests through the authoritative Shelf State interface.
- [x] The complete frontend production build and cargo-xwin-produced Windows regression executable pass; the produced native test binary is executed directly on Windows.
- [x] Actual WebView2 smoke observes exact Drop Shelf bounds, width-driven grow/shrink, content-driven shrink, Manual Height Override preservation, final reset, and absence of premature scrolling.
- [x] No persistence field, schema migration, sizing setting, compatibility layer, dependency, or second sizing invariant is added.

## Answer — observed evidence, 2026-10-20

- Red-before-green Shelf State regression reproduced the reported defect: the second accepted batch returned 236 instead of 180 logical pixels. The same operation-level regression passed after the shared sizing fix.
- Final Windows native suite: 113/113 passed. A pruning regression exercised Remove Batch, final/non-final Remove Item, Clear Temporary Items, item Copy, batch Copy, and confirmed-Missing cleanup in both automatic and manual-height modes. Existing rollback, cancellation, restart, and final-empty regressions passed.
- Actual WebView2 measured one-column row counts 1–6 at heights 180, 180, 230, 282, 334, 334. Three/four/five-row scroll content equaled viewport heights 160/212/264; the sixth row overflowed at the 334-pixel cap. Widths 348/516/347 produced heights 230/180/334 for six batches.
- Kept the specified 8-pixel top collection boundary and removed the redundant 4-pixel footer top padding. This resolves the measured 4-pixel premature overflow without changing 48-pixel cards or 24-pixel action targets.
- Exact installed candidate proved native accepted intake growing to 282, then immediate removal shrink to 230 and 180 with zero premature overflow and unchanged top-left origin (600, 200). Manual 400-pixel height survived content removal and width-only save; final-empty state returned 191 by 180.
- Physical high/mixed-DPI and actual Windows Contrast hardware were not exercised; deterministic native geometry and media emulation are not represented as physical coverage.
