# 01: Compact Manage Items presentation

**What to build:** Make Manage Items a compact 240-logical-pixel item-management surface from both the Drop Shelf and Edge Rail. Replace the visible shortened path with an availability-only second line while retaining the exact full path through tooltip and accessibility text. Preserve every established management, drag, placement, scrolling, virtualization, appearance, keyboard, focus, lifecycle, state, and source-safety contract.

**Parent specification:** [DropCove 0.0.22 — Compact Manage Items Popup](../spec.md)

**Blocked by:** None (can start immediately).

**Status:** resolved — implemented and actual Windows/WebView2 evidence recorded on 2026-10-22; final installed-candidate human acceptance belongs to Ticket 02.

## Acceptance criteria

- [x] Manage Items has a fixed client width of `240` logical pixels when opened from either the Drop Shelf or Edge Rail; no per-origin width branch or setting is introduced.
- [x] Native placement uses the new width while preserving right, left, below, then above preference, the preferred `8`-pixel origin gap, the preferred `16`-pixel work-area margin, and containment-first clamp fallback.
- [x] Each row shows the item name on the first text line and exactly `Available`, `Unavailable`, or `Missing` on the second line.
- [x] No visible row path or `Pinned ·` availability prefix remains.
- [x] The exact full path remains available through the existing drag-region tooltip and accessibility text, including for identically named items from different folders.
- [x] The Pin icon state, accessible pressed state/name, `X/N pinned` header summary, Bulk Pinning, Close, Remove Item, Shell Visual, direct item drag-out, status reporting, and action order remain unchanged.
- [x] Rows remain `54` logical pixels high with `5`-pixel gaps; maximum height remains `480` logical pixels; existing dynamic height, collection padding, scrollbar, and virtualization measurements remain unchanged.
- [x] Ordinary and virtualized batches preserve bounded mounting, scrolling to the final item, focused-row retention, accessibility set position, and complete Tab/Shift+Tab traversal.
- [x] Pointer and keyboard opening, Enter/Space activation, Escape drag cancellation and later dismissal, explicit Close, exact-origin focus restoration, replacement, owner/batch closure, and active-drag guards remain unchanged.
- [x] Dark, Light, Windows Contrast where available, focus, disabled, reduced-motion, border, action, and muted availability presentation remain coherent without a new token, animation, dependency, or abstraction.
- [x] Shelf state, persistence, availability classification, Temporary/Pinned lifecycle, Copy results, source files, and management projection ownership remain unchanged.
- [x] Existing native placement regressions cover `240`-pixel right/left/below/above candidates, monitor corners, constrained work areas, and clamp fallback.
- [x] The frontend production build passes; the cargo-xwin-built Windows native regression executable passes when run directly.
- [x] Actual Windows Resident/WebView2 inspection demonstrates the `240`-pixel surface and approved row content from both origins with short, long, identical-name, Available, Unavailable, Missing, Temporary, Pinned, ordinary, and virtualized item sets.
- [x] Version metadata remains `0.0.21`; no installer is released and no Git commit is created in this ticket.

## Testing seam

Primary: actual Windows Resident Instance and real WebView2 Manage Items surface from both origins. Supporting: existing native placement/lifecycle regressions, frontend production build, and the cargo-xwin-built native Windows suite. Add no frontend component-test framework, screenshot snapshots, CSS/source assertions, or forwarding-only tests.

## Demo path

Create a multi-item Shelf Batch containing short and long names, two same-named items from different folders, and Available/Unavailable/Missing plus Temporary/Pinned states. Open Manage Items from the Drop Shelf, measure the popup, inspect every row, hover for the full path, traverse every action by keyboard, drag one item, and scroll a virtualized batch to the final row. Repeat opening and placement from the Edge Rail.

## Non-goals

Release metadata synchronization, packaging, installation, broader surface redesign, row-density changes, path controls, per-origin geometry, state or persistence changes, new dependencies, telemetry, platform expansion, or Git commit creation.

## Evidence — 2026-10-22

- Minimal cutover: shared native width `280 → 240`, availability replaces visible path in the existing two-line item-info block, duplicate trailing `Pinned ·` status removed. Tooltip, accessibility path, actions, drag region, row height, virtualization, lifecycle, and state ownership remain unchanged.
- Existing placement tests use the production width and independent expected coordinates. The three tests were observed failing with width 280 before passing with width 240. Focused actual-crate tests passed **3/3**; complete native Windows executable passed **124/124**; frontend regression suite passed **13/13**; frontend production typecheck/build passed.
- Actual embedded 0.0.21 Windows Resident used an owned profile and unique WebView2 data directory. Drop Shelf and Edge Rail origins both rendered `240 × 480` with a 70-item batch. The Rail Manage action was also opened by a physical pointer click. A three-item ordinary batch rendered `240 × 292`.
- Real surfaces displayed Available, Missing after removing only an owned fixture, and Unavailable for an offline drive root. Two identically named references retained distinct full-path tooltip titles and actual Chromium accessibility names. Row height measured 54 pixels; the large batch mounted 10 initial rows and 11 near maximum scroll, with item 69 reachable at scrollTop 3710.
- Pointer Pin changed its pressed state and header summary. Keyboard opening focused Bulk Pinning; complete ordinary-list Tab traversal wrapped in Bulk → Close → row Pin/Remove order; reverse movement was exercised. A physical item drag followed by Escape retained the reference; row Remove removed one reference. Dark and Light were inspected visually. Source fixture bytes remained owned and intact except the deliberate Missing fixture, which was restored.
- Direct Standards and Spec review found no remaining actionable finding. Removed one stale native comment naming the old 280-pixel client. Parallel-review tool invocation was unavailable, so this was a direct review, not independent reviewer proof.
- Limits: actual Windows Contrast Theme, mixed/fractional DPI, Narrator speech, complete physical accepted-Copy target matrix, and whole virtualized keyboard traversal were not observed in this development smoke. No new test framework or committed source-text assertion was added. Version stayed 0.0.21 throughout Ticket 01.
- Initial isolated launches without a unique WebView2 data directory failed with `0x8007139F`; only owned processes were shut down. A later owned WebView directory allowed real-surface inspection. Initial startup defaults could reconcile the global Run entry before its baseline was captured; Ticket 02 must restore the ambient installed profile's configured startup command and report that original-byte restoration cannot be claimed.

## Final installed-candidate correction and verification

- Release smoke found ordinary overflow rows shrinking to 47 pixels. Added only `flex-none` to preserve the specified 54-pixel extent; final installed checks proved every row in an ordinary 35-item batch remained 54 pixels on both origins, with unchanged five-pixel gaps.
- Final installed 70-item virtualization retained 10 mounted rows, full forward Tab traversal through row 69, and wrap back to Bulk Pinning. Final three-item content remained `240 × 292` with three 54-pixel rows. Available, Missing, Unavailable, full-path accessibility, Dark/Light, physical Copy/cancellation, and source safety were rechecked on the final candidate.
- The checkboxes record implemented/preserved contracts, not a claim that every physical accessibility/hardware configuration was exercised. Actual Contrast, reduced-motion hardware configuration, mixed/fractional DPI, Narrator speech, and exhaustive physical focus/lifecycle combinations remain explicitly unobserved in the release ticket. Final human acceptance remains a separate gate.
- Final candidate, superseded artifacts, installed workflow coverage, cleanup, and exact observed limits are recorded in [ROADMAP 0.0.22](../../../docs/ROADMAP.md#0022--compact-manage-items-popup).