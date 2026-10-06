# 02: Manage Bulk Pinning inside the anchored popup

**Parent specification:** [Bulk Pinning for Shelf Batches Specification](../spec.md)

**What to build:** Add Bulk Pinning to the anchored multi-item popup through a fixed header that shows the derived pinned count and reuses the atomic manager operation from Ticket 01. The header must remain reachable while long item lists scroll, participate fully in keyboard and assistive-technology workflows, and preserve existing popup lifetime, focus restoration, virtualization, and shelf geometry.

**Blocked by:** 01: Pin or unpin an entire Shelf Batch from its card.

**Status:** needs-triage

**Testing seam:** Reuse the manager behavior completed in Ticket 01; do not duplicate lifecycle logic in the popup. Use installed self-contained Release UI Automation and direct interaction for fixed-header layout, virtualization, focus order, keyboard activation, accessible state, popup lifetime, and outside-click behavior.

**Demo path:** Open a long multi-item Shelf Batch by keyboard, use the focused header action to Pin All from Mixed, traverse every virtualized item action in both directions, Unpin All with `Space`, close with `Esc`, then reopen by pointer and show that in-popup mutation stays open while a card mutation follows outside-click dismissal.

- [x] The popup has a fixed, non-scrolling header above the existing item scroller, with `X of N pinned` on the left and the tri-state bulk toggle on the right.
- [x] The header derives Off, On, and Mixed from the current Shelf Items and uses the same target rules, visual language, accessible names, and atomic manager mutation as the card.
- [x] The fixed header remains visible while a long item list scrolls vertically; horizontal scrolling remains disabled.
- [x] Existing popup width and overall height bounds remain governing. The header consumes part of the bounded popup height, and only the item list scrolls within the remaining space.
- [x] Opening, mutating, scrolling, or closing the popup does not change the preferred or actual Drop Shelf bounds, Automatic Shelf Growth, or Manual Height Override.
- [x] Activating Bulk Pinning inside the popup keeps that popup open, updates the header, card, and item rows, and retains focus on the header control after success.
- [x] A persistence failure inside the popup keeps the popup open, retains focus on the header control, leaves every state unchanged, shows the existing failure status, and performs no automatic retry.
- [x] Activating Bulk Pinning from a card remains an outside-popup interaction and closes any open popup under the existing outside-click rule; no same-batch exception is introduced.
- [x] A keyboard-opened popup initially focuses the bulk header control. Pointer opening does not force unexpected keyboard focus.
- [x] The logical focus order is the fixed header action followed by individual Pin/Unpin and Remove for every item row. `Tab` and `Shift+Tab` traverse and wrap that complete sequence in both directions.
- [x] Keyboard traversal realizes only the target virtualized row and does not eagerly materialize the complete item list.
- [x] `Enter` and `Space` activate the bulk toggle. `Esc` closes the popup before shelf dismissal and restores focus to the originating chevron when it still exists.
- [x] Individual Pin/Unpin, item removal, accepted item or batch drag-out, and restored persistence immediately recompute the visible count and aggregate state or close the popup when its existing owner rules require closure.
- [x] Full item view models and visual requests remain deferred until the popup opens and are released when it closes, hides, or loses its owner; adding the fixed header does not retain closed-popup row projections.
- [x] UI Automation reports Off, On, and Indeterminate correctly on the popup control and exposes count-aware item-centric names matching the action the user will perform.
- [x] Installed Release smoke proves fixed-header scrolling, bounded popup geometry, Off/Mixed/On transitions, success and failure focus retention, card outside-click dismissal, bidirectional traversal across virtualized rows, `Enter`/`Space`, `Esc` focus return, immediate aggregate recomputation, and unchanged shelf bounds.

## Comments

### 2026-10-06 — Popup implementation; desktop proof incomplete

- Installed red/green smoke: the old popup had no `3 of 10 pinned` header; the new installed-directory Release exposes that fixed summary, a native Indeterminate bulk toggle, and a separately bounded item scroller. Build/publish completed without warnings/errors.
- Correctly targeted `BatchPopupBulkPin` UIA actions proved deterministic abort keeps every persisted reference unchanged, the popup open, and the header focused. Manual retry pins every item and updates the count to `10 of 10 pinned` while retaining header focus.
- Actual `SendInput` Enter on Manage opens with header focus. Actual Enter on the header unpins every item; Space pins every item. Closing with Esc restored the originating Manage action. Pointer-like UIA Invoke does not force header focus.
- The complete logical cycle is implemented, but the current keyboard smoke passed only the first four forward item actions before foreground switched to the user's terminal. Synthetic input stopped immediately. Full forward/reverse wrap, fixed-header scrolling, and integrated qualification remain open; this ticket is not fully accepted.
- Driver correction: target the popup bulk toggle by `AutomationId`, not by root enumeration index. UIA roots reorder when PopupHost opens; selecting the card instead correctly closes the popup and is not a product failure.
- [Partial installed popup evidence](../../../artifacts/bulk-pinning-popup-smoke.json) records exercised behavior and the exact interrupted traversal; it does not claim a completed cycle or release qualification.

### 2026-10-06 — Exclusive-foreground final replay

- User approved exclusive foreground. [Final popup evidence](../../../artifacts/bulk-pinning-popup-smoke.json) now contains all 21 forward and reverse actions, both wraps, Enter/Space opening exactly once, header activation, and Esc return. The earlier interrupted run remains historical, not the final result.
- A keyboard-source defect was reproduced: programmatically focused Manage plus actual Enter opened without header focus. PreviewKeyDown now explicitly distinguishes Enter/Space from pointer/UIA Invoke, opens with keyboard focus, then marks handled to suppress duplicate Click. Actual Enter and Space both passed after the fix.
- A 100-item popup realizes only 10–11 row visuals at a time. End-of-list scrolling and reverse wrap leave header bounds and `350 × 348` shelf bounds unchanged. [Scrolled visual](../../../artifacts/bulk-pinning-popup-scrolled.png).
- Actual pointer opening does not force header focus. In-popup mutation retains popup/focus; a real card click closes it. Individual unpin and removal recompute Mixed/On counts immediately. Final-build deterministic abort/retry and pinned/temporary restarts preserve every reference and source file.
- Delayed real SQLite success and abort while switching from a 10-item to a 12-item popup preserve the new owner's Mixed state, name, and Manage focus. Final Standards and Spec reviews both reported zero actionable findings.
- Integrated native drag and high-contrast/scale qualification remain Ticket 03 work; no final release-ready claim.

### Superseding acceptance record — post-session native replay

- Implementation and installed interaction acceptance are complete; the earlier interrupted/partial comments are historical and superseded by final popup evidence and [correlated real drag evidence](../../../artifacts/bulk-pinning-real-drag.json).
- Actual Copy receivers accepted bulk-pinned, bulk-unpinned, and Mixed payloads; only participating Temporary references were consumed. Actual item drag consumed one Temporary item and retained its popup with `2 of 2 pinned`. Real held-Escape cancellation, explicit rejection, and a primary process exiting before Drop over an instrumented reject backing retained references.
- The triage label requests acceptance review, not further implementation. Feature release qualification remains independently gated by Ticket 03 high-contrast and current scale evidence.



- [100-batch/1,000-item actual surface proof](../../../artifacts/bulk-pinning-scale-surface.json) exercises grid end-scroll, popup, exactly-one-batch Pin All/Unpin All, unchanged references, popup closure with no row automation retained, and hide/hotkey restoration. `ReleasePopupProjections` also clears ItemsSource, realized visual requests, full item models, and header Tag/Content. This functional proof is deliberately separate from the failed residency measurement.

