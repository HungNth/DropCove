# 05: Edge Rail and Rail-origin Selection Actions parity

**Parent specification:** [DropCove 0.0.28 — Selection Actions](../spec.md)

**What to build:** Deliver complete pointer-only Selection Actions on the expanded Edge Rail and in Manage Items opened from the Rail, including Batch Selection and Item Selection Pin/Unpin/Remove, transient holds, inline confirmation, foreground preservation, virtualization, fullscreen/display lifecycle, and parity with the already working Drop Shelf workflows.

**Blocked by:** 04 — Manage Items ownership-safe Remove Selected.

**Status:** ready-for-agent

**Dependencies:** Tickets 01–04 Selection Actions menu, atomic mutations, authoritative confirmation, and cleanup behavior; existing Edge Rail selection, resident transient ownership, pointer expansion/collapse, fullscreen policy, and no-activate Manage Items origin.

**Testing seam:** One real Windows Resident Instance while another application owns foreground focus, exercising expanded Edge Rail and Rail-origin Manage Items below and above virtualization thresholds, supported by frontend Rail lifecycle tests and existing native resident/foreground regressions.

**Demo path:** Keep another application foreground, create a Rail Batch Selection, Pin and Remove it through secondary-click, open Manage Items from the Rail and repeat on Item Selection, Cancel and Confirm inline removal, then exercise pointer leave, fullscreen, display transition, scroll, and collapse without DropCove activation.

- [ ] Expanded Edge Rail Batch Selection supports the approved selected/unselected secondary-click targeting, ignored modifiers, labels, atomic Pin/Unpin, singleton Remove Batch, and confirmed Remove Selected behavior.
- [ ] Rail-origin Manage Items supports the approved Item Selection Pin/Unpin/Remove behavior without enabling keyboard invocation or focus ownership.
- [ ] Edge Rail and Rail-origin menus and confirmations are pointer-only, never call foreground activation, never focus DropCove, and add no live-announcement owner.
- [ ] Selection Actions menu and removal confirmation acquire distinct or safely composable resident transient ownership so the Rail and popup cannot collapse underneath interaction.
- [ ] Pointer leave while a transient is owned does not collapse the target; dismissal releases ownership and resumes ordinary delayed collapse.
- [ ] Inline Rail confirmation exposes the same exact counts, ownership copy, Cancel, and Remove actions without moving focus.
- [ ] Pin/Unpin success retains the applicable selection; Cancel and failure retain it; successful removal prunes disappeared identities and closes Rail-origin Manage Items only when its batch disappears.
- [ ] Fullscreen, owner destruction, cooperative shutdown, Exit, forced surface switch, and display transitions cancel unconfirmed transients without mutation and preserve authoritative teardown behavior.
- [ ] Scroll, virtualization unmount, target disappearance, projection replacement, drag start, Rail collapse, and display transition close stale menu ownership and release every hold exactly once.
- [ ] Item Actions Menu and Selection Actions Menu remain mutually exclusive; Item Actions keep More-only capability, privacy, local feedback, and explicit Explorer foreground behavior.
- [ ] Drop Shelf and Drop-Shelf-origin Manage Items retain their keyboard/focus-capable behavior without new Rail-specific branches changing them.
- [ ] Real foreground observation proves another application remains foreground during Rail menu open, Pin/Unpin, Cancel, Remove confirmation, confirmed removal, local success, and local failure.
- [ ] Ordinary and more-than-50-member collections preserve selected identities, counts, labels, and action scope across virtualization.
- [ ] Existing Edge Rail drag-in, drag-out, fullscreen hiding, Shake, Clear Temporary Items, Shell Visual, selection, and no-activate regressions remain green.
