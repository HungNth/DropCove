# 02: Grow the visible Drop Shelf by responsive rows

**Parent specification:** [Automatic Shelf Growth](../spec.md)

**What to build:** Make an accepted Shelf Batch grow the visible Drop Shelf only when the responsive grid needs another row and no Manual Height Override applies. Growth must use the approved bounded height tiers, preserve width, remain top-anchored and reachable, keep the newest batch visible, and scroll after four rows without restoring Compact or Expanded modes.

**Blocked by:** 01: Separate preferred width from Manual Height Override.

**Status:** in-progress

- [x] One decision-rich sizing policy is the single source for responsive column count, Shelf Batch row count, the `180/236/292/348` logical-height tiers, the four-row cap, and grow-versus-hold decisions.
- [x] Row count uses the rendered grid's current actual width and responsive column boundaries; Shelf Item count inside a Shelf Batch does not affect the result.
- [x] At one-column width, accepted Shelf Batches produce heights `180`, `236`, `292`, and `348` for one through four rows; later rows remain at `348` and scroll vertically.
- [x] At wider widths, a Shelf Batch that fills free capacity in an existing row leaves window height unchanged, while the first batch requiring a new row advances to the next tier.
- [x] Automatic growth changes height only, keeps the top edge fixed when space permits, shifts upward only as required for work-area reachability, and clamps actual bounds without changing durable sizing state.
- [x] A Manual Height Override suppresses automatic growth; new Shelf Batches instead reflow or scroll inside the user-selected height.
- [ ] Rejected or unsupported drops do not resize the shelf, and content accepted while the shelf is Hidden or EdgeDocked does not move a non-visible window.
- [x] Programmatic growth never persists automatic height, creates a Manual Height Override, or overwrites preferred width.
- [ ] An open multi-item popup closes before bounds change, and accepted drops leave the batch viewport at the newest-first position.
- [ ] Automated policy and geometry tests cover exact column boundaries, row/tier mapping, cap and overflow, same-row additions, manual suppression, top anchoring, work-area clamp, DPI scaling, and rejected drops without duplicating constants across modules.
- [ ] Installed self-contained Release smoke with real Explorer/Desktop drops demonstrates one-column tier progression, wider multi-column capacity, bounded scrolling, stable width, popup closure, and top-anchored reachable growth.

## Implementation and verification evidence

- Row-aware growth, accepted-drop filtering, manual suppression, width preservation, top anchoring and work-area correction are implemented. Pure sizing-policy suite passed `16/16`; anchored-growth geometry suite passed `4/4`; integrated automated suite passed `190/190`.
- Installed real Explorer one-column drops observed `180/236/292/348/348` for batches 1–5 at fixed width/top. Native width-only resize to `350` preserved automatic-height eligibility; at that width batches 1–3 observed `180/180/236`. Evidence: `artifacts/automatic-growth-installed-drops.json`.
- Actual bottom-edge scenario grew from `236` to `292`, moved top from `796` to `740`, and retained bottom at the work-area boundary `1032`. Evidence: `artifacts/automatic-growth-ui-smoke.json`.
- Multi-item popup keyboard traversal and post-growth shelf keyboard focus were observed. The Explorer-source interaction can itself dismiss the popup, so growth-only closure causality is not separately claimed. Exact rejected-drop native geometry and complete popup/viewport acceptance remain pending rather than inferred as passing.
- Source implementation is delivered; the unchecked acceptance items remain open, and final installed Release acceptance is blocked in Ticket 04.
