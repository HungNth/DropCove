# 03: Mirrored Compact Edge Rail

**What to build:** Refine the collapsed and expanded Edge Rail into mirrored screen-edge surfaces with square screen-edge corners, symmetric rounded desktop-facing corners, compact Shelf Batch rows, a conditional bottom-left Clear icon, and an inward top-corner Open Shelf icon. Remove the expanded retained-batch header while preserving no-activate interaction, Rail Handle activation, timers, placement ownership, transient holds, scrolling, drag/drop, and fullscreen behavior.

**Blocked by:** 02 — Compact Shelf Batch and Manage Items Controls.

**Status:** resolved

**Parent specification:** [DropCove 0.0.10 — Compact Drop Shelf and Edge Rail UI Refinement](../spec.md)

## Acceptance criteria

- [x] A Left-docked Rail has square top-left/bottom-left screen-edge corners and continuously bordered rounded top-right/bottom-right desktop-facing corners in both collapsed and expanded states.
- [x] A Right-docked Rail mirrors that geometry: square top-right/bottom-right corners and continuously bordered rounded top-left/bottom-left corners in both states.
- [x] Neither docking edge shows the current square bottom desktop-facing mismatch, an all-corners-rounded expanded surface, a doubled border, or a transparent/opaque corner artifact.
- [x] The expanded `Retained Batches (N)` title is removed.
- [x] Open Shelf is an inward directional icon at top-right for Left docking and top-left for Right docking, with native tooltip, accessible name, hover/focus treatment, and forced-color support.
- [x] The collapsed Rail Handle retains its vertical label and entire existing activation target; no second small Open Shelf control is introduced.
- [x] Clear Temporary Items is a bottom-left trash icon only while Temporary Items exist and continues to use the existing no-activate transient hold and confirmation lifecycle.
- [x] Compact Edge Rail batch rows are `48` logical pixels high with a `4`-pixel gap. Expanded height follows `max(280, min(640, 52 × batch count + 62))` after the approved 0.0.12 chrome-budget correction, keeping one through four batches at 280 pixels, growing five through eleven batches, and scrolling at 640 pixels for twelve or more.
- [x] Compact rows preserve Shell Visuals, truncated names, all icon actions, direct item/batch drag regions, scrolling, virtualization, and Manage Items anchoring without clipping or overlap.
- [x] Edge-dependent visual placement consumes the active Rail projection; it does not create a second docking-state source or alter settings/remembered-monitor ownership.
- [x] Hover/drag-entry expansion delay, pointer/drag-leave collapse delay, no-activate behavior, transient hold-open, final-item hide, display/DPI reflow, fullscreen hide/restore, drop-in, drag-out, and Open Shelf monitor selection remain unchanged.
- [x] Left/Right, collapsed/expanded, short/scrolling, all-Pinned/mixed, confirmation, management flyout, error, and disabled states remain legible in Dark, Light, and Windows Contrast presentation.
- [x] Package, crate, Tauri, PE, and installer metadata remain `0.0.9`.

## Testing seam

Use the actual no-activate Edge Rail in the running Windows Resident Instance. Exercise Left and Right docking across collapsed/expanded states, counts on every compact-height tier, scrolling, pointer/drag timers, drop-in, item/batch drag-out, Manage Items, Clear confirmation, Open Shelf, fullscreen hide/restore, display reflow, and final-item removal. Inspect real corners and control placement in Dark, Light, and Windows Contrast presentation without inferring behavior from DOM or CSS source alone.

## Demo path

Dock left, expand, and show square screen-edge corners, two rounded desktop-facing corners, top-right inward Open Shelf, and bottom-left Clear. Repeat mirrored on the right. Add enough batches to cross the 280-pixel and 640-pixel tiers, scroll and operate compact cards, open Manage Items, cancel Clear, open the Drop Shelf, and confirm foreground focus remains with the current application until the established Open Shelf transition.

## Non-goals

Top/bottom docking, new Rail settings, different monitor ownership, altered expansion/collapse timers, keyboard traversal on the no-activate Rail, fullscreen-policy changes, backend Shelf commands, release version synchronization, packaging, or Git commit creation.

## Answer

- Implemented mirrored collapsed/expanded corner radii, inward top-corner Open Shelf, bottom-left conditional Clear, compact rows and native height calculation. Existing collapsed full-surface label/activation and native Rail ownership remain.
- Physical native cursor hover expanded Right Rail to 280 × 280. Left Rail with 65 batches expanded to 280 × 640; actual radii were 0/8/8/0, Open Shelf x=249, and row height=48. Native scrollTop 2200 retained bounded mounting (16 cards).
- Actual Rail Clear cancel retained 65 batches; confirm retained exactly three Pinned IDs and removed the trash icon. Open Shelf transitioned back to the full Shelf. Dark/Light/forced-colors media-emulated Rail screenshots were inspected; actual Windows theme and full no-activate/fullscreen/display regression remain human acceptance.
- TDD at the existing resident transition seam: focused rail-height regression failed before the formula fix and passed afterward; full Windows native suite passed 101/101. No new framework or compatibility path was added.
- Metadata remained 0.0.9 during this slice and was synchronized for the exact candidate in Ticket 04. Final source review found no remaining defect; no commit created.