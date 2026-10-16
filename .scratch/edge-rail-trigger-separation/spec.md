# Edge Rail Handle Trigger Separation

Status: ready-for-agent
Design confirmation: approved by user on 2026-10-15
Testing seam: actual Windows Resident Instance and WebView2 Edge Rail surface, supported by the existing native Rail timing and geometry regression; this preserves the previously approved highest surface seam
Ticket decomposition: pending `/to-tickets`
Implementation approval: pending explicit user approval after ticket decomposition
Release assignment: unassigned; the active 0.0.13 release gate remains separate
Supersedes: collapsed Rail Handle full-surface Open Shelf requirements 25 and 89 from the completed Compact Drop Shelf and Edge Rail UI Refinement specification
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 4, 6, and 7
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

The collapsed Rail Handle currently combines two different intentions across one narrow `16 × 280` surface: pointer presence expands the Edge Rail after a delay, while clicking anywhere opens the Drop Shelf. When the pointer approaches the Rail Handle slowly, expansion can fail to begin, forcing the user to move outside the Edge Rail and enter again. The full-surface Open Shelf target also gives the user no stable region dedicated only to expansion.

The user needs Open Shelf and Edge Rail expansion to be separate, predictable actions. Moving the pointer into the expansion region at any speed must reliably start the established expansion delay. The Open Shelf control must remain immediately actionable without causing expansion, and the split must preserve the existing Edge Rail lifecycle, storage drag, accessibility, placement, and source-file safety behavior.

## Solution

Widen the collapsed Rail Handle to `24 × 280` logical pixels and divide it into two vertically stacked regions. A `24 × 24` icon button at the top performs Open Shelf only. The remaining `24 × 256` body is a pointer-only expansion region with a decorative vertical grip. The button and body have independent hover feedback, and the former vertical `Open Shelf` label is removed.

Entering the body starts the existing `200 ms` Edge Rail expansion delay regardless of pointer speed or whether the pointer arrived from outside the window or from the Open Shelf button. Entering the Open Shelf button never starts expansion and cancels a body expansion that has not yet completed. Once the Edge Rail is expanded, moving over its existing Open Shelf button does not collapse it. Leaving the complete Edge Rail retains the existing `300 ms` collapse delay and transient-hold behavior.

Open Shelf remains one keyboard-focusable icon button with a native tooltip, visible focus indicator, action-specific accessible name, and native Enter/Space activation. The decorative body and grip are not controls and do not enter the Tab order. Valid external storage drag entry remains a separate native trigger over the whole Rail window, including the pixels occupied by the Open Shelf button.

## User Stories

1. As a pointer user approaching the Rail Handle slowly, I want the Edge Rail expansion delay to start reliably, so that I do not need to leave and re-enter the Rail Handle.
2. As a pointer user approaching the Rail Handle quickly, I want the same expansion timing as a slow approach, so that pointer speed does not change whether expansion works.
3. As a user who wants the expanded Edge Rail, I want a dedicated body region for expansion, so that Open Shelf and expansion are not competing actions on the same target.
4. As a user who wants the Drop Shelf, I want a dedicated Open Shelf button at the top of the Rail Handle, so that I can open the Drop Shelf directly.
5. As a user hovering the Open Shelf button, I do not want the Edge Rail to expand, so that the button performs only the action it communicates.
6. As a user clicking the collapsed Open Shelf button, I want the Drop Shelf to open immediately, so that expansion is not an intermediate step.
7. As a user moving from the Open Shelf button into the Rail Handle body, I want the `200 ms` expansion delay to start immediately on entering the body, so that I do not need to leave the complete Rail window first.
8. As a user moving from the body onto the Open Shelf button before expansion completes, I want the pending expansion canceled, so that the Edge Rail does not open while I am targeting Open Shelf.
9. As a user moving onto the Open Shelf button after the Edge Rail has expanded, I want the Edge Rail to remain expanded, so that its layout does not collapse before I activate the button.
10. As a user pausing over the Rail Handle body, I want the existing `200 ms` delay retained, so that incidental travel across the screen edge does not expand the Edge Rail immediately.
11. As a user leaving the complete Edge Rail, I want the existing `300 ms` collapse delay retained, so that small pointer excursions do not close the surface abruptly.
12. As a user returning before the collapse deadline, I want collapse canceled as it is today, so that I can continue interacting without reopening the Edge Rail.
13. As a user working through Clear Temporary Items, batch drag, or Manage Items, I want the existing transient holds retained, so that the Edge Rail does not collapse during an active workflow.
14. As a user clicking the Rail Handle body, I do not want an unrelated Open Shelf or immediate expansion action, so that the body remains a pointer-presence region rather than a misleading button.
15. As a user viewing the collapsed Rail Handle, I want the Open Shelf icon and expansion grip to look different, so that I can distinguish the two regions without text.
16. As a user hovering the Open Shelf button, I want only that button to show hover feedback, so that its hit region is clear.
17. As a user hovering the Rail Handle body, I want only the body and grip to show hover feedback, so that the expansion region is clear.
18. As a user with the Rail docked on the left, I want the Open Shelf arrow to point inward toward the desktop, so that its direction matches the resulting Drop Shelf transition.
19. As a user with the Rail docked on the right, I want the Open Shelf arrow to mirror and point inward toward the desktop, so that the control remains spatially consistent.
20. As a user of the expanded Edge Rail, I want its existing top-corner Open Shelf control and content layout retained, so that this change does not redesign the expanded surface.
21. As a keyboard user, I want the collapsed Open Shelf control to remain one Tab stop with a visible focus indicator, so that the icon-only presentation remains operable.
22. As a keyboard user, I want Enter and Space on the collapsed Open Shelf button to open the Drop Shelf immediately, so that keyboard activation matches pointer activation.
23. As a keyboard user, I do not want the decorative body or grip to enter the Tab order, so that there is no focus stop without a keyboard action.
24. As a keyboard user focusing Open Shelf, I do not want focus alone to expand the Edge Rail or create a collapse hold, so that keyboard focus does not acquire hidden pointer semantics.
25. As a screen-reader user, I want the collapsed icon button to expose the concise name `Open Shelf`, so that replacing visible text does not remove the action name.
26. As a pointer user, I want the collapsed icon button to expose the native `Open Shelf` tooltip, so that the icon remains discoverable.
27. As a user dragging valid files or folders into DropCove, I want entry over any part of the collapsed Rail Handle to retain the existing delayed expansion behavior, so that the new button does not create a dead storage-drop zone.
28. As a user hovering the Open Shelf button without dragging storage items, I do not want the storage-drag expansion rule applied, so that ordinary pointer hover and valid external drag remain distinct triggers.
29. As a user dragging unsupported data or an application-owned drag over the Rail, I want the existing rejection and drag-session behavior retained, so that the trigger split does not broaden accepted input.
30. As a user opening the Drop Shelf from the Rail, I want it shown and activated on the display where the Edge Rail is currently docked, so that placement remains predictable.
31. As a user interacting with the no-activate Edge Rail, I want foreground focus behavior preserved, so that hovering or expanding the Rail does not steal focus from another application.
32. As a user in Dark, Light, or Windows Contrast presentation, I want both collapsed regions, their hover states, the grip, border, icon, and focus indicator to remain legible, so that the split works in every supported presentation.
33. As a user with reduced-motion preferences, I want the existing motion policy preserved, so that this interaction change does not introduce new decorative animation.
34. As a user seeing a Rail status or error, I want existing status reporting and live accessibility behavior preserved, so that the layout split does not hide failures.
35. As a user removing the final Shelf Item, I want the existing transition to Hidden retained, so that an empty Edge Rail is not left on screen.
36. As a user opening Open Shelf while a Rail transient is active, I want the established transient cleanup and unchanged Shelf data behavior retained, so that the new button does not bypass lifecycle rules.
37. As a maintainer, I want the feature to reuse existing Open Shelf, pointer timing, storage-drag, icon, theme, and accessibility mechanisms, so that one presentation change does not create a second state model.
38. As a release owner, I want the completed interaction exercised in the actual Windows Resident Instance on both docking edges, so that native window geometry, WebView2 hit testing, no-activate behavior, and pointer timing are proved together.

## Implementation Decisions

- This is a follow-on presentation change. It deliberately supersedes only the completed refinement requirements that preserve the collapsed Rail Handle as a full-surface Open Shelf target with a vertical label. The completed specification remains historical evidence; this specification is authoritative for the new collapsed interaction.
- Preserve the domain distinction between the Shelf, Drop Shelf, Edge Rail, Rail Handle, Open Shelf, and Edge Rail expansion. Open Shelf shows and activates the Drop Shelf; expansion reveals the expanded Edge Rail without changing the display state to the Drop Shelf.
- Change the collapsed Rail Handle geometry from `16 × 280` to `24 × 280` logical pixels. Keep the total collapsed height unchanged.
- Allocate the top `24 × 24` logical pixels to one Open Shelf button. Allocate the remaining `24 × 256` logical pixels to the expansion body.
- Remove the collapsed vertical `Open Shelf` text. Use the existing `14`-logical-pixel inward directional arrow inside the existing standard `24 × 24` icon-button target.
- Mirror the arrow by docking edge: rightward for a left-docked Rail and leftward for a right-docked Rail.
- Represent the body with a quiet decorative vertical grip. The grip conveys the expansion region, remains pointer-transparent within the body, is hidden from accessibility APIs, and is not focusable.
- Keep one continuous collapsed Rail surface with the established square screen-edge corners, rounded desktop-facing corners, border, theme tokens, and forced-color mappings. Do not present the body as a second button.
- Give the Open Shelf button and expansion body independent hover feedback using the existing semantic hover tokens. Hovering one region must not visually activate the other.
- The expansion body uses the normal non-clickable cursor treatment. It has no click, Enter, or Space action.
- Pointer entry into the collapsed body schedules expansion using the existing `200 ms` delay. Arrival speed must not affect scheduling, and entering from the Open Shelf button must not require leaving the Rail window first.
- Pointer entry into the collapsed Open Shelf button does not schedule expansion. Moving from the body to the button before the `200 ms` deadline cancels the pending expansion.
- The collapsed Open Shelf button remains immediately active through pointer click and native Enter/Space button activation. Activating it invokes the existing Open Shelf lifecycle directly rather than expanding first.
- The collapsed Open Shelf button remains the only focusable element in the collapsed Rail Handle. It retains a visible focus indicator, native `Open Shelf` tooltip, and `Open Shelf` accessible name. Focus alone does not expand the Edge Rail or establish a transient hold.
- After expansion, preserve the complete existing expanded Edge Rail layout and its desktop-facing top-corner Open Shelf button. Pointer movement over that button does not collapse an already expanded Rail.
- Preserve the existing `300 ms` collapse delay when the pointer leaves the complete expanded Edge Rail. Preserve re-entry cancellation and all existing transient holds.
- Keep valid external storage drag entry as a separate native trigger over the entire Rail window. A valid drag over either the Open Shelf button or body schedules the existing delayed expansion; ordinary pointer hover over the button does not.
- Preserve existing Copy eligibility, unsupported-input rejection, application-owned drag exclusion, drag leave behavior, and storage-drop state mutation.
- Preserve the existing Open Shelf command, transient cleanup, current-docked-display placement, Drop Shelf activation, and no-activate Edge Rail behavior.
- Preserve current Rail state ownership, timers, commands, events, Shelf state, persistence, final-item hiding, fullscreen policy, display fallback, settings ownership, status reporting, and source-file safety.
- Preserve the current expanded width, expanded height formula, batch layout, scrolling, actions, and Clear/Manage Items behavior. Only the collapsed width and collapsed presentation hit regions change.
- Reuse the installed icon library and existing icon primitive. Do not add an icon package, custom tooltip system, frontend component framework, persistence field, setting, telemetry, compatibility layer, or second design system.
- No ADR is required. The change is reversible presentation behavior and does not alter state or placement ownership.
- Do not assign a release version or change roadmap status in this specification. Ticket decomposition and explicit implementation approval remain separate gates, and the active release gate must finish independently.

## Testing Decisions

- The primary and highest acceptance seam is the running Windows Resident Instance with the actual WebView2 Edge Rail. This is the existing user-approved Rail surface seam and is the only seam that proves native window dimensions, screen-edge hit testing, pointer speed, WebView2 region transitions, no-activate behavior, docking-edge mirroring, keyboard focus presentation, and storage drag behavior together.
- A good acceptance check observes user-visible state transitions and actions. It does not assert Vue event-handler names, DOM implementation structure, CSS class names, Tauri command wiring, or internal timer fields.
- On both Left and Right docking edges, verify the collapsed native window is `24 × 280` logical pixels, the Open Shelf button occupies the top `24 × 24`, and the expansion body occupies the remaining height without gaps or overlapping targets.
- Enter the Open Shelf button from outside and remain longer than `200 ms`; verify the Edge Rail stays collapsed. Click the button and verify the Drop Shelf opens immediately on the Rail's current display.
- Enter the expansion body slowly, at ordinary speed, and quickly; in every case verify the Edge Rail expands after the established `200 ms` delay without requiring an outside/re-enter cycle.
- Move from the Open Shelf button into the body; verify the `200 ms` delay starts when the body is entered. Move from the body onto the button before the deadline; verify expansion is canceled even after waiting beyond the original deadline.
- Expand the Edge Rail and move onto its Open Shelf button; verify the Rail remains expanded. Leave the complete surface and verify collapse after `300 ms`; re-enter before the deadline and verify collapse is canceled.
- Exercise Clear Temporary Items, batch drag, and Manage Items transient holds; verify the trigger split does not collapse the Edge Rail while an established hold is active.
- Hover each collapsed region separately in Dark, Light, and an actual Windows Contrast Theme. Verify independent hover feedback, continuous mirrored borders/corners, visible inward arrow, visible grip, and no new animation outside the established motion policy.
- Traverse the collapsed surface with the existing keyboard route. Verify exactly one focusable Open Shelf control, visible focus, native Enter/Space activation, native tooltip, accessible name, and no focus stop on the body or grip. Verify focus alone does not expand the Edge Rail.
- Drag valid external storage paths across both the top button pixels and body pixels; verify both paths retain delayed expansion and Copy behavior. Exercise unsupported data and application-owned drag paths to verify they remain rejected or isolated as before.
- Verify Open Shelf retains current-display placement, closes established Rail transients, shows and activates the Drop Shelf, and leaves Shelf data unchanged except through already-established actions.
- Verify removing the final Shelf Item still hides the Rail and that errors/status announcements remain perceivable in the collapsed and expanded lifecycle.
- Update the existing native Rail timing and geometry regression for the new collapsed width while preserving assertions for the `200 ms` expansion delay, cancellation before expansion, `300 ms` collapse delay, re-entry cancellation, expanded dimensions, and height tiers.
- Preserve the existing native storage eligibility regression as supporting coverage. Do not treat it as proof of WebView2 hit-region behavior.
- Do not add a frontend component-test framework for this feature. The split-region bug depends on the real Windows no-activate WebView2 surface, so a synthetic component test would be a lower-value duplicate seam.
- Prior art is the Edge Rail workflow's native timer regression and real Windows Rail smoke, plus the compact refinement's actual-surface checks for both docking edges, themes, icon buttons, tooltips, accessible names, focus indicators, and no-activate behavior.

## Out of Scope

- Redesigning the expanded Edge Rail header, cards, actions, footer, scrolling, or height formula.
- Changing the `200 ms` expansion delay or `300 ms` collapse delay.
- Adding click-to-expand, keyboard-expand, focus-to-expand, or a second focusable control to the Rail Handle body.
- Changing valid storage drop eligibility, Copy semantics, application-owned drag handling, or drag completion rules.
- Changing the Open Shelf command, Drop Shelf placement, configured monitor ownership, disconnected-monitor fallback, or remembered dismissal state.
- Changing Shelf state, persistence, settings, fullscreen policy, transient ownership, status ownership, or source-file safety.
- Adding visible `Open Shelf` or `Expand` text to the collapsed Rail Handle.
- Adding a new icon dependency, tooltip subsystem, frontend test framework, animation system, design system, telemetry, compatibility path, or persistence migration.
- Rewriting the completed Compact Drop Shelf and Edge Rail UI Refinement specification; its superseded criteria remain historical context.
- Assigning a release version, decomposing implementation tickets, approving implementation, changing roadmap delivery state, packaging an installer, or committing repository changes.
- Supporting platforms other than Windows x86_64.

## Further Notes

- The glossary defines Open Shelf separately from Edge Rail expansion: Open Shelf shows and activates the Drop Shelf on the display where the Edge Rail is currently docked.
- Pointer hover and valid external storage drag intentionally differ over the same top `24 × 24` pixels. Ordinary pointer hover does not expand; a valid external drag still schedules expansion because the native storage-drop target owns the complete Rail window.
- The previous full-surface Open Shelf target optimized click area. This follow-on change accepts a standard `24 × 24` Open Shelf target in exchange for a larger, dedicated, reliable expansion body and clearer action separation.
- The narrower `16`-pixel option was considered and rejected during design confirmation because it would make the icon and Open Shelf target too narrow. The selected `24`-pixel width reuses the established icon-button size.
- The active release remains outside this specification. `/to-tickets` is the next workflow step; implementation requires separate explicit user approval after ticket decomposition.
