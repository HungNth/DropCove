# 03: Edge Rail pointer selection and lifecycle parity

**What to build:** Extend the complete Batch Selection and occurrence-safe drag workflow to the expanded Edge Rail without activating DropCove or stealing foreground focus. Preserve Rail-specific transient holds, aggregate confirmation, collapse timing, fullscreen policy, surface switching, and active native drag behavior while keeping selection local to the Rail.

**Blocked by:** 02: Occurrence-safe mixed batch-set drag core.

**Status:** resolved

**Parent specification:** [DropCove 0.0.25 — Multi-Batch Selection and Drag-Out](../spec.md)

**Testing seam:** Actual expanded Edge Rail on Windows while another application owns foreground focus, using real pointer modifiers, ordinary and virtualized collections, mixed external/Managed content, unavailable storage, fullscreen coverage, and real native Copy/cancel/reject destinations, supported by Resident presentation-state regressions.

**Demo path:** Keep another application focused, expand the Rail, form plain/Ctrl/Shift selections, copy a mixed selected set, cancel and retry an unavailable subset, switch to the Drop Shelf, repeat a rejected drag while fullscreen hiding occurs, and confirm no activation theft or selection restoration after a hide clear trigger.

- [x] The expanded Edge Rail owns an independent surface-local Batch Selection and selection anchor using stable Shelf Batch identities outside mounted virtual rows.
- [x] Plain click, `Ctrl+click`, `Shift+click`, no-valid-anchor fallback, and non-additive `Ctrl+Shift` match the approved Drop Shelf pointer model.
- [x] Pointer threshold arbitration preserves complete selected drag versus plain-click collapse on Rail cards.
- [x] Selected Rail cards use the same semantic accent treatment and Windows Contrast resources as Drop Shelf cards without changing compact card geometry.
- [x] The existing Rail footer budget shows `N batches · M items` while selection is non-empty without changing expansion height tiers or corner-action targets.
- [x] Pin/Unpin, Bulk Pinning, Manage Items, accepted drop-in, Remove Item, Remove Batch, and Missing pruning update selection by the same identity rules as the Drop Shelf.
- [x] The Rail remains pointer-only. It adds no focusable batch selection regions, roving focus, keyboard shortcuts, or live announcement channel.
- [x] Pointer selection and batch-set drag do not activate the Rail WebView, Drop Shelf, Resident Instance, or another DropCove surface.
- [x] Dragging a selected Rail card uses the complete local Batch Selection; dragging an unselected card uses a singleton set.
- [x] The full mixed availability, aggregate confirmation, stale-token invalidation, deduplicated payload, occurrence completion, and Managed File cleanup contract from Ticket 02 works through the Rail.
- [x] Rail batch-set preparation closes Rail-owned Manage Items and uses the existing BatchDragConfirmation transient hold.
- [x] Canceling or escaping the aggregate confirmation releases the hold, cancels the prepared token, retains selection, and preserves collapse timing.
- [x] Direct drag from the confirmation primary action and click-to-arm/re-drag both preserve no-activate behavior.
- [x] Rejected or canceled native drag retains selection when the Rail remained visible and no clear trigger occurred.
- [x] Accepted Copy clears Rail selection and refreshes retained Pinned/Unavailable content and counts.
- [x] Opening the Drop Shelf from the Rail clears Rail selection; the Drop Shelf opens with no transferred selection.
- [x] Dismissing the Drop Shelf back to EdgeDocked clears Drop Shelf selection; the Rail starts with no transferred selection.
- [x] Fullscreen hiding clears Rail Batch Selection, closes Manage Items, and discards unconfirmed prompts according to existing fullscreen policy.
- [x] An already-running native drag remains active when fullscreen hiding begins. Its recorded participants complete safely, but rejection or cancellation does not restore the cleared Rail selection.
- [x] Leaving fullscreen restores only the collapsed Rail Handle and does not resurrect selection, confirmation, or Manage Items.
- [x] The ActiveDrag transient hold continues to prevent premature collapse while the OLE operation runs and arms normal collapse timing afterward.
- [x] Rail expansion/collapse, hover timers, selected monitor placement, Left/Right mirroring, Shake behavior, incoming drop placeholder, and fullscreen visibility settings remain unchanged.
- [x] Collections above 50 batches preserve off-screen range selection, selected identities, footer counts, and native drag scope while Rail rows mount and unmount.
- [x] Resident presentation regressions cover selection clear on surface switch/fullscreen hide, ActiveDrag preservation, collapse timing, and no stale transient holds.
- [x] Frontend regressions cover Rail pointer gestures, action isolation, selected treatment/counts, confirmation lifecycle, and absence of focus/live-region behavior through observable DOM state.
- [x] A real Edge Rail smoke records the foreground window before and after plain/Ctrl/Shift selection, confirmation, Copy, cancel/reject, surface switch, and fullscreen hiding; DropCove never becomes foreground.
- [x] Version metadata remains `0.0.24` and no Git commit is created without separate explicit user authorization.

## Verification evidence — 2026-10-24

- Real expanded Rail pointer input selected non-adjacent batches while the owned destination retained foreground HWND `724200`; the Rail WebView kept `BODY` focused and exposed no polite live region. Footer showed `2 batches · 2 items`.
- Actual selected Rail native Copy consumed two Temporary batches (`58 → 56`), retained foreground focus in the destination, and used the shared occurrence-safe drag module.
- Surface switch, collapsed-Rail reset, and transient-close events clear the local selection without transferring it. Resident regression coverage for ActiveDrag hold, collapse timing, fullscreen transient dismissal, and drag preservation passed in the **179/179** native suite.
- Definitive fullscreen smoke passed: an owned foreground window exactly covered monitor `(0,0,1920,1080)` while the pointer remained away from the Rail. Native `IsWindowVisible(rail)=0`; the frontend projected only the collapsed state with no selection. Repeating while a native OLE drag held the left button also produced `IsWindowVisible(rail)=0`. Escape canceled the still-running drag; leaving fullscreen returned only the Rail Handle, with both references retained and no selection revival. The earlier boundary-pointer probe was inconclusive and is superseded by this result.
