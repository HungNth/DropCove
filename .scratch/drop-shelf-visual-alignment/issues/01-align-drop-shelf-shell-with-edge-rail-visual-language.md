# 01: Align the Drop Shelf shell with the Edge Rail visual language

**Parent specification:** [Drop Shelf Visual Alignment](../spec.md)

**What to build:** Restyle the complete Drop Shelf shell so it uses the Edge Rail's theme-aware base surface, one continuous header/content/footer layer, a subtle non-layout boundary, smaller shell glyphs with unchanged interaction targets, rounded shell-control states, and actual Windows 11 native window corners. Preserve every existing Drop Shelf interaction, responsive sizing contract, and Edge Rail behavior.

**Blocked by:** None (can start immediately).

**Status:** complete

- [x] The Drop Shelf shell uses the same theme-aware layer fill as the Edge Rail in Light, Dark, and High Contrast without introducing fixed RGB/ARGB colors or a parallel theme system.
- [x] Header, content, and footer render as one continuous shell with no contrasting header/footer bands and no separators; existing Shelf Batch cards, Shelf Item rows, popup surfaces, status surfaces, and drag feedback retain their established brushes and hierarchy.
- [x] A persistent one-logical-pixel neutral card-stroke outline renders as an overlay and does not consume content dimensions, move controls, change the `8`-pixel content padding, or alter responsive column boundaries.
- [x] Header and footer remain `32` logical pixels high; Settings remains `32 × 32`, dismissal remains `36 × 32`, and Clear Temporary Items remains `28 × 28`.
- [x] Settings, dismissal, and Clear Temporary Items use glyph sizes `11`, `10`, and `11` logical pixels respectively while retaining their existing glyph identity, command, tooltip, accessible name, focus behavior, and tab semantics.
- [x] Hover and pressed backgrounds for only Settings, dismissal, and Clear Temporary Items use the standard control corner radius; shared card, popup, Pin/Unpin, Manage Items, and Remove action styles remain visually unchanged.
- [x] The native Drop Shelf window opts into `DWMWCP_ROUND` after its resizable borderless styles are applied; Edge Rail, Settings, confirmation dialogs, popup hosts, and other windows are not opted in by this change.
- [x] The implementation relies on native DWM shadow and system corner policy and adds no Mica, Acrylic, custom XAML shadow, transparent helper window, per-pixel alpha, custom window region, or clipping fallback.
- [x] A rejected or ignored DWM corner hint cannot prevent DropCove from starting or operating, but implementation smoke does not claim rounded-corner success unless actual normal-window corners are observed.
- [x] The ordinary state shows the neutral outline; the existing one-pixel accent resize outline overlays it during resize hover without changing layout or adding per-edge drawing logic.
- [x] Borderless resizing from all existing edges and corners, cursor selection, drag region behavior, preferred width, Manual Height Override, work-area clamping, popup lifetime, and resize completion semantics remain unchanged.
- [x] Automatic Shelf Growth retains the `180/236/292/348` logical-pixel tiers, complete-row fit, no-live-shrink behavior, and existing responsive column boundaries.
- [x] No application state, persistence schema, domain model, view-model contract, drag lifecycle, dependency, timer, polling loop, telemetry, or new UI mode is introduced.
- [x] The complete automated suite passes without adding tests that assert XAML source text, resource forwarding, private element names, duplicated visual constants, or native-call wiring.
- [x] A self-contained Release smoke run against an isolated profile observes the unified shell, targeted shell-control styling, actual normal-window rounding at `180 × 180` and after resizing, native resize behavior, and unchanged core interactions before this ticket is marked complete.

## Comments

### Implementation and observed progress — qualification remains open

- The aligned self-contained Release is published side-by-side under the installed program directory, not installed over the normal resident EXE or live profile. Current application DLL SHA-256: `4ac32f5423bfa98e736524f1a0782ceaa914f9c6ec398511a64a28ee4447a195`.
- Light/Dark minimum and wider screenshots show one continuous shell and actual rounded native corners. UI Automation observes the unchanged `32 × 32`, `36 × 32`, and `28 × 28` shell targets. Real relative mouse packets observe rounded shell-control hover states; Settings keyboard activation, Clear confirmation, header drag, and all eight real resize directions/cursors are exercised.
- The neutral and accent overlays are inset one pixel inside DWM's outer border, without consuming layout space. Native hit-test evidence shows the accent overlay, but actual pointer-only movement does not show it in either the baseline binary or aligned binary. **The resize-hover criterion remains unchecked; Ticket 01 is not complete.** No input-logic workaround or new hover implementation is added outside the approved styling scope.
- Three actual two-file OLE drops grow the shelf from `180` to `236`, `292`, and `348` while preserving width and the top edge. High Contrast is unobserved; the host contrast state is not mutated.
- Product documentation describes the layer surface, DWM corners/shadow, neutral outline, scoped shell controls, unchanged geometry/Edge Rail, and the inherited pointer-hover gap. [Qualification evidence](../../../artifacts/visual-alignment-qualification.json) records exercised configurations and limitations. Final full suite: 204 passed, zero failed/skipped, 594 ms. Parallel review: Standards zero actionable findings; Spec zero actionable source findings, explicitly incomplete acceptance.
- [Cleanup](../../../artifacts/visual-alignment-cleanup.json) removes both disposable harnesses, three disposable profiles, and 1,000 unused fixture files. The independently isolated verified demo and its two source fixtures remain running intentionally; normal resident/profile data is untouched. No commit.
- Ticket 01 stays `needs-info`. The combined theme criterion and pointer-only accent hover criterion remain unchecked. Ticket 02 remains blocked by this incomplete acceptance; none of the checked implementation/smoke criteria is a release-readiness claim.

### Continuation — pointer-only hover acceptance resolved

- The user requested continuation. Tagged boundary tracing showed that `InputNonClientPointerSource` enter/move events already fired, but legacy parent-window leave messages cleared the outline while the pointer remained in the input-sink HWND. The final fix subscribes source enter/move/exit and removes obsolete MainWindow legacy move/leave tracking. Native hit-test, resize regions, and cursor APIs remain unchanged; non-border events clear both visual state and active resize direction.
- Current clean Release DLL SHA-256 is `0ef07acd6060ba2170dba483b97685d93b8fa4de3abedafcdc8d7f858ff63a51`, superseding the earlier `4ac32f…` implementation snapshot above. [Current pointer qualification](../../../artifacts/visual-hover-qualification.json) proves all eight actual relative-mouse hover cases show accent and correct cursors, then caption/client/outside restore neutral and normal cursors. No `WM_NCHITTEST` injection is used as acceptance evidence. Control/content rectangles do not move.
- All eight real resize operations retain exact expected bounds. Same-build Light/Dark minimum/wider windows and shell-control hover are captured. Full suite: 204 passed, zero failed/skipped, 643 ms. Final two-axis source review: Standards zero actionable findings; Spec zero actionable source findings. Temporary traces and the DBWIN monitor are removed/closed.
- [Failed initial probe](../../../artifacts/visual-hover-failing-probe.json) and [diagnosis](../../../artifacts/visual-hover-diagnosis.json) remain explicitly failed/diagnostic, not reclassified as passes. The first combined Light/Dark/High Contrast criterion is still unchecked while isolated contrast qualification is pending. Ticket 01 remains `needs-info`; no commit or Release-ready claim.

### Final acceptance — Ticket 01 complete, Release gate separate

- The earlier hover/High Contrast blockers above are superseded by [final qualification](../../../artifacts/visual-shelf-final-qualification.json). Final application DLL SHA-256: `7a77e7262f55c760fd9a47c9c8e9bae638e2b9d106ef7d40e29b0ed407076ce3`; compiled resource PRI SHA-256: `481ea1a3fb317c395330d97d199068ad4ca3ab5568a4767199610847f7201843`. The aggregate payload fingerprint is `2688731cf0de7fe787afef494b373eec3bc94ab5166701bced155d8330e11d84`.
- Actual [High Contrast Black guest proof](../../../artifacts/visual-final-high-contrast.json) shows the continuous minimum/wider shell, distinct card/neutral boundary, unchanged accessible target rectangles, and a legible white system-focus ring after actual Tab. The guest exposed inherited background-oriented focus brushes; only the three page-local shell controls now use existing WinUI system focus brushes. Shared card/popup styles and Edge Rail source remain unchanged. Host theme/contrast matches its exact baseline; no host contrast activation.
- Final same-build Light/Dark shell/control-hover/keyboard evidence, all eight pointer-only accent/cursor transitions, actual eight-direction resizing, and type-filtered UIA geometry pass. Full suite: 204 passed, zero failed/skipped, 695 ms. Final parallel review: Standards zero actionable findings; Spec zero actionable source findings.
- [Final cleanup](../../../artifacts/visual-shelf-final-cleanup.json): all five temporary scripts, both guest roots, disposable profiles and unused fixtures are removed without errors. The final isolated demo remains visible and responds to WM_NULL after cleanup; the earlier user-touched demo profile and normal user data are preserved. All changes remain uncommitted.
- Ticket 01's implementation and acceptance are complete. This does **not** complete Ticket 02 or declare Release readiness: broad Edge Rail qualification remains unobserved and the independent canonical residency gate still fails.
