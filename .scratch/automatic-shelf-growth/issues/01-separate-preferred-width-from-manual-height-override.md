# 01: Separate preferred width from Manual Height Override

**Parent specification:** [Automatic Shelf Growth](../spec.md)

**What to build:** Preserve the user's horizontal and vertical sizing intent independently so later Automatic Shelf Growth can change transient height without weakening manual control. Width-only resizing must remain durable without creating or clearing a Manual Height Override, while a completed vertical or corner resize that changes height must establish the durable override. Cut over existing retained sizing data, empty-shelf pending choices, restart behavior, and final-item reset as one coherent user-visible lifecycle. Automatic Shelf Growth itself remains disabled in this ticket.

**Blocked by:** None (can start immediately).

**Status:** complete

- [x] The sizing state represents preferred logical width independently from an optional Manual Height Override; transient actual bounds and future automatic target height are not persisted as user choices.
- [x] Before Ticket 02 enables Automatic Shelf Growth, opening or restoring a shelf with preferred width but no Manual Height Override uses `180` logical pixels of height, subject to temporary current-monitor work-area clamping; this display height is not persisted as a manual choice.
- [x] A completed left/right resize that changes width but not height updates preferred width and neither creates nor clears the current Manual Height Override.
- [x] A completed top/bottom or corner resize creates or replaces Manual Height Override only when final logical height differs from resize-start logical height.
- [x] Programmatic positioning, restoration, DPI conversion, work-area clamping, and other non-user bounds changes cannot be classified as completed user resize.
- [x] Preferred width and Manual Height Override survive hide/show and application restart while at least one Shelf Item remains.
- [x] Every valid pre-cutover preferred-size record associated with retained Shelf Items preserves its stored width and height as a manual size; no legacy and canonical persistence paths remain side by side after cutover.
- [x] Resizing an empty visible shelf creates an in-memory pending width and optional height override; the first accepted Shelf Batch persists that choice atomically with its Shelf Items.
- [x] Restarting before an empty shelf accepts content discards the pending choice and opens the next empty shelf at `180 Ã— 180`.
- [x] Removing the final Shelf Item from the Drop Shelf or Edge Rail hides the shelf, deletes durable sizing state, resets preferred width and Manual Height Override, and preserves remembered Edge Rail placement.
- [x] Manager-seam, real temporary SQLite, and resize-completion tests cover axis semantics, cutover, empty-to-nonempty persistence, restart, failure atomicity, and final-item reset through externally observable behavior.
- [x] Existing manual resize, responsive grid, popup, drag, and Edge Rail behavior remains operational, and content changes still keep stable visible bounds until a later ticket enables Automatic Shelf Growth.

## Delivery evidence

- Canonical `ShelfSizingState` separates preferred width from nullable Manual Height Override; first acceptance persists the pending state, not a synthesized display height. Legacy sizing rows are transactionally cut over and the old size model/setter are removed.
- Real temporary SQLite suite: `25/25` passing. Resize-intent suite: `3/3` passing. Full automated suite after integration: `190/190` passing.
- Before enabling growth, isolated WinUI smoke observed a visible `180 × 180` shelf at `96 DPI`; the intermediate width-only opening height was explicit and transient.
- Installed pointer smoke later verified width-only resize to `350` leaves `manual_height = NULL`, vertical resize selects `240`, and corner resize selects `220 × 240`; later accepted drops and restart preserve manual height.
- Retained Ticket 01 Debug binary was exercised in a separate test profile after integration: three real accepted Explorer batches all held `180` height before growth was enabled. Evidence: `artifacts/automatic-growth-ticket01-stable-bounds.json`. Final Release acceptance remains blocked in Ticket 04.
