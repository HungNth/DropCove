# DropCove 0.0.10 — Compact Drop Shelf and Edge Rail UI Refinement

Status: resolved
Version: 0.0.10
Design confirmation: approved by user on 2026-10-15
Testing seam: actual Windows Resident Instance and packaged application approved by user on 2026-10-15
Ticket decomposition: four vertical slices approved by user on 2026-10-15
Implementation approval: all four tickets approved by user on 2026-10-15; user confirmed final release acceptance on 2026-10-15; Git commit authorization remains separate
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.10
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 8–11 and 19
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

DropCove 0.0.9 preserves a functional Shelf workflow, but its compact presentation still wastes space and renders inconsistently at transparent window boundaries. The Drop Shelf carries a title/subtitle header and verbose empty guidance. Shelf Batch cards repeat visible drag hints and text actions, while Manage Items rows and headers use text controls that consume height. The expanded Edge Rail retains a header title and text actions even though its narrow surface needs corner-positioned controls.

The rounded surfaces also do not form one reliable window-frame contract. The Drop Shelf and Manage Items can expose corner gaps or outer-border artifacts, and the expanded Edge Rail rounds screen-edge corners that must remain square. The Rail's bottom desktop-facing corner can appear mismatched with its rounded top corner. These defects are visible across Light, Dark, and Windows Contrast presentation and make the surfaces appear detached from their transparent borderless windows.

Users need a compact, consistent presentation without changing Shelf state, persistence, drag-and-drop, pinning, removal, confirmation, focus restoration, placement, no-activate Rail behavior, or source-file safety.

## Solution

Refine the existing presentation surfaces in place. Align Drop Shelf and Manage Items backgrounds and borders flush with their window frames. Give collapsed and expanded Edge Rail surfaces square screen-edge corners and symmetric rounded desktop-facing corners for both docking edges.

Remove the Drop Shelf header, reduce the empty state to a downward icon and `Drop here`, and remove visible drag-hint subtitles while retaining full drag guidance in tooltips and accessibility text. Replace verbose action labels with compact inline-SVG icon controls using one visual language, native tooltips, visible hover/focus states, and action-specific accessible names and states.

Move Clear Temporary Items to a conditional bottom-left trash icon on the Drop Shelf and expanded Edge Rail. Remove the expanded Rail title and place an inward Open Shelf arrow in the desktop-facing top corner. Preserve the full collapsed Rail Handle as the existing Open Shelf target. Verify the completed design through the actual Windows Resident Instance and one exact installed 0.0.10 candidate rather than adding a frontend component-test framework.

## User Stories

1. As a user opening the Drop Shelf, I want its rounded surface to meet the transparent window frame without corner gaps, so that it appears as one coherent Windows surface.
2. As a user opening Manage Items, I want its rounded popup surface to have the same flush frame treatment, so that related Shelf surfaces feel consistent.
3. As a user docking the Edge Rail on the left, I want both left screen-edge corners square and both right desktop-facing corners rounded, so that the Rail appears attached to the screen edge.
4. As a user docking the Edge Rail on the right, I want both right screen-edge corners square and both left desktop-facing corners rounded, so that the Rail mirrors the left-edge presentation correctly.
5. As a user switching between collapsed and expanded Edge Rail states, I want corner ownership to remain consistent, so that expansion does not introduce a mismatched bottom corner or border.
6. As a user in Dark presentation, I want surface backgrounds, borders, icons, and focus indicators to remain visually coherent.
7. As a user in Light presentation, I want transparent corners and compact controls to remain legible without dark edge artifacts.
8. As a Windows Contrast user, I want system colors, borders, icons, and focus indicators to remain distinguishable.
9. As a user opening the Drop Shelf, I do not want a redundant `DropCove` title or `References, not copies` subtitle consuming workspace.
10. As a user viewing an empty Drop Shelf, I want only a downward icon and `Drop here`, so that the drop target is immediate and compact.
11. As a user viewing a Shelf Batch, I want its name and Shell Visual to remain primary, so that action guidance does not crowd the card.
12. As a user dragging a card, I want its complete path or drag instruction retained in tooltip and accessibility text even though the visible drag-hint subtitle is removed.
13. As a user viewing a single-item batch, I want Pin and Remove controls on one compact trailing row.
14. As a user viewing a multi-item batch, I want Pin All, Manage Items, and Remove controls on one compact trailing row.
15. As a user opening Manage Items, I want Bulk Pinning and Close controls represented compactly while the title and pinned summary remain readable.
16. As a user inspecting a Manage Items row, I want Pin and Remove controls on one compact row with the availability label.
17. As a user, I want Pin controls to show whether the reference is retained without requiring visible button text.
18. As a user managing a mixed batch, I want Bulk Pinning to distinguish Off, On, and Indeterminate states.
19. As a user removing a reference, I want a remove-reference icon distinct from the trash icon, so that the control does not imply deletion of the source file or folder.
20. As a user clearing Temporary Items, I want a recognizable trash icon that still opens the existing confirmation before any Shelf state changes.
21. As a user with no Temporary Items, I do not want an irrelevant disabled trash control.
22. As a user of the expanded Edge Rail, I do not want a redundant `Retained Batches (N)` header consuming vertical space.
23. As a user with the Rail docked left, I want Open Shelf at the top-right with an arrow pointing into the desktop.
24. As a user with the Rail docked right, I want Open Shelf at the top-left with an arrow pointing into the desktop.
25. As a user of the collapsed Rail Handle, I want its existing full-surface activation target retained, so that opening the Drop Shelf does not require hitting a small icon.
26. As a pointer user, I want every icon action to expose a native tooltip and clear hover state.
27. As a keyboard user, I want every icon action reachable in the existing Tab order with a visible focus indicator and standard Enter/Space activation.
28. As a screen-reader user, I want each icon action to expose a concise action-specific name and current state.
29. As a screen-reader user, I want binary Pin controls represented as pressed or not pressed and Bulk Pinning represented as a three-state checkbox.
30. As a user working with unavailable paths, mixed batches, confirmations, or errors, I want the existing lifecycle and status behavior preserved while controls become compact.
31. As a user resizing the Drop Shelf, I want the established eight-direction resize, cursor, hover outline, sizing intent, and responsive grid behavior preserved.
32. As a user dragging references into or out of DropCove, I want native Windows Copy behavior and Temporary/Pinned consumption rules unchanged.
33. As a user opening Manage Items from either the Drop Shelf or Edge Rail, I want placement, no-activate behavior, focus restoration, and dismissal rules unchanged.
34. As a release owner, I want the installed 0.0.10 application to match the verified development surfaces across Dark, Light, and Windows Contrast presentation.
35. As a maintainer, I want the refinement implemented without an icon package, frontend test framework, second design system, or new product state.

## Implementation Decisions

- Treat version 0.0.10 as a clean presentation cutover. Remove obsolete visible labels and layout paths rather than preserving compatibility variants.
- Preserve the domain distinction between the persistent Shelf and its Drop Shelf and Edge Rail presentation surfaces. The Rail Handle remains the collapsed state of the Edge Rail.
- Keep Core Shelf State, persistence, window ownership, commands, events, drag preparation/completion, confirmations, monitor placement, resizing, focus restoration, and source-file safety unchanged.
- Use the existing transparent borderless window composition. Correct surface box sizing, clipping, backgrounds, borders, and radii first. Change native window setup only if the running Windows surface proves that CSS cannot remove the artifact.
- Apply flush frame treatment to the Drop Shelf and Manage Items window surfaces. Internal Batch Drag and Clear Temporary confirmation dialogs retain their independent modal borders and radii. The decorated Settings window is outside this corner contract.
- Give the Edge Rail one mirrored corner rule in both collapsed and expanded states: screen-edge corners square, desktop-facing corners rounded. Left docking rounds the right corners; right docking rounds the left corners.
- Keep borders continuous through both desktop-facing Rail corners. Do not fake the bottom corner with an overlay or edge-specific one-off element.
- Remove the Drop Shelf title/subtitle header in every state. The empty state contains only a downward icon and the exact visible text `Drop here`.
- Remove only the visible Shelf Batch drag-hint subtitle. Retain the draggable region's full-path tooltip and its accessible `drag to copy` name.
- Use dependency-free inline SVG paths with `currentColor`. A small shared visual primitive may own SVG sizing and decorative behavior; action semantics remain at each button call site.
- Standard icon buttons are `24 × 24` logical pixels with `14–16` logical-pixel icons. They use existing semantic colors and forced-color system mappings rather than fixed icon colors.
- Use an outline/filled pushpin for binary Pin state. Bulk Pinning uses the same visual language with distinct Off, On, and Mixed presentation. Use a list icon for Manage Items, minus-in-circle for Remove Reference/Batch, `X` for Close, trash can for Clear Temporary Items, and an inward directional arrow for Open Shelf.
- Use the existing native `title` pattern for icon tooltips. Do not build a custom tooltip subsystem for this release.
- Binary Pin buttons expose dynamic accessible names and `aria-pressed`. Bulk Pinning remains `role="checkbox"` with `aria-checked="false"`, `"true"`, or `"mixed"`; selectors and visual states must follow that actual contract.
- Keep action semantics outside the shared icon primitive. Remove, Close, Manage Items, Clear, and Open Shelf retain their current command and lifecycle owners.
- Place each card or row's actions in one trailing row. Compact layout must not hide names, availability, pinned summary, status, or keyboard focus.
- Set compact Edge Rail batch rows to `48` logical pixels with a `4`-pixel gap and use `max(280, min(640, 52 × batch count + 62))` for expanded height (chrome budget corrected with user approval in 0.0.12). Update Drop Shelf and Manage Items virtualization measurements to their resulting rendered card/row extents; do not leave stale fixed-height assumptions that overlap or skip content.
- Retain the Manage Items title, item count, and pinned summary. Replace only Bulk Pinning, row actions, and Close presentation.
- Show the bottom-left trash icon only while Temporary Items exist. Activating it opens the existing confirmation; cancellation and failure preserve the exact prior Shelf state.
- Remove the expanded Edge Rail title. Place Open Shelf at top-right for a left-docked Rail and top-left for a right-docked Rail, with the arrow pointing away from the screen edge.
- Preserve the collapsed Rail Handle's full activation target and vertical label. Do not add a second small Open Shelf button to the collapsed state.
- No persistence schema, settings field, backend command, compatibility layer, telemetry, icon dependency, component framework, or general design-system rewrite is introduced.
- No ADR is required. The refinement follows established surface ownership, accessibility, theme, and Windows-only architecture and introduces no hard-to-reverse ownership decision.

## Testing Decisions

- The primary seam is the running Windows Resident Instance using the actual Drop Shelf, Manage Items, collapsed Edge Rail, and expanded Edge Rail. This is the only seam that proves transparent frame alignment, no-activate Rail behavior, WebView2 rendering, focus, and real surface geometry together.
- Exercise Drop Shelf corners in empty and populated states, during normal, drag-over, resize-hover, confirmation, status, and disabled-control conditions.
- Exercise Manage Items from both Drop Shelf and Edge Rail origins, including pointer and keyboard opening, virtualized and non-virtualized batches, status display, close, Escape, and focus restoration.
- Exercise collapsed and expanded Edge Rail on both Left and Right edges. Inspect all four corners, continuous borders, inward Open Shelf placement, bottom-left Clear placement, scrolling, and transient hold-open behavior.
- Run every surface in Dark, Light, and Windows Contrast presentation. Verify `currentColor` icons, system-color forced-color mappings, visible borders, hover states, and focus indicators.
- Exercise each icon action with pointer, Tab, Shift+Tab, Enter, and Space where the existing control is keyboard-enabled. Observe native tooltips, action-specific accessible names, binary pressed state, and tri-state checked state.
- Verify visible drag-hint text is absent while full path and drag guidance remain available through tooltip and accessibility inspection.
- Use mixed Temporary/Pinned batches to verify Pin, Bulk Pinning Off/On/Mixed, Remove Item, Remove Batch, Clear confirmation, cancellation, failure retention, and final-item behavior without re-testing private command forwarding.
- Use enough Shelf Batches and Shelf Items to cross the virtualization threshold. Compact height changes must preserve bounded mounting, correct scrolling, logical position, Tab order, and focus retention.
- Run the existing `pnpm build` contract for Vue typechecking and production bundling. Do not add a frontend component-test framework or permanent tests that assert CSS source text, SVG path bytes, labels removed from templates, or exact DOM class names.
- Run the complete produced Windows native regression executable and the cargo-xwin release/Tauri/NSIS build. Existing native tests remain the regression seam for Shelf state, drag, persistence, placement, shutdown, and installer behavior.
- Before the version bump, launch the real development/release application and smoke the changed surfaces. Keep package, crate, Tauri, PE, and installer metadata at 0.0.9 until the presentation and regression gates pass.
- Synchronize metadata to 0.0.10 only after implementation verification. Build one exact `DropCove-Setup-0.0.10.exe`, record setup and installed executable sizes and SHA-256 hashes, and exercise those exact installed bytes.
- Exact-candidate smoke covers the refined surfaces plus intake, drag-out, hotkey, startup, Settings, Shake-to-open, Cooperative Shutdown, update/repair, uninstall, and Profile Data preservation.
- Good permanent tests catch consumer-visible state or geometry regressions that can be exercised deterministically. Presentation acceptance relies on the actual surface rather than snapshots, source assertions, mock echoes, or bare process existence.
- No unavailable hardware or theme coverage is inferred. Record any DPI, monitor, Contrast Theme, or interaction limit that was not directly observed.

## Out of Scope

- New Shelf behavior, persistence semantics, drag result rules, availability rules, sizing intent, monitor selection, Edge Rail timers, fullscreen policy, Settings fields, or keyboard-navigation model.
- Redesigning confirmation dialog content, Settings, notification-area UI, Shake-to-open, installer UI, Recovery Backup, or Cooperative Shutdown.
- Replacing the collapsed Rail Handle with an icon-only control.
- Custom tooltip infrastructure, animation redesign, icon packages, component-test frameworks, screenshot regression infrastructure, or a general-purpose design-system migration.
- New source-file deletion behavior. Remove and Clear continue to remove references only.
- Supporting top/bottom Rail docking, macOS, Linux, x86, or ARM64.
- TypeScript 7 or any product scope after version 0.0.10.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- The current implementation already owns every required action and lifecycle path. Version 0.0.10 changes presentation and geometry rather than introducing commands or persisted state.
- The current Bulk Pinning template already uses the correct three-state checkbox API, but its visual selectors target the wrong ARIA attribute. The refinement treats this as an existing presentation defect and aligns styling with `aria-checked`.
- Version metadata is synchronized to 0.0.10. Final release acceptance was confirmed by user on 2026-10-15 against the exact candidate recorded in Ticket 04.
- Ticket decomposition and implementation approval remain separate workflow gates. Publishing this specification does not authorize a Git commit.

## Delivery Evidence

- Technical source implementation is complete across all four approved slices. The exact candidate and installed executable hashes, exercised native surface/OLE behavior, 101/101 native regression, final two-axis review, and cleanup evidence are recorded in [Ticket 04](issues/04-release-0.0.10.md) and the roadmap.
- User confirmed all outstanding acceptance tests passed on 2026-10-15 against the exact candidate recorded in Ticket 04. The ambient default database hash change remains recorded as an un-backed-up restart observation rather than an unverified restoration claim.
- All four tickets are resolved. Git commit authorization remains a separate gate.