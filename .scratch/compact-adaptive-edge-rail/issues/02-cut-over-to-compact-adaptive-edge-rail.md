# 02: Cut over to the Compact Adaptive Edge Rail

**Parent specification:** [Compact Adaptive Edge Rail](../spec.md)

**What to build:** Replace the old fixed Edge Rail with the approved visible Rail Handle and content-driven expanded shell. Recompose Shelf Batch rows and shell controls, move the already-working Clear Temporary Items action into its final footer position, and apply deterministic centered native-window geometry without changing established focus, drag, flyout, monitor, or fullscreen behavior.

**Blocked by:** 01: Clear Temporary Items from the Edge Rail.

**Status:** needs-info

- [ ] The resting Edge Rail is a plain, theme-aware `16 × 96` logical-pixel Rail Handle centered vertically on the selected monitor work area.
- [ ] The Rail Handle contains no text, count, glyph, badge, or child button and remains the complete direct hover and drag-entry target.
- [ ] Only the two corners facing the desktop use a `16` logical-pixel radius; the two corners against the screen edge remain square on both Left and Right placements.
- [ ] Hover and drag entry retain the existing `200 ms` expansion delay, pointer leave retains the existing `300 ms` collapse delay, and direct interaction preserves the foreground application's focus.
- [ ] The expanded Edge Rail uses a fixed `280` logical-pixel width and remains centered around the Rail Handle's vertical center.
- [ ] Shelf Batch summaries cut over to fixed `64` logical-pixel horizontal rows with representative icon at left, title/subtitle in the flexible middle region, and pinned indication plus Items affordance at right.
- [ ] Single-item and multi-item Shelf Batches use the same row height; long or localized text trims without widening the rail or overlapping controls.
- [ ] The shell uses `4` logical pixels of outer padding, `28` logical-pixel header and footer rows, `4` logical-pixel header/list and list/footer gaps, and `4` logical pixels between adjacent batch rows, with no trailing row gap.
- [ ] One small pure Edge Rail sizing policy is the single source for fixed logical dimensions, the `68n + 68` target-height calculation, the `640` maximum, and grow-versus-hold decisions.
- [ ] One through eight Shelf Batches produce target heights `136`, `204`, `272`, `340`, `408`, `476`, `544`, and `612` logical pixels; nine or more produce `640` logical pixels.
- [ ] Nine or more Shelf Batches scroll vertically inside the `640` maximum, all batches remain reachable, the standard overflow indicator is available, and horizontal scrolling remains disabled.
- [ ] Target dimensions are DPI-scaled and clamped to the selected monitor work area; a short or narrow work area keeps the entire actual window reachable without persisting the clamped dimensions.
- [ ] Rail geometry remains center-anchored: growth moves top and bottom equally around the stable center within physical rounding tolerance. Drop Shelf top-edge growth behavior is not used as a substitute for this rail contract.
- [ ] An accepted drop that creates a Shelf Batch grows the expanded rail immediately when the new target is taller; unsupported, rejected, or empty accepted results do not resize it.
- [ ] Removal, accepted drag-out, Missing cleanup, Clear Temporary Items, and other target-reducing mutations refresh content without live shrink while the rail remains expanded.
- [ ] The next collapse and expansion recalculates the lower target after removals; removal of the final Shelf Item remains the immediate Hidden transition exception.
- [ ] Open Shelf uses a `28 × 28` logical-pixel target and `11` logical-pixel glyph at top-left for a right-docked rail and top-right for a left-docked rail, and retains its existing command meaning.
- [ ] The working Clear Temporary Items control from Ticket 01 moves into the final `28 × 28` bottom-left footer position with an `11` logical-pixel glyph and unchanged behavior.
- [ ] Open Shelf and Clear Temporary Items appear only in the expanded state and retain accessible names, tooltips, keyboard activation, focus visibility, and theme-aware interaction states.
- [ ] Existing multi-item flyout dimensions, desktop-facing placement, scrolling, item actions, deferred item projections, and collapse suppression remain operational.
- [ ] Existing item and batch drag-in/out, Copy-only completion semantics, Temporary/Pinned behavior, Missing cleanup, Unavailable retention, cancellation, and rejection behavior remain operational.
- [ ] Existing monitor/edge settings, remembered-monitor fallback, fullscreen policy, event-driven display/DPI handling, topmost/tool-window styles, no-activate behavior, and reduced-motion handling remain unchanged.
- [ ] Data-driven sizing tests cover every tier, the exact chrome/list accounting, counts above the cap, non-positive counts, grow-after-addition, hold-after-removal, and recalculation on the next expansion.
- [ ] Geometry tests prove centered Left/Right placement, equal top/bottom growth, logical-to-physical scaling, short/narrow work-area clamping, and absence of durable state changes from clamping.
- [ ] Targeted installed Release smoke proves Rail Handle bounds, representative one/eight/nine-batch expanded bounds, complete row fit, vertical scrolling, accepted-drop growth, removal hold, later shrink, final-item hide, Open Shelf, Clear Temporary Items, focus preservation, and flyout hold.
- [ ] No content-measured sizing, render-measure-resize loop, manual rail resizing, persisted rail dimensions, invisible proximity window, cursor polling, reverse-curve flare, custom shadow helper, Mica, Acrylic, new dependency, telemetry, or new UI test framework is introduced.

## Comments

- Source implements 16 × 96 Rail Handle, 280-wide adaptive tiers, grow-only mutation handling, compact 64-pixel rows, mirrored controls and desktop-facing corner resources. Geometry reuses the existing centered, DPI-scaled, work-area-clamped rail positioning rather than Drop Shelf top-anchored growth.
- Release build: zero warnings/errors. Full suite 225/225; subsequently added persistence-failure regression 1/1.
- UI Automation observed Right handle (1904,468,16,96) and Left handle (0,468,16,96). Expanded geometry, row fit, corners, flyout and Clear remain unobserved: pointer hit-test resolves to Windows LockApp PID 8356, not the isolated test process. No production hover workaround.
- Ad-hoc parallel Standards and Spec source reviews returned no findings. These are not the formal fixed-point code-review workflow and do not establish native acceptance. Formal review and installed qualification remain pending; ticket stays open.

### 2026-10-07 — Eight-row viewport fit blocker

- Eight-row viewport fit resolved: ItemsRepeater had truncated realization to 6 rows due to root EffectiveViewport intersection. User approved cutting over to WinUI's stock virtualized ListView (SelectionMode=None, zero-margin ItemContainerStyle, internal ScrollViewer).
- Verified in isolated guest environment (`artifacts/adaptive-rail-8rows-listview.png`, `artifacts/adaptive-rail-8rows-listview.json`): all 8 batch rows (Row0.txt through Row7.txt) realize and display completely within 280×612 bounds without vertical scrolling; foreground focus preserved.
- Release build: 0 warnings/errors. Full automated suite: 226/226 passed.
- Remaining blocker: complete installed qualification across the full matrix (Ticket 03). No commit attempted.
