# 01: Make Manage Items compact and explicitly dismissible

**What to build:** Make Manage Items on the Drop Shelf and Edge Rail consistently compact and easier to dismiss. Both surfaces use the approved 280-logical-pixel width, preserve useful identity from long Path References, present Bulk Pinning as `[pin] X/N pinned`, and expose an explicit Close action without changing lifecycle, drag, placement, scrolling, or ownership behavior.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Parent specification:** [Compact Manage Items Controls Specification](../spec.md)

- [x] The Drop Shelf Manage Items popup and Edge Rail Manage Items Flyout each use an outer width of 280 logical pixels under normal work-area conditions, and item content cannot expand either surface beyond that contract.
- [x] Both surfaces reuse one Shelf Item presentation rule for visible Path References rather than maintaining separate formatters.
- [x] A short Path Reference remains complete when it fits; a long file reference preserves `…\parent\file.ext`; and a long folder reference preserves `…\parent\folder`.
- [x] Root paths, root-level items, UNC-style values, trailing directory separators, Missing references, and Unavailable references are formatted without filesystem access, canonical-path mutation, or exceptions.
- [x] If the preserved tail is still too wide, character ellipsis keeps the row bounded; horizontal scrolling is not introduced.
- [x] The full canonical Path Reference remains available through the path tooltip and accessibility metadata.
- [x] Both fixed headers use the visible order Bulk Pinning, `X/N pinned`, flexible space, Close; the count updates after every mutation that changes the current Shelf Items.
- [x] Bulk Pinning retains its existing tri-state semantics, compact target, count-aware accessible action name, tooltip, busy behavior, and Off/On/Indeterminate automation state.
- [x] Item-row Pin/Unpin and Remove Item actions remain on the right with unchanged semantics, dimensions, and accessible names.
- [x] Both surfaces expose an icon-only Close button at the upper right with the tooltip and accessible name `Close Manage Items`.
- [x] Close supplements rather than replaces `Escape`, outside click or native light dismissal, and Manage Items toggle-close; every route uses the established cleanup path.
- [x] Closing restores focus to the originating Manage Items control when it still exists and safely handles owner replacement or removal.
- [x] Keyboard-opened Manage Items focuses Bulk Pinning first; `Tab` and `Shift+Tab` traverse Bulk Pinning, Close, then Pin/Unpin and Remove Item for every row with correct wrapping and on-demand row realization.
- [x] The Close action does not bypass active-drag, pending-mutation, or owner-lifetime guards and does not leave stale projections or Edge Rail open state.
- [x] Edge Rail no-activate behavior, rail expansion retention, desktop-facing placement, maximum height, fixed header, vertical scrolling, open/close motion, and collapse resumption remain unchanged.
- [x] Pin/Unpin, Bulk Pinning, Remove Item, Successful Drag-Out, Temporary consumption, Pinned retention, Missing cleanup, Unavailable retention, and source-filesystem safety remain unchanged.
- [x] Light, Dark, High Contrast, text trimming, pointer states, disabled states, mixed state, and visible keyboard focus continue to use existing WinUI controls, styles, and theme resources.
- [x] No new schema, setting, public abstraction, custom control, runtime dependency, UI test project, telemetry, timer, cache, polling loop, compatibility path, or background worker is added.
- [x] The complete existing automated suite passes, a Release build succeeds, and a direct application smoke observes both updated Manage Items surfaces before the ticket is considered complete.
- [x] All repository changes remain uncommitted until the user explicitly authorizes a Git commit.

## Testing seam

Use the installed or side-by-side self-contained Release with an isolated profile as the authority for actual WinUI width, path rendering, header order, focus, accessibility, dismissal, Edge Rail no-activate behavior, and cleanup. Keep the existing MSTest suite as regression coverage; do not add a permanent UI test project or source-text presentation tests.

## Demo path

Create one mixed-lifecycle multi-item Shelf Batch containing a short path, deeply nested file, deeply nested folder, long name, and enough items to scroll. Open Manage Items on the Drop Shelf and Edge Rail, show the stable 280-logical-pixel width, compact path tails with full tooltips, `[pin] X/N pinned [close]` header, keyboard traversal, all dismissal routes, focus return, and unchanged Pin/Unpin, Remove Item, and drag behavior.

## Comments

### 2026-10-09 — Implemented and verified; UI commit approved

- Release build/publish succeeds without warnings or errors. Full MSTest suite: 221 passed, zero failed/skipped. Native placement regression: 68 passed; the compact-width DPI test failed before the native width-policy fix and passed afterward.
- The installed EXE and application/Core/Services/Native DLLs match the publish payload. Qualified application DLL SHA-256: `6558106D6D1EE1631757BAE91003398308E52AE51577383EEF19D1EE5D363461`.
- Same-payload installed UI Automation and visual evidence covers Drop Shelf and both Edge Rail sides in Light and Dark at 96 DPI: 466 checks pass. The actual Popup surface is 280 logical pixels wide; WinUI's separate 300-pixel PopupHost includes transparent transport padding and is not the rendered surface.
- Evidence verifies short and compact file/folder paths, full-path accessibility metadata, header order, all three pin states/counts, full forward/reverse traversal over virtualized rows, fixed header, Close/Space/Escape/outside/toggle dismissal, focus return, pending-mutation Close disabling/re-enabling, Pin/Unpin/Bulk Pinning and Remove Item refresh.
- Source review confirms preserved drag/owner guards and existing theme resources. High Contrast, non-default text scaling and real drag acceptance/cancellation are not claimed as newly certified by this host smoke; these limits are recorded rather than inferred.
- Standards review: zero outstanding findings. Spec review: one computation-timing finding resolved by pure cached tail-range calculation on presentation creation/path changes; final re-review reports zero findings.
- User explicitly authorized the UI commit and separately accepted keeping Ticket 02 blocked by the inherited residency gate. The gate remains unchanged and no release-ready claim is made.
- [Qualification evidence](../qualification.json) contains payload identity, exercised cases, environment/cleanup details, paired performance results and remaining observations.
