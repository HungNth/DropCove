# 03: Manage Items Row Actions and Virtualized Lifecycle

**Parent specification:** [DropCove 0.0.27 — Type-aware Item Actions](../spec.md)

**What to build:** Expose the approved Item Actions Menu on eligible Manage Items rows without changing Item Selection semantics. The clicked row remains the sole action target, existing selection stays intact, virtualized rows close stale menus safely, and the popup's keyboard, focus, accessibility, and lifecycle contracts remain complete.

**Blocked by:** 02 — Drop Shelf Item Actions Round Trip

**Status:** resolved

## Acceptance criteria

- [ ] External Reference rows expose Show in File Explorer then Copy Path; Managed Text rows expose Copy Text; Managed Image rows expose no More button.
- [ ] Eligible row action order is Pin/Unpin, More, Remove; Managed Image remains Pin/Unpin, Remove.
- [ ] Activating More or a menu item targets only that row, does not select it, does not clear or retarget Item Selection, and does not move the selection anchor.
- [ ] An action on a selected row still targets only that row; an action on an unselected row leaves the visible selection unchanged.
- [ ] Pin/Unpin, Remove, Bulk Pinning, direct drag, selected-row drag, unselected-row drag, and Escape ordering retain their 0.0.26 scope and behavior.
- [ ] The existing cyclic Tab order incorporates each eligible More control in source-row action order without losing virtualized focus recovery.
- [ ] Standard menu keyboard behavior, accessible names, focus-visible styling, local announcements/status, and focus restoration work in Dark, Light, and Windows Contrast presentation.
- [ ] Pointer-opened Manage Items from the Edge Rail remains no-activate and receives no forced keyboard focus or live-announcement ownership.
- [ ] At most one row menu is open; it closes on action, Escape, outside click, another More action, row removal, batch replacement/disappearance, projection refresh, scroll/unmount, popup close, owner hide, busy mutation, modal confirmation, or native drag ownership.
- [ ] Ordinary and more-than-50-item collections preserve exact menu target identity through virtualization and never dispatch an action to a recycled row.
- [ ] Managed File Profile Data paths remain absent from row tooltip, accessible text, menu labels, announcements, and errors; External paths remain available.
- [ ] Frontend behavior tests extend the existing Manage Items row, selection, virtualization, and focus seams instead of asserting component source or CSS strings.
- [ ] Actual Windows smoke covers pointer and keyboard actions, selected/unselected rows, ordinary and virtualized collections, stale target closure, Copy/cancelled external conditions, and popup lifetime from both origins.
- [ ] Frontend production build and freshly cargo-xwin-compiled native regression tests pass for this slice.

## Testing seam

Use the existing Manage Items component renderer, Item Selection, roving-focus, and virtual collection seams. Exercise observable action target, selection state, mounted row identity, ARIA output, Tab order, focus owner, and status. Use the real popup on Windows for no-activate origin and native action proof.

## Demo path

1. Open a mixed batch from the Drop Shelf and create a non-empty Item Selection.
2. Copy/reveal from selected and unselected rows and verify only the clicked row participates while selection remains unchanged.
3. Tab through Bulk Pinning, Close, row drag regions, Pin, More, and Remove controls; exercise menu keyboard commands and Escape.
4. Repeat in a batch above 50 items, scroll/unmount an open target, and verify the menu closes without stale dispatch.
5. Open the popup from the Edge Rail and verify pointer actions do not activate DropCove.

## Dependencies and limits

- Reuses the shared menu, authoritative action boundary, and native adapters completed in Ticket 02.
- Does not add selection-wide actions, batch actions, right-click menus, or new popup geometry.
