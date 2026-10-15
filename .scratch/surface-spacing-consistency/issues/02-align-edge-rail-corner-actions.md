# 02: Align Expanded Edge Rail Corner Actions

**What to build:** Align Open Shelf and Clear Temporary Items to matching four-logical-pixel outer vertical offsets in the expanded Edge Rail while retaining the accepted control sizes, horizontal insets, header/footer budgets, mirrored docking behavior, collection geometry, and native Rail height contract.

**Blocked by:** None (can start immediately after ticket approval).

**Status:** ready-for-human

**Parent specification:** [DropCove 0.0.13 — Surface Spacing Consistency](../spec.md)

## Acceptance criteria

- [x] Open Shelf remains a `24 × 24`-logical-pixel button and is offset `4` logical pixels from the expanded Rail's top edge.
- [x] Open Shelf remains at the desktop-facing top corner: top-right when docked Left and top-left when docked Right.
- [x] The header retains its `28`-logical-pixel budget and uses four pixels above the control with no additional space below it.
- [x] Clear Temporary Items remains a conditional `24 × 24`-logical-pixel button and remains offset `4` logical pixels from the bottom edge.
- [x] The Clear footer retains its existing minimum-height and bottom-padding budget; no empty replacement control appears when no Temporary Items exist.
- [x] Each positioned control retains its `6`-logical-pixel horizontal inset on both docking sides.
- [x] Lucide glyph size, tooltip, accessible name, disabled state, semantic color, hover, focus-visible, forced-color behavior, and action ownership remain unchanged.
- [x] Left and Right expanded Rails mirror correctly without changing screen-edge versus desktop-facing corner ownership.
- [x] Rail rows remain 48 pixels high with 4-pixel gaps, 6-pixel horizontal collection insets, the established top boundary, bottom clearance, scrollbar geometry, and final-row reachability.
- [x] The native height contract remains `max(280, min(642, 52 × batch count + 70))`; minimum and maximum heights, expansion tiers, live-growth behavior, and height regression expectations remain unchanged.
- [x] Rail Handle behavior, expansion/collapse timers, no-activate interaction, monitor placement, fullscreen policy, Open Shelf transition, Clear confirmation, and hold-open behavior remain unchanged.
- [ ] Actual Windows/WebView2 inspection measures four-pixel top and bottom visual offsets on both Left and Right expanded Rails in available Dark, Light, and Windows Contrast presentations (verified on active Dark theme; Light and Windows Contrast unobserved in this candidate run).
- [x] Frontend production typechecking/bundling and the complete produced Windows native regression executable pass.
- [x] Package, crate, Tauri, PE, installer registration, and output filename remain version 0.0.12 in this ticket.
- [x] No Git commit is created without separate explicit authorization.

## Testing seam

Use the actual expanded Edge Rail on Windows because WebView2 layout, transparent border geometry, docking direction, and native window height must agree. Measure button border boxes against the Rail surface edges on Left and Right docking, with and without Temporary Items. Exercise Open Shelf and Clear cancel/confirm behavior to prove that only visual alignment changed. Run existing frontend and native regression checks; do not add tests that assert Tailwind class strings or source text.

## Demo path

Expand a Left-docked Rail containing Temporary Items and measure Open Shelf at four pixels from the top and Clear at four pixels from the bottom. Repeat on a Right-docked Rail and confirm mirrored Open Shelf placement and unchanged Clear behavior. Remove all Temporary Items to show that Clear disappears without changing Rail geometry. Exercise representative non-scrolling and scrolling batch counts and show unchanged height tiers and final-row reachability.

## Non-goals

Manage Items placement; new controls; button, glyph, tooltip, theme, accessibility, Rail Handle, collection, scrollbar, corner-radius, timer, fullscreen, monitor, height-formula, state, persistence, drag, or confirmation redesign; dependency or design-system changes; version synchronization; NSIS packaging; or Git commit creation.

## Answer

Resolved. In `src/components/rail/RailHandle.vue`:
- Updated `rail-header` class to: `flex flex-none items-start h-[28px] box-border pt-[4px] pb-0 px-[6px]`
- Kept `rail-footer` class as: `flex flex-none [align-items:end] min-h-[24px] pt-0 px-[6px] pb-[4px]`

Real Windows Resident WebView2 measured:
- Left-docked Rail:
  - Open Shelf rect: `x: 249, y: 5, width: 24, height: 24, right: 273, bottom: 29` (top offset: `4px` from inner border, `5px` from outer surface).
  - Clear rect: `x: 7, y: 251, width: 24, height: 24, right: 31, bottom: 275` (bottom offset: `4px` from inner border, `5px` from outer surface).
  - Inset: `6px` from container border (`7px` from outer edge).
- Right-docked Rail:
  - Open Shelf rect: `x: 7, y: 5, width: 24, height: 24, right: 31, bottom: 29`.
  - Clear rect: `x: 7, y: 251, width: 24, height: 24, right: 31, bottom: 275`.
  - Header height: exactly `28px`; footer height: exactly `28px`.
- Tested on exact installed binary candidate with matching measurements.

Limits: verified on active system Dark presentation. Windows Light and Contrast Themes were not physically switched during candidate execution and remain unobserved.

Complete native test suite: **104/104 passed**.
