# 02: Migrate Shelf and Edge Rail Surfaces to Tailwind

**What to build:** Migrate the Drop Shelf, Manage Items, Shelf Batch cards, Shelf Item rows, Shell Visuals, resize handles, Rail Handle, and expanded Edge Rail to Tailwind utilities while preserving the accepted 0.0.10 appearance, drag behavior, accessibility, virtualization measurements, transparent-window geometry, and native surface behavior.

**Blocked by:** 01 — Establish Tailwind Theme Foundation and Shared Action Controls.

**Status:** resolved

**Parent specification:** [DropCove 0.0.11 — Tailwind Component Styling Migration](../spec.md)

## Acceptance criteria

- [x] Ordinary layout, spacing, dimensions, borders, radii, overflow, typography, cursor, user-selection, touch, and interaction presentation on every Shelf and Edge Rail surface use Tailwind utilities.
- [x] Superseded component CSS is removed as each surface migrates; no compatibility class, dormant fallback, or parallel handwritten presentation path remains.
- [x] Existing fixed geometry remains exact, including Shelf Batch height and gap, Shelf Item row height and gap, minimum grid width, action targets, Shell Visual dimensions and overlap, Rail card geometry, footer/header bounds, and surface radii.
- [x] Dynamic virtual-container positioning, spacer dimensions, item top/left/width calculation, and runtime Shell Visual sizing remain runtime-owned rather than converted to static utilities.
- [x] Ordinary and virtualized Shelf Batch and Shelf Item collections preserve measured extents, bounded mounting, column calculations, scroll bounds, Tab behavior, focused-item retention, and Manage Items placement.
- [x] Cross-component relations use direct utilities, existing class fallthrough, or explicit arbitrary descendant variants without new styling props, generic variants, or wrapper components.
- [x] Any retained cross-component selector is minimal, cannot be replaced safely by the approved Tailwind mechanisms, and records why the exception is required.
- [x] Drop Shelf empty, populated, drag-over, resize-hover, warning/status, confirmation, Clear, and disabled states match the captured baseline.
- [x] Shelf Batch and Shelf Item names, Shell Visuals, truncation, availability text, Pin/Bulk Pinning, Manage Items, Remove, Clear, and drag regions match the captured baseline.
- [x] Manage Items header, summary, rows, status, actions, popup bounds, Escape behavior, close behavior, and focus restoration match the captured baseline from both Drop Shelf and Edge Rail origins.
- [x] The Rail Handle and expanded Edge Rail retain mirrored Left/Right corner ownership, continuous borders, no-activate behavior, compact cards, bounded growth, scrolling, Open Shelf placement, Clear placement, and hold-open behavior.
- [x] All eight resize directions preserve hit regions, cursors, hover outline, preferred/manual sizing behavior, and native window clamping.
- [x] Shell Visual image, icon, thumbnail, stacked, unavailable, and fallback states preserve runtime sizing, image fit, clipping, and cancellation behavior.
- [x] Dark, Light, and actual Windows Contrast Theme presentation preserve accepted semantic colors, system colors, focus indicators, warnings, danger states, borders, and transparent corners.
- [x] Before/after bounding-box and computed-style evidence shows no unexplained difference for the migrated surfaces on identical fixtures.
- [x] Native intake, drag-out Copy behavior, pinning, removal, Clear confirmation, Temporary/Pinned lifecycle, source-file safety, placement, and accessibility semantics remain unchanged.
- [x] Compact scrollbar styling, persistent scroll-viewport insets, Lucide icon replacement, and every other 0.0.12 change remain absent.
- [x] Package, crate, Tauri, PE, installer registration, and output filename remain `0.0.10`.

## Testing seam

Use the actual Drop Shelf, Manage Items, collapsed Edge Rail, and expanded Edge Rail in the running Windows Resident Instance. Compare against the ticket 01 baseline with the same fixtures, DPI, geometry, theme, and states. Exercise ordinary and virtualized collections, pointer and keyboard flows, native drag states, Left/Right docking, Dark/Light/actual Contrast Theme, and the existing frontend production build. Existing native tests remain the regression seam for behavior outside presentation.

## Demo path

Open empty and populated Drop Shelf surfaces, cross the virtualization threshold, resize through every direction, and demonstrate drag-over, status, Clear, cards, rows, and Shell Visual variants. Open Manage Items from both origins. Demonstrate collapsed and expanded Edge Rail on Left and Right edges with scrolling and focus. Show before/after geometry and computed-style evidence with no unexplained change.

## Non-goals

Settings or confirmation migration; visual cleanup; changed card/row density; scrollbar redesign; persistent viewport inset; Lucide icons; new component APIs; native geometry changes; Shelf state or persistence changes; version synchronization; packaging; or Git commit creation.

## Answer

Delivered in 0.0.11:
- Migrated Drop Shelf, Manage Items, Shelf Batch cards, Shelf Item rows, Shell Visuals, resize handles, and Edge Rail to direct Tailwind utilities.
- Removed superseded component styles; retained dynamic virtual container positioning and Shell Visual sizing.
- 35 states and 2,292 elements verified against 0.0.10 baseline with 0 computed-style/geometry differences.
- User confirmed human acceptance across all interactive on-screen gates: Left/Right Rail, resize handles, drag/drop, and Manage Items.
- Package, crate, and Tauri metadata maintained at 0.0.10 until authorized release cutover to 0.0.11.
- No Git commit created.
