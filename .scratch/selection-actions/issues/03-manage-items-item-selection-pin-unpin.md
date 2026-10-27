# 03: Manage Items Item Selection Pin/Unpin

**Parent specification:** [DropCove 0.0.28 — Selection Actions](../spec.md)

**What to build:** Extend the shared Selection Actions Menu to Drop-Shelf-origin Manage Items so one or several selected Shelf Items can be pinned or unpinned atomically through secondary-click or keyboard invocation, with exact item-domain validation, virtualization stability, action isolation, selection retention, and deterministic focus behavior.

**Blocked by:** 01 — Drop Shelf Batch Selection Pin/Unpin round trip.

**Status:** ready-for-agent

**Dependencies:** Ticket 01's shared menu interaction model, existing popup-local Item Selection and virtualized grid, independent row actions, Bulk Pinning, focused-item recovery, and serialized Core Shelf State publication.

**Testing seam:** The focus-capable Manage Items popup opened from Drop Shelf with ordinary and more-than-50-item collections, real pointer modifiers and keyboard invocation, supported by mounted Vue interaction/accessibility tests and item-domain Core Shelf State rollback tests.

**Demo path:** Open a 65-item batch from the Drop Shelf, create a non-contiguous Item Selection, secondary-click a selected row and Pin Selected, invoke Unpin Selected with `Shift+F10`, then secondary-click an unselected row and verify only that item becomes the target.

- [ ] A secondary-click on a selected row preserves and targets the complete Item Selection; a secondary-click on an unselected row replaces selection with that row before opening.
- [ ] Secondary-button modifiers are ignored, and row Pin, More, Remove, confirmation, input, and textarea controls do not retarget selection or open Selection Actions.
- [ ] The WebView browser menu remains suppressed on Shelf Item row selection boundaries while More remains the sole Item Actions Menu entry point.
- [ ] One selected row shows Pin Item or Unpin Item; two or more selected rows show Pin Selected or Unpin Selected.
- [ ] All-Pinned targets offer Unpin; None and Mixed targets offer Pin without changing availability, ordering, item identity, popup geometry, or source objects.
- [ ] One authoritative item-selection mutation validates a non-empty unique item identity set and requires every target to belong to the declared active Shelf Batch before writes.
- [ ] Unknown, duplicate, stale, wrong-domain, or cross-batch identities reject the complete operation with no partial state.
- [ ] Successful Pin/Unpin updates exactly the selected item identities in one transaction, publishes one replacement snapshot, retains Item Selection and valid anchor, and invalidates stale prepared drag state.
- [ ] Failure preserves selection, focus ownership, prior Pin states, visible and durable Shelf state, and Managed File bytes.
- [ ] Context Menu key and `Shift+F10` open for the focused row; an unselected focused row becomes the sole target first.
- [ ] Menu navigation, Escape, Tab exit, focus restoration, accessible names, local announcements, and Dark/Light/Contrast presentation match the approved focus-capable contract.
- [ ] Virtualization above 50 items preserves off-screen selected identities and exact target scope; scroll/unmount closes stale menu pixels without changing selection.
- [ ] Existing per-row Pin/Unpin, More, and Remove controls retain singleton scope; Bulk Pinning retains complete-batch scope.
- [ ] Existing Item Selection drag, selected counts, focus recovery, popup close/replacement, and Item Actions privacy regressions remain unchanged.
- [ ] Rail-origin Manage Items remains on its prior pointer/no-activate behavior until Ticket 05 enables complete Selection Actions parity; this ticket adds no partial Rail-origin menu.
