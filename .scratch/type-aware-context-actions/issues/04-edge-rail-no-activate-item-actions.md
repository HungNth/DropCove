# 04: Edge Rail No-activate Item Actions

**Parent specification:** [DropCove 0.0.27 — Type-aware Item Actions](../spec.md)

**What to build:** Expose the approved Item Actions Menu on eligible single-item cards in the expanded Edge Rail while preserving its pointer-only, no-activate, non-focus-stealing behavior. Copy actions keep the current foreground application; explicit Show in File Explorer may activate Explorer; menu and feedback lifetime remain coordinated with Rail collapse, fullscreen, display, drag, and surface transitions.

**Blocked by:** 02 — Drop Shelf Item Actions Round Trip

**Status:** resolved

## Acceptance criteria

- [ ] Eligible External and Managed Text single-item Rail cards expose the same More/action mapping and Pin/Unpin, More, Remove order as the Drop Shelf; Managed Images and multi-item cards expose no Item Actions Menu.
- [ ] Opening or operating the Rail menu with the pointer does not activate or focus DropCove and does not steal foreground from the current application.
- [ ] Copy Path and Copy Text preserve foreground ownership while writing the exact real Windows clipboard payload.
- [ ] Explicit Show in File Explorer may activate Explorer because the user directly requested that destination.
- [ ] Menu pointer interaction is treated as part of the expanded Rail interaction region so normal hover/collapse timing does not dismiss it mid-action.
- [ ] At most one Rail Item Actions Menu is open, and it closes cleanly on action, outside pointer, target disappearance, projection change, collapse, Open Shelf, final-item hiding, fullscreen occlusion, display reflow, settings placement change, owner hide, drag/modal ownership, and Resident Instance shutdown.
- [ ] Rail Copy success uses local non-toast visual feedback without claiming keyboard or screen-reader focus ownership; failures remain locally visible and actionable.
- [ ] Item Actions do not change Rail Batch Selection, Pin state, Temporary/Pinned lifecycle, item order, expansion sizing, scrolling, monitor ownership, or display state.
- [ ] Active native drag, availability confirmation, Clear Temporary Items, Manage Items, and existing transient holds keep their established precedence over Item Actions.
- [ ] Managed File Profile Data paths are absent from Rail tooltip/accessibility/status output; External Reference paths remain available.
- [ ] Real Windows smoke records the foreground owner before/after menu opening, both Copy actions, Explorer reveal, error feedback, natural collapse, fullscreen hide/restore, and Open Shelf transition.
- [ ] Frontend behavior tests cover Rail action capability, target-local dispatch, status, and lifecycle through observable state without mock foreground claims.
- [ ] Frontend production build and freshly cargo-xwin-compiled native regression tests pass for this slice.

## Testing seam

Use the actual expanded Edge Rail on Windows as the authoritative no-activate/foreground seam, with real clipboard readback and Explorer observation. Supporting frontend tests cover projection and lifecycle rendering, but foreground preservation is accepted only from the native surface.

## Demo path

1. Keep another application in foreground and expand a Rail containing External, Managed Text, and Managed Image single-item cards.
2. Open More and perform Copy Path and Copy Text; verify clipboard payloads and unchanged foreground ownership.
3. Invoke Show in File Explorer and verify Explorer intentionally becomes foreground.
4. Exercise outside-pointer collapse, fullscreen occlusion, display reflow, Open Shelf, drag/modal ownership, and final-item removal with a menu open.
5. Verify every action leaves Shelf/selection/lifecycle state unchanged.

## Dependencies and limits

- Reuses the shared menu, authoritative action boundary, and native adapters completed in Ticket 02.
- Can run in parallel with Ticket 03 because Manage Items and Rail presentation ownership are independent after the shared foundation lands.
- Does not make the Edge Rail keyboard-activatable.
