# 03: Recalculate automatic height across shelf lifecycles

**Parent specification:** [Automatic Shelf Growth](../spec.md)

**What to build:** Extend Automatic Shelf Growth beyond the immediate drop path so every Drop Shelf lifecycle respects the same hybrid contract. A shelf without a Manual Height Override must derive useful height when shown or restored, remain stable while visible content is removed, and reclaim unused height only on a later show or restart. Hidden and Edge Rail workflows must defer window geometry until the full shelf becomes visible.

**Blocked by:** 02: Grow the visible Drop Shelf by responsive rows.

**Status:** in-progress

- [x] Showing or restoring a non-empty Drop Shelf resolves preferred width and current-monitor clamping first, then derives automatic height from the responsive rows when no Manual Height Override exists.
- [x] A retained Manual Height Override restores its selected logical height, subject only to temporary DPI/work-area clamping, and continues to suppress Automatic Shelf Growth after hide/show and restart.
- [ ] Shelf Batches accepted while Hidden or EdgeDocked update content without moving the non-visible shelf; the next transition to the Drop Shelf calculates the correct height.
- [ ] Removing a Shelf Batch or Shelf Item, successful drag-out, Missing cleanup, Clear Temporary Items, pin changes, popup activity, and responsive reflow do not shrink or otherwise resize the shelf while content remains visible.
- [x] After visible removals, the next hide/show or restart begins a new visible lifetime and may open at a lower automatic tier when no Manual Height Override exists.
- [x] Width-only native resizing reflows or scrolls without changing height during or immediately after the resize, preserves any existing Manual Height Override state, and affects row calculation for the next accepted Shelf Batch or show.
- [x] Final-item removal continues to hide the shelf and reset sizing state; a subsequent workflow starts at the compact default and can use Automatic Shelf Growth.
- [x] Pre-cutover retained sizing records remain manual for their current content lifecycle and therefore do not begin Automatic Shelf Growth until final-item reset.
- [ ] Manager, persistence, sizing-policy, and native geometry tests cover show/restore, restart, Hidden/EdgeDocked deferral, grow-only visible lifetime, later recalculation, width-only reflow, manual restoration, and final-item reset.
- [ ] Installed self-contained Release smoke demonstrates no live shrink after removal, lower recalculated height after hide/show and restart, deferred Edge Rail content, exact manual restoration, width-only stability, and reachable monitor-aware opening.

## Implementation and verification evidence

- Show/restore computes the exact fitted monitor width before selecting the opening row tier. Display reflow preserves the visible automatic height instead of restoring `180`. Native sizing records the authoritative edge on each `WM_SIZING` message.
- Installed startup with five one-column batches observed `348`; removing four batches held `348`; later hide/show and restart recalculated `180`. A real vertical resize selected `240`, four later drops remained at `240`, and restart restored `240`. Evidence: `artifacts/automatic-growth-lifecycle.json`.
- Final installed artifact opened three batches at `350 × 236`, centered at `(785, 398)`; this verifies the reviewed direct-opening tier fix. Evidence: `artifacts/automatic-growth-final-launch.json`.
- Full automated suite passed `190/190`; Standards and Spec reviewers report no remaining code findings after width-preflight, sizing-direction and pure-geometry corrections.
- Source implementation is delivered. Full native Edge Rail content-deferral and remaining installed lifecycle acceptance are not claimed as observed; final acceptance remains blocked in Ticket 04.
