# 01: Drop Shelf Batch Selection Pin/Unpin round trip

**Parent specification:** [DropCove 0.0.28 — Selection Actions](../spec.md)

**What to build:** Make the first complete Selection Actions workflow work on the Drop Shelf: secondary-click or keyboard-open the Selection Actions Menu for one or several selected Shelf Batches, see scope-correct Pin/Unpin labels, and atomically change every targeted Shelf Item while preserving selection, Item Actions, virtualization, and rollback safety.

**Blocked by:** External release gate — `0.0.27` final acceptance as recorded by the canonical Roadmap. No Selection Actions ticket blocks this slice.

**Status:** ready-for-agent

**Dependencies:** Existing Batch Selection, virtualized Drop Shelf grid, established menu keyboard behavior, single-item and whole-batch Pinning, serialized Core Shelf State mutations, and Shelf snapshot publication.

**Testing seam:** The real Drop Shelf in a Windows Resident Instance with ordinary and virtualized Batch Selection, real secondary-click and keyboard invocation, supported by mounted Vue interaction/accessibility tests and Core Shelf State temporary SQLite rollback tests.

**Demo path:** Build a mixed non-contiguous Batch Selection containing one-item and multi-item batches, secondary-click a selected card, Pin Selected, reopen and Unpin Selected, repeat through `Shift+F10`, then secondary-click an unselected card and verify it becomes the sole menu target.

- [ ] Secondary-clicking a selected card preserves and targets the complete Batch Selection; secondary-clicking an unselected card replaces selection with that card before opening.
- [ ] Secondary-button `Ctrl`, `Shift`, and `Ctrl+Shift` modifiers do not extend, toggle, or range-select.
- [ ] Pin, More, Manage, Remove, confirmation, input, and textarea controls do not retarget selection or open Selection Actions; the WebView browser menu does not appear on the card selection region.
- [ ] One selected one-item batch shows Pin Item or Unpin Item; one selected multi-item batch shows Pin All Items or Unpin All Items; two or more selected batches show Pin Selected or Unpin Selected.
- [ ] All-Pinned targets offer Unpin; None and Mixed targets offer Pin, including Available, Missing, and Unavailable retained items.
- [ ] One authoritative batch-selection mutation validates a non-empty unique batch identity set before writes and updates every contained item in one transaction.
- [ ] Unknown, duplicate, stale, or wrong-domain identities reject the complete mutation with no visible or durable partial state.
- [ ] Successful Pin/Unpin publishes one replacement Shelf snapshot, retains Batch Selection and its valid anchor, and invalidates prepared drag state bound to the older Shelf snapshot.
- [ ] Validation or persistence failure preserves prior Pin states, selection, source objects, sizing, and durable state.
- [ ] Pointer invocation anchors the menu at the invocation point; Context Menu key and `Shift+F10` anchor it to the focused card.
- [ ] Menu/menuitem semantics, Arrow/Home/End, Enter/Space, Escape, Tab exit, focus restoration, visible focus, and Contrast Theme resources match the established menu contract.
- [ ] Ordinary and more-than-50-batch virtualized collections retain exact selected identities; scroll, unmount, projection replacement, target disappearance, drag, resize, move, modal ownership, hide, fullscreen, and display transitions dismiss stale menu ownership.
- [ ] Opening Selection Actions closes Item Actions and vice versa; Item Actions retain their existing More-only capability matrix and selection-neutral behavior.
- [ ] This internal delivery slice exposes no disabled, stub, or misleading Remove entry; existing direct Remove Batch controls remain available. It is not a release candidate, and Ticket 02 must complete the canonical Selection Actions Menu before any candidate or broader-surface rollout.
