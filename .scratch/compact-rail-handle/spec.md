# Compact Rail Handle

Status: ready-for-agent
Design confirmation: user approved restoring the original 16-pixel width, a 16-by-16 Open Shelf button, and a 10-pixel icon.
Ticket decomposition: pending
Implementation approval: separate gate after ticket decomposition
Release assignment: unassigned
Parent behavior: [Edge Rail Handle Trigger Separation](../edge-rail-trigger-separation/spec.md)

## Problem Statement

The 24-pixel collapsed Rail Handle and its Open Shelf button occupy too much screen-edge space for the user. The user wants the original narrow width while retaining the separated Open Shelf and hover-expansion interactions.

## Solution

Restore the collapsed Rail Handle to 16 by 280 logical pixels. Reduce the top Open Shelf button to 16 by 16 logical pixels and its inward arrow to 10 pixels. The remaining 16 by 264 logical-pixel outer region is the expansion body. Existing surface border chrome remains part of these outer dimensions, not additional width or height.

## User Stories

1. As a user, I want the collapsed Rail Handle restored to its original 16-pixel width, so that it occupies less screen-edge space.
2. As a user, I want a proportionally smaller Open Shelf button and arrow, so that the button fits the narrow Rail Handle.
3. As a pointer user, I want the button and expansion body to remain separate, so that hovering Open Shelf does not expand the Edge Rail.
4. As a keyboard user, I want visible focus and native Enter/Space activation retained, so that the smaller button remains operable.
5. As a user, I want tooltip, status alerts, contrast, and clickability preserved, so that shrinking the control does not remove its meaning or feedback.
6. As a user dragging storage items, I want drag expansion and end/leave reconciliation to remain correct, so that changing the top boundary does not leave stale expansion requests.

## Implementation Decisions

- Collapsed outer dimensions: 16 by 280 logical pixels.
- Top Open Shelf outer hit region: 16 by 16 logical pixels; icon: 10 by 10 logical pixels. Remaining body outer region: 16 by 264 logical pixels.
- Preserve the established real border, box sizing, mirrored corners, semantic colors, decorative grip, and independent hover feedback. Account for border chrome explicitly; do not enlarge the native window to accommodate an overflowing button.
- Update native pointer classification to use the new 16-pixel top-region boundary and verify agreement with actual WebView2 hit routing, including border pixels and DPI conversion.
- Retain one focusable native button, Open Shelf accessible name, native tooltip/status title, visible focus, Enter/Space activation, and visible/assertive status reporting. Body and grip remain non-focusable.
- Preserve 200 ms expansion, 300 ms collapse, button/body transitions, expanded-surface re-entry cancellation, valid external drag over the whole window, and drag-end/leave reconciliation.
- Do not shrink expanded Edge Rail controls or icons elsewhere. Preserve expanded dimensions, layout, lifecycle, monitor placement, no-activate behavior, and source-file safety.
- This deliberately supersedes only the collapsed geometry and icon dimensions of the parent design. The previously resolved ticket remains unchanged historical evidence.
- The user accepts a 16-by-16 pointer target, smaller than the project's usual 24-by-24 target, in exchange for restoring the narrow resting surface. Do not claim this target meets the usual 24-pixel target standard.
- No new dependency, setting, state owner, ADR, compatibility path, version bump, installer change, or commit is required.

## Testing Decisions

- Primary seam: actual Windows Resident/WebView2 surface on both docking edges. Measure native outer dimensions, button and body hit routing, 10-pixel arrow, clipping, clickability, tooltip, focus, status, and inward mirroring.
- Exercise ordinary pointer on the button, button-to-body and body-to-button transitions, expanded re-entry, 200/300 ms timing, and valid storage drag entry/end/leave over each region.
- Supporting seam: existing native timing/geometry and pointer-classification regressions, including the new 16-pixel boundary at standard and fractional DPI. These do not replace actual surface acceptance.
- Verify Tab, Enter, Space, Dark, Light, actual Windows Contrast, and reduced-motion preservation. Report unavailable checks as unobserved rather than accepted.
- Run existing frontend typechecking/build and the produced Windows native regression executable. Add no frontend framework or source-text tests.

## Out of Scope

Expanded Edge Rail redesign, unrelated icon resizing, timing changes, state or persistence changes, release assignment, packaging, Git commits, and modifying the resolved parent ticket.

## Further Notes

This artifact records the user's approved follow-on design. Ticket decomposition and implementation approval remain separate project workflow gates. The smaller pointer target is an explicit user-selected trade-off, not an accessibility improvement.
