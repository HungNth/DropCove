# 02: Drop Shelf Item Actions Round Trip

**Parent specification:** [DropCove 0.0.27 — Type-aware Item Actions](../spec.md)

**What to build:** Add a complete Item Actions Menu workflow to eligible single-item Drop Shelf cards. External References can be shown in File Explorer or copied as exact paths; Managed Text can be copied in full; Managed Images expose no More action. The workflow resolves the current item by identity in Core Shelf State, performs real native Windows side effects, reports local feedback, and never changes Shelf lifecycle or selection.

**Blocked by:** 01 — Authoritative Item Classification and Managed File Path Privacy

**Status:** resolved

## Acceptance criteria

- [ ] One read-only item-action boundary accepts a Shelf Item identity plus Show in File Explorer, Copy Path, or Copy Text; the WebView does not send authoritative path or text payloads.
- [ ] Core Shelf State rejects missing/stale identities and invalid ownership/content-kind combinations before any native side effect.
- [ ] The action resolver releases persistence/state locks before Windows clipboard or Shell work begins.
- [ ] Copy Path writes the exact retained External Reference path as native Unicode text with no quotes, newline, normalization, canonicalization, or existence requirement.
- [ ] Copy Text reads and writes the complete exact Managed Text content, preserving Unicode and line endings beyond the bounded hover preview.
- [ ] Managed Text More/Copy Text capability is rendered for cached Available, Missing, and Unavailable states; cached availability never hides the menu or pre-rejects the action.
- [ ] Copy Text distinguishes the committed Managed File record state `ready` from cached item availability, performs a live file read at invocation, succeeds if previously missing/unavailable bytes are readable again before projection refresh, and reports a non-mutating local error when they remain unreadable.
- [ ] Show in File Explorer checks the current path on activation, selects a file or non-root folder in its parent, opens a filesystem root, and reports inaccessible/missing failure without mutation.
- [ ] Eligible single-item Drop Shelf cards render action controls in Pin/Unpin, More, Remove order; Managed Images retain Pin/Unpin, Remove with no More button.
- [ ] External menus contain Show in File Explorer then Copy Path; Managed Text menus contain Copy Text only; multi-item cards have no ambiguous Item Actions Menu.
- [ ] The shared Item Actions Menu supports Arrow Up/Down, Home/End, Enter/Space, Escape, Tab exit, outside-click dismissal, one-menu ownership, and focus restoration to More.
- [ ] More/menu pointer and keyboard interaction does not start drag, change Batch Selection, move a selection anchor, mutate Pin state, consume a Temporary Item, remove an item, resize, reorder, or change display state.
- [ ] Copy success announces `Path copied.` or `Text copied.` locally; failures identify the failed operation locally; successful Explorer reveal has no redundant DropCove banner; no Windows toast is emitted.
- [ ] Menu closure covers action completion, target disappearance, projection change, scrolling/unmount, another More action, busy/drag/resize/move/modal ownership, and Drop Shelf hide.
- [ ] Native tests with temporary SQLite/Profile Data prove exact action resolution, invalid-action rejection, full text/path extraction, stale identity rejection, staged/unready-record rejection, cached Missing/Unavailable live-read behavior, and complete pre/post Shelf equality.
- [ ] Real Windows smoke reads the actual clipboard back and observes Explorer file/folder/root behavior plus missing-path failure.
- [ ] Frontend behavior tests exercise rendered capability, action order, keyboard/focus behavior, close triggers, feedback, and selection isolation through observable output.
- [ ] Frontend production build and freshly cargo-xwin-compiled native regression tests pass for this slice.

## Testing seam

Use the new Core Shelf State read-only action boundary with real temporary SQLite/Profile Data for deterministic validation. Use the installed/development Windows executable for actual clipboard readback and Explorer observation; do not substitute mock-echo adapter tests for native side effects. Extend existing card renderer and menu interaction seams for DOM, ARIA, focus, and event isolation.

## Demo path

1. Create eligible External file, External folder, Managed Text longer than 500 characters, and Managed Image one-item batches.
2. Copy exact path and exact full text from the Drop Shelf and read the real Windows clipboard back.
3. Restore Managed Text bytes after a cached Missing/Unavailable classification and verify Copy Text succeeds without projection refresh; repeat while unreadable and verify local failure with the item retained.
4. Reveal the file, folder, and root in Explorer; then remove a fixture and observe safe local failure.
5. Verify Managed Image and multi-item cards have no Item Actions Menu.
6. Build a Batch Selection, invoke one card action, and verify selection and every Shelf Item lifecycle remain unchanged.

## Dependencies and limits

- Requires Ticket 01 classification/privacy contract.
- Establishes the shared menu and authoritative native action foundation used by Tickets 03 and 04.
- Does not yet expose Item Actions in Manage Items or the Edge Rail.
