# 02: Manage Items pointer Item Selection Copy round trip

**What to build:** Make the first complete multi-item workflow work through Manage Items for Available content: select one or several Shelf Items with Windows-style pointer gestures, see stable selected state and counts across ordinary and virtualized collections, drag a selected row as one ordered native Copy payload, drag an unselected row as a singleton, and preserve existing action scope and source safety.

**Blocked by:** 01: Occurrence-safe item-set drag foundation.

**Status:** resolved

**Parent specification:** [DropCove 0.0.26 — Manage Items Item Selection and Drag-Out](../spec.md)

**Dependencies:** Ticket 01's ordered item-set preparation/start/cancel/complete contract and the existing Manage Items virtual collection, header, item rows, Pin/Remove actions, Bulk Pinning, and announcement channel.

**Testing seam:** The actual Manage Items WebView2 popup with real pointer modifiers, ordinary and more-than-50-item batches, and real native Copy/cancel/reject destinations, supported by frontend interaction and virtualization tests plus all-Available item-set state regressions.

**Demo path:** Open a batch above the virtualization threshold, form a non-contiguous selection, replace it with an off-screen Shift range, drag one selected row into a real destination, repeat with cancel/reject, then drag an unselected row and verify that it transfers alone.

- [x] Item Selection is popup-local presentation state for the active Shelf Batch and stores stable item identities plus an optional anchor outside mounted virtual rows.
- [x] Plain click selects only the target and sets the anchor.
- [x] `Ctrl+click` toggles only the target and sets the anchor even when toggling it off.
- [x] `Shift+click` replaces selection with the inclusive source-order range and preserves a valid anchor.
- [x] With no valid anchor, Shift selects only the target and establishes it as anchor; `Ctrl+Shift` uses the same replacement-range behavior.
- [x] Range selection includes virtualized off-screen items and is independent of viewport position.
- [x] Pointer press on an already selected row preserves the pre-press selection until release below the established drag threshold.
- [x] Release below threshold collapses selection to the row; crossing the threshold drags the pre-press Item Selection.
- [x] Dragging an unselected row uses a singleton item set without first replacing visible selection.
- [x] Selected rows use existing accent resources and Windows Highlight resources in Contrast Themes without changing row or popup geometry.
- [x] The header reports `S selected · X/N pinned` while selection is non-empty and the established pinned summary otherwise.
- [x] Pin/Unpin and Remove retain per-item scope; Bulk Pinning retains complete-batch scope; action activation does not change selection or anchor.
- [x] Newly added items remain unselected; item removal prunes only vanished selected identities and clears a vanished anchor without choosing another.
- [x] Empty collection-space click clears Item Selection.
- [x] All-Available selected payload order follows source-item order rather than click order.
- [x] Accepted Copy clears Item Selection; canceled or rejected drag retains it when no separate clear trigger occurred.
- [x] The popup remains open while its batch survives, including after partial Temporary consumption.
- [x] External source objects remain byte-exact and are never removed by Copy completion.

## Evidence — 2026-10-26

- Row regression failed before implementation on absent selected semantics, then passed. Final frontend suite **43/43**.
- Exact installed pointer matrix: off-screen Shift range selected **56** items; Ctrl+Shift replacement selected **3**; toggle-off anchor followed by Shift selected **5**; empty-space click cleared the set; Shift without anchor selected **1**. Per-item Pin retained selection, and Bulk Pinning changed all **65** items without retargeting selection.
- Real Windows OLE selected Copy preserved source order and exact bytes; an unselected row emitted one path. Native Escape cancellation and destination rejection retained the selected set. All **75** owned external source hashes remained unchanged.
- Selection and preparation lifecycle are shared with Batch Selection through the common selection and prepared-drag composables, with no obsolete import aliases.
- Shared final release evidence: [Ticket 06](06-release-dropcove-0.0.26.md).
