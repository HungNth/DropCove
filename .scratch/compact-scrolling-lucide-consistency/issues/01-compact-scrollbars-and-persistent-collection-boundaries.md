# 01: Compact Scrollbars and Persistent Collection Boundaries

**What to build:** Give the Drop Shelf, expanded Edge Rail, Manage Items, and Settings one compact native WebView2 scrollbar treatment, and keep Shelf Batches and rows visibly separated from the top edge of each applicable collection viewport at every scroll position. The completed slice must work for ordinary and virtualized collections without changing layout, scroll bounds, keyboard navigation, drag workflows, or native surface placement.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.12 — Compact Scrolling and Lucide Icon Consistency](../spec.md)

## Acceptance criteria

- [x] The Drop Shelf batch grid, expanded Edge Rail batch list, Manage Items item list, and Settings content pane use one native WebView2 scrollbar policy.
- [x] A vertical scrollbar appears only when content overflows; no surface permanently reserves a channel when its content fits.
- [x] The scrollbar interaction channel is 8 pixels wide and the visible rounded thumb is approximately 4 pixels wide on the real WebView2 surface.
- [x] The scrollbar track is transparent, including under Windows Contrast Theme.
- [x] The idle thumb uses the semantic default-border color, hover uses the semantic hover-border color, and active drag uses the semantic accent color.
- [x] Windows Contrast Theme uses `ButtonText` for the idle thumb and `Highlight` for hover and active drag, with distinguishable real-system-theme rendering.
- [x] Drop Shelf, Edge Rail, and Manage Items collection content is clipped behind a fixed 4-pixel top viewport boundary outside the actual scrollport.
- [x] Settings receives the compact scrollbar treatment but no collection top boundary.
- [x] Fully or partially visible Shelf Batches and rows cannot paint into the 4-pixel boundary at the start, middle, or maximum-scroll position.
- [x] Ordinary collections with at most 50 entries and virtualized collections with more than 50 entries follow the same visual boundary invariant.
- [x] Virtualized start/end calculation, absolute positioning, total height, bounded mounting, column width, focused-item retention, keyboard reveal, and maximum-scroll behavior remain based on the real zero-based scrollport without component-specific inset offsets.
- [x] The Drop Shelf preserves minimum-width and multi-column calculations, card width and height, 4-pixel gaps, horizontal-overflow suppression, automatic growth, resize behavior, transparent border geometry, and drag workflows.
- [x] The Edge Rail retains 4 pixels above and 4 pixels below its rows without adding vertical clearance; one through four, five through eleven, twelve, and more than fifty batches retain the accepted expansion and scrolling behavior.
- [x] Five through eleven Edge Rail batches do not acquire a premature scrollbar from duplicated top spacing, and the final row remains reachable at maximum scroll.
- [x] Manage Items preserves popup geometry, 54-pixel rows, 5-pixel gaps, Tab and Shift+Tab traversal, Escape handling, focus restoration, item drag, removal, and status behavior.
- [x] Drop Shelf and Edge Rail placement, no-activate behavior, fullscreen handling, Pin/Remove/Manage/Clear actions, and Copy-only drag behavior remain unchanged.
- [x] No custom Vue scrollbar, second scroll-position model, component-specific virtual offset parameter, compatibility path, or permanent frontend test framework is introduced.
- [x] Frontend production typechecking and bundling pass after the complete scrolling slice.
- [x] Package, crate, Tauri, PE, installer registration, and output filename remain version 0.0.11 in this ticket.
- [x] No Git commit is created without separate explicit user authorization.

## Testing seam

Use the actual Windows Resident Instance and real WebView2 surfaces. Exercise all four scrollbar owners with fitting and overflowing content in Dark, Light, and actual Windows Contrast Theme. Use ordinary and virtualized fixtures at the start, middle, and end of the viewport; record scrollbar dimensions and states, the 4-pixel clip boundary, mounted-entry bounds, column widths, focused-item reveal, and maximum scroll. Existing frontend and native checks provide regression coverage; do not add source-text assertions or a permanent visual test framework.

## Demo path

Open the Drop Shelf at minimum and multi-column widths, scroll ordinary and virtualized batches through partial-card positions, and demonstrate the persistent top boundary. Expand the Edge Rail with tiered and virtualized batch counts and show that its existing height formula does not scroll early. Open short and virtualized Manage Items lists and traverse them by pointer and keyboard. Overflow Settings and demonstrate idle, hover, and dragged thumb states in Dark, Light, and Windows Contrast Themes.

## Non-goals

Lucide icon replacement; action or empty-state glyph sizing; TypeScript 7; new Settings fields; custom scrollbars; collection redesign; changes to Shelf state, persistence, drag semantics, placement, or native window ownership; version synchronization; NSIS packaging; work after 0.0.12; or Git commit creation.

## Answer

Resolved after automated checks and the user's interaction acceptance. Real WebView2 confirmed the 4-pixel external collection boundary, 8-pixel native scrollbar channel, ordinary/virtualized collections, reactive columns, and maximum-scroll reachability. Frontend and cargo-xwin builds passed; the corrected native suite passed 101/101 tests.

Historical pre-correction finding: the old 11-batch height of 624 pixels was 10 pixels short once the 28-pixel header, 28-pixel footer, 2-pixel borders, and collection clearance were included. The user approved `52*n+62`; the measured correction below resolved this conflict. Subsequent user acceptance covers the interactive gates listed at the end of this ticket.

Corrected-build smoke asserts horizontal clearance on virtualized rows: Drop Shelf 7 pixels per side at 180/356/524 widths and 1/2/3 columns; Rail 6 pixels per side; Manage Items 7 pixels per side. Horizontal padding remains inside the scrollport; shared positioning reads actual computed padding rather than fixed component offsets. All three retain the 4-pixel top boundary. Frontend and cargo-xwin builds passed after the correction.

Expanded Edge Rail height budget verification with corrected formula `52*n+62` on 2026-10-15:
- Executable: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe` (13,385,728 bytes, SHA-256 `db96271dd6654f184f7f8ff644ef1d8ba2be0940c1235bcd71b781bfa18d7276`).
- Seam: Real Windows WebView2 expanded Edge Rail on isolated profile and unique CDP port, seeded with owned fixture files through `accept_drop`, docked via native main window `WM_CLOSE`, and expanded via `on_rail_pointer_enter` and `tick_rail`.
- Proved exact height progression for batch counts 5 through 11 with 52px step and zero overflow:
  - Count 5: projected/window height 322, scroll content 260, client height 260, overflow=false (no scrollbar), 4px fixed boundary, 6px horizontal clearance, last card visible.
  - Count 6: projected/window height 374, scroll content 312, client height 312, overflow=false, 4px fixed boundary, 6px clearance, last card visible.
  - Count 7: projected/window height 426, scroll content 364, client height 364, overflow=false, 4px fixed boundary, 6px clearance, last card visible.
  - Count 8: projected/window height 478, scroll content 416, client height 416, overflow=false, 4px fixed boundary, 6px clearance, last card visible.
  - Count 9: projected/window height 530, scroll content 468, client height 468, overflow=false, 4px fixed boundary, 6px clearance, last card visible.
  - Count 10: projected/window height 582, scroll content 520, client height 520, overflow=false, 4px fixed boundary, 6px clearance, last card visible.
  - Count 11: projected/window height 634, scroll content 572, client height 572, overflow=false (prior 10px deficit resolved; no premature scroll), 4px fixed boundary, 6px clearance, last card visible.
- Proved batch count 12: clamped maximum height 640, scroll content 624, client height 578, vertical overflow present (scrollHeight > clientHeight), 4px fixed boundary, 6px left clearance and 14px right clearance (6px card clearance to 8px native scrollbar channel), maximum scroll reachable with 12th card fully visible in viewport.
- Clean cooperative shutdown (`--shutdown --profile`) returned exit code 0; primary process exited cleanly; owned profiles, WebView2 data, fixtures, and scaffolds deleted.
- Limits of this automated slice: batch counts 1–4 and virtualized >50 items were not re-exercised here; high-DPI fractional scaling was not tested. The later user acceptance below covers the requested physical interaction and actual-theme gates. Packaging belongs to ticket 03.


### User acceptance — 2026-10-15
- User confirmed human acceptance across all interactive on-screen gates: physical scrollbar hover/drag, 1–4 and 5–11 batch expansion, 12+ batch scrolling, Drop Shelf, Edge Rail, Manage Items, Settings, and Windows Contrast Theme.
