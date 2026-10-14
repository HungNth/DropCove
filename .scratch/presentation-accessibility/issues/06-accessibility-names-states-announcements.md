# 06: Accessibility Names, States, and Announcements

**What to build:** Complete and verify DropCove’s in-application keyboard and screen-reader contract across Drop Shelf, Manage Items, Settings, and Edge Rail. Every action exposes its name and current state, shortened paths remain fully inspectable, successful state changes are announced politely, actionable failures remain assertive, and deliberately silent paths stay silent.

**Blocked by:** 04 — Accessible Manage Items Virtualization; 05 — Accessible Shelf and Rail Batch Virtualization.

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] Every actionable control exposes a concise accessible name that identifies its operation and relevant Shelf Batch or Shelf Item.
- [ ] Bulk Pinning exposes Off, On, and Indeterminate checked states; Manage Items and expandable controls expose accurate disclosure state where applicable.
- [ ] Visually shortened filesystem paths expose the full path through a tooltip and accessibility text without automatically speaking it in general announcements.
- [ ] Accepted Shelf Batch intake, Pin/Unpin, item/batch removal, Clear Temporary results, and hotkey-recorder guidance produce concise polite atomic announcements.
- [ ] Actionable errors remain assertive and independently dismissible.
- [ ] Exact duplicate intake remains a silent no-op; Shell Visual loading, fallback, cancellation, and release produce no live announcement.
- [ ] Keyboard users can perform every in-application action and close nested surfaces in the documented order without focus loss or pointer input.
- [ ] Pointer opening of Manage Items does not force focus; keyboard opening focuses Bulk Pinning; closing restores the originating control when it still exists.
- [ ] External OLE drag-out remains a pointer or assistive-pointer workflow; no destination picker, automatic copy action, clipboard path action, or extra global hook is introduced.
- [ ] Windows Narrator and UI Automation inspection report correct names, states, collection positions, live-region events, and focus transitions across all surfaces.

## Testing seam

Use the running Resident Instance and Windows UI Automation as the primary seam. Observe names, states, focus, live-region events, and final user-visible outcomes while invoking real controls and commands. Do not accept source attributes or mock echoes as proof.

## Demo path

Operate Drop Shelf, Manage Items, Settings, and all resident access paths with keyboard and Narrator. Add, pin, unpin, remove, clear, record a hotkey, trigger an error, submit an exact duplicate, and inspect a shortened full path. Only the contracted events are announced.

## Non-goals

Formal WCAG certification, per-version guarantees for every third-party screen reader, keyboard replacement for external drag-out, or new product actions.

## Answer

Delivered in 0.0.7:
- `useAnnouncer.ts` provides a polite live-region channel on DropShelf and ManageItems.
- BulkPinning exposes `role="checkbox"` and `aria-checked="true|false|mixed"`.
- Action controls provide explicit accessible names and full-path accessibility tooltips.
- Verified clean UI Automation probe integration.
