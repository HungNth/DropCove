# Automatic Shelf Growth

Status: ready-for-agent

## Problem Statement

The responsive Drop Shelf preserves one fixed user-selected window size while content changes. When the shelf starts at its compact `180 × 180` logical-pixel size, adding a small number of Shelf Batches immediately pushes content into vertical scrolling even though bounded height growth would expose those batches with less interaction.

Earlier DropCove behavior progressively increased shelf height as content accumulated. Removing that behavior made manual resizing predictable, but it also made users repeatedly resize the shelf for a common short-lived workflow. Restoring the old batch-count rule directly would conflict with the current responsive grid because wider shelves can display several Shelf Batches in one row.

The Drop Shelf needs bounded, row-aware Automatic Shelf Growth without taking control away from users. Automatic height must remain transient and derived from current content, while an explicit vertical resize must become a durable Manual Height Override. Width selection, responsive columns, work-area clamping, persistence, Edge Rail behavior, and existing drag lifecycle semantics must remain coherent.

## Solution

Add Automatic Shelf Growth to the existing responsive Drop Shelf. While no Manual Height Override applies, the visible shelf derives its target height from the number of responsive-grid rows required at its current actual width. It uses four fixed logical-height tiers: `180`, `236`, `292`, and `348` for one, two, three, and four-or-more rows. Content beyond four rows scrolls vertically.

Automatic growth runs after an accepted drop creates a Shelf Batch and whenever the Drop Shelf is shown or restored. It changes height only when the derived target is taller than the currently visible shelf. Removing content never shrinks the shelf while it remains visible; the next show or restart recalculates the derived height from the remaining content.

A completed user resize that changes logical height creates a Manual Height Override. That override suspends Automatic Shelf Growth, survives hide/show and restart while Shelf Items remain, and is removed when the final Shelf Item is removed. A width-only resize remains durable but does not create a height override. Automatic height is never persisted as though the user selected it.

## User Stories

1. As a Windows user, I want the Drop Shelf to grow automatically when new Shelf Batches require another grid row, so that recently added content is visible without an immediate manual resize.
2. As a Windows user, I want automatic growth based on rendered grid rows rather than raw Shelf Batch count, so that wider shelves do not grow vertically when existing columns already have capacity.
3. As a Windows user, I want a one-row shelf to remain `180` logical pixels high, so that the initial surface stays compact.
4. As a Windows user, I want a two-row shelf to grow to `236` logical pixels high, so that the second row is visible with the established amount of surrounding space.
5. As a Windows user, I want a three-row shelf to grow to `292` logical pixels high, so that the third row is visible without manual adjustment.
6. As a Windows user, I want a four-row shelf to grow to `348` logical pixels high, so that short multi-batch workflows remain visible.
7. As a Windows user, I want content beyond four rows to scroll vertically, so that the Drop Shelf never grows indefinitely.
8. As a Windows user, I want automatic growth to leave shelf width unchanged, so that the application does not rearrange my desktop horizontally.
9. As a Windows user, I want the current responsive column boundaries retained, so that my chosen width continues to determine how many Shelf Batches fit in each row.
10. As a Windows user, I want a new Shelf Batch that fills an existing row to leave the shelf height unchanged, so that the window moves only when visible capacity actually requires another row.
11. As a Windows user, I want a new Shelf Batch that starts a new row to increase shelf height immediately, so that the newest batch remains visible after the drop.
12. As a Windows user, I want accepted drops to return the batch viewport to its newest-first position, so that the newly created Shelf Batch is not hidden by prior scrolling.
13. As a Windows user, I want unsupported or rejected drops to leave shelf height unchanged, so that failed input cannot move the window.
14. As a Windows user, I want adding content while the shelf is hidden or in the Edge Rail to defer sizing until the Drop Shelf opens, so that a non-visible window is not moved unnecessarily.
15. As a Windows user, I want opening a non-empty Drop Shelf without a Manual Height Override to derive height from its current Shelf Batches, so that it opens at a useful compact size.
16. As a Windows user, I want restarting DropCove without a Manual Height Override to derive height again from restored Shelf Batches, so that transient automatic height does not become stale preference data.
17. As a Windows user, I want removing or consuming Shelf Batches to leave the visible shelf height stable, so that controls and drag targets do not move beneath the pointer.
18. As a Windows user, I want the next hide/show after removals to recalculate automatic height, so that unused vertical space is reclaimed without movement during the active interaction.
19. As a Windows user, I want a restarted shelf to recalculate automatic height from the remaining content, so that it does not reopen at a height justified only by removed Shelf Batches.
20. As a Windows user, I want deleting the final Shelf Item to hide the shelf and reset sizing state, so that the next workflow starts from the compact default.
21. As a Windows user, I want to resize the Drop Shelf from every existing edge and corner, so that Automatic Shelf Growth does not replace normal desktop-window control.
22. As a Windows user, I want a completed resize that changes height to create a Manual Height Override, so that the application respects my explicit vertical sizing decision.
23. As a Windows user, I want a corner resize that changes height to create a Manual Height Override, so that diagonal resizing has the same authority as resizing a horizontal edge.
24. As a Windows user, I want a resize that changes width but not height to keep Automatic Shelf Growth enabled, so that choosing a different column layout does not silently lock vertical behavior.
25. As a Windows user, I want width-only resizing to leave height unchanged during the resize, so that dragging a side edge behaves like a normal desktop window.
26. As a Windows user, I want narrowing the shelf to reflow or scroll without live automatic height changes, so that the bottom edge does not move while I drag a side edge.
27. As a Windows user, I want the next accepted Shelf Batch after a width-only resize to use the new row count, so that later automatic growth reflects the layout I chose.
28. As a Windows user, I want a Manual Height Override to survive hiding and reopening the Drop Shelf, so that dismissal does not discard my explicit height.
29. As a Windows user, I want a Manual Height Override to survive restarting DropCove while Shelf Items remain, so that I do not have to repeat the same resize.
30. As a Windows user, I want new Shelf Batches to scroll inside a manually sized shelf instead of overriding my selected height, so that the application never fights my explicit choice.
31. As a Windows user, I want my selected width to survive hide/show and restart while Shelf Items remain, so that Automatic Shelf Growth does not weaken the existing width preference.
32. As a Windows user, I want automatic height kept separate from my selected width and Manual Height Override, so that a temporary content peak cannot become a permanent large shelf.
33. As a Windows user, I want resizing an empty visible shelf to apply to the first subsequent accepted Shelf Batch in that running instance, so that I can prepare the shelf before dropping content.
34. As a Windows user, I want an empty-shelf size choice to become durable only after the first Shelf Item is accepted, so that empty transient window state is not written as retained workspace state.
35. As a Windows user, I want restarting DropCove before an empty shelf accepts content to restore `180 × 180`, so that an empty pending choice does not survive as persisted state.
36. As an existing DropCove user, I want a valid persisted preferred size associated with retained Shelf Items treated as an explicit manual size after the feature update, so that the cutover does not reinterpret my existing shelf geometry.
37. As a multi-monitor user, I want automatic target heights expressed in logical pixels, so that the growth tiers feel consistent across display scales.
38. As a multi-monitor user, I want actual bounds clamped to the active monitor work area, so that automatic growth cannot place controls outside the reachable desktop.
39. As a multi-monitor user, I want work-area clamping to remain temporary, so that monitor constraints do not create or overwrite a Manual Height Override.
40. As a Windows user, I want automatic growth to keep the top edge fixed when space permits, so that the shelf expands downward like the earlier bounded behavior.
41. As a Windows user, I want the shelf shifted upward only when required to stay inside the work area, so that reachability takes precedence near the bottom of a monitor.
42. As a Windows user, I want overflow to remain vertical-only when the work area cannot fit an automatic target height, so that the feature does not create horizontal navigation.
43. As a Windows user, I want an open multi-item popup closed before automatic window growth, so that the popup cannot become detached from its Shelf Batch.
44. As an Edge Rail user, I want rail dimensions, hover expansion, placement, scrolling, and drag behavior unchanged, so that Automatic Shelf Growth affects only the Drop Shelf.
45. As a keyboard or assistive-technology user, I want focus, automation names, and keyboard interaction preserved through automatic growth, so that the feature does not interrupt accessible workflows.
46. As a reduced-motion user, I want automatic growth to respect Windows motion preferences, so that the restored behavior does not introduce unwanted animation.
47. As a performance-conscious user, I want growth calculations to occur only at accepted-drop and show/restore event boundaries, so that the feature adds no polling or idle CPU cost.
48. As a performance-conscious user, I want responsive scrolling and drag interaction retained at the existing large-data acceptance scale, so that automatic sizing does not regress the shelf's responsiveness or memory limits.

## Implementation Decisions

- This specification supersedes the Responsive Drop Shelf requirements that prohibit automatic resizing after Shelf Batch additions, require stable visible bounds for every content mutation, make pointer resizing the only expansion model, and list automatic content-driven growth as out of scope. It does not restore Compact or Expanded presentation modes.
- The sizing model distinguishes four concepts:
  - **Preferred width**: the user-selected logical width, shared across monitors and durable while Shelf Items remain.
  - **Manual Height Override**: an optional user-selected logical height that suspends Automatic Shelf Growth and is durable while Shelf Items remain.
  - **Automatic target height**: a transient logical height derived from current responsive-grid rows when no Manual Height Override exists.
  - **Actual bounds**: the DPI-scaled, work-area-clamped window geometry used on the current monitor.
- The current single preferred width/height pair is replaced by one canonical sizing-state interface that can represent preferred width independently from an optional Manual Height Override. Automatic target height is never stored in that durable state.
- The persistence schema is cut over to one canonical singleton sizing record that stores preferred width and optional manual height semantics. No parallel legacy and new persistence paths, compatibility aliases, or fallback readers remain after the cutover.
- The current schema writes a preferred-size record with the first accepted Shelf Batch and cannot identify which axis the user later resized. The confirmed cutover rule does not guess: any valid existing preferred-size record associated with retained Shelf Items preserves its width and becomes a Manual Height Override at its stored height. Existing users enter Automatic Shelf Growth after that content lifecycle reaches empty and resets sizing state.
- A width-only completed user resize persists preferred width when content exists and preserves the current Manual Height Override state: it neither creates nor clears one. A completed user resize creates or replaces Manual Height Override only when final logical height differs from the resize-start logical height.
- While at least one Shelf Item remains, programmatic restoration, Automatic Shelf Growth, DPI conversion, monitor work-area clamping, positioning, responsive reflow, popup changes, and content removal never create a Manual Height Override or overwrite preferred width. Final-item removal is the explicit reset exception defined below.
- The first opening remains `180 × 180` logical pixels. The existing `180 × 180` minimum and current-monitor work-area maximum remain unchanged, including the rule that full reachability wins when a work area is smaller than the minimum.
- Resizing an empty visible shelf records a pending preferred width and, when height changed, a pending Manual Height Override in memory. The first subsequently accepted Shelf Item persists that pending user choice atomically with the accepted Shelf Batch. Restart before content acceptance discards the pending choice and returns to `180 × 180`.
- Removing the final Shelf Item from either the visible Drop Shelf or Edge Rail hides the shelf, deletes the durable sizing record, clears preferred width and Manual Height Override to defaults, and preserves remembered Edge Rail placement.
- Automatic row count uses the same actual content width and responsive-column rule as the rendered Shelf Batch grid. With `16` logical pixels of total horizontal content padding, `164` logical pixels minimum card width, and `4` logical pixels between columns, column count remains the largest positive integer whose cards and gaps fit the actual content width.
- Automatic row count is `ceil(Shelf Batch count / column count)`. Shelf Item count inside a Shelf Batch does not affect row count because each Shelf Batch occupies one card.
- The automatic logical-height tiers are product constants, not measurements taken after rendering: zero or one row uses `180`, two rows use `236`, three rows use `292`, and four or more rows use `348`. The existing card layout must continue to fit one through four complete rows at those tiers.
- Four rows are the automatic-growth cap. Additional rows use the existing vertical ScrollViewer; horizontal scrolling remains disabled.
- On accepted creation of a Shelf Batch while the Drop Shelf is visible and no Manual Height Override exists, the application recalculates rows after the batch is available to the responsive layout. It grows only when the derived tier exceeds the current visible logical height.
- Filling unused capacity in an existing row does not resize the window. Rejected or unsupported drops do not resize the window.
- Content accepted while the shelf is Hidden or EdgeDocked changes durable content only. Automatic target height is calculated when the Drop Shelf next becomes visible.
- On show or startup restore, preferred width is resolved and clamped first. If a Manual Height Override exists, that height is restored and clamped. Otherwise, row count is derived from the resulting actual width and the corresponding automatic height tier is applied.
- Automatic height does not shrink during one visible lifetime. Shelf Batch or Shelf Item removal, successful drag-out, Missing cleanup, Clear Temporary Items, popup activity, pin changes, and responsive reflow retain current bounds while content remains.
- A later show or restart begins a new visible lifetime. Without a Manual Height Override, it recalculates the tier from current content and may therefore open shorter than the prior visible lifetime.
- Width-only native resize reflows cards and scrolling but never invokes vertical Automatic Shelf Growth during or immediately after that resize. A later accepted Shelf Batch uses the new actual width when calculating rows.
- Native resize direction and start/end logical bounds are retained for the completed resize decision. Top, bottom, and corner resizing creates a Manual Height Override only when logical height actually changes; left or right resizing alone updates preferred width only.
- Automatic growth keeps the current top edge fixed when the target fits. If the target would leave the active work area, the shelf moves upward only as far as required and clamps actual height when necessary. Clamping never changes durable sizing state.
- An open multi-item popup closes before programmatic shelf growth, preserving the existing rule that a popup cannot outlive or detach from its owner during a bounds change.
- Automatic growth returns the Shelf Batch viewport to newest-first position after an accepted drop, preserving the existing drop-refresh behavior.
- Automatic growth is event-driven. It introduces no timers, cursor polling, background layout polling, or repeated render measurement.
- The existing Drop Shelf manager remains the highest state seam for content lifecycle and durable sizing semantics. The window module remains responsible for applying DPI-aware actual bounds. The responsive layout remains responsible for columns and card reflow.
- One small pure sizing-policy seam may centralize decision-rich calculations shared by the window and tests: column count, row count, automatic tier, grow-versus-hold decision, and top-anchored work-area target. It must not duplicate XAML rendering or expose a broad window-management interface.
- Existing dependencies and the WinUI 3 CLI toolchain remain sufficient. No package or new UI mode is introduced.

## Testing Decisions

- Good tests assert consumer-visible state transitions and geometry decisions: which height is selected, whether a resize is automatic or manual, what persists, when the shelf holds its current bounds, and how work-area clamping affects actual bounds. Tests must not assert private fields, source text, event forwarding, XAML element names, or persistence calls echoed through mocks.
- The Drop Shelf manager remains the highest automated state seam. Manager tests cover preferred width, optional Manual Height Override, final-item reset, pending empty-shelf choices, accepted-drop transitions, and absence of automatic-state persistence.
- Real temporary SQLite tests cover the canonical sizing record. They verify width-only persistence without creating or clearing Manual Height Override, manual-height persistence, restart restoration, first-item atomic persistence after empty-shelf resize, deletion after final-item removal, stale empty-record cleanup, preservation of Edge Rail placement, and schema cutover of every valid existing preferred-size record with retained content into a manual choice.
- One pure sizing-policy seam covers decision-rich layout behavior without constructing WinUI controls. Data-driven tests cover current responsive column boundaries, row calculation at multiple widths, the `180/236/292/348` tiers, four-row cap, existing-row fills, rejected drops, grow-only behavior within one visible lifetime, and recalculation on the next show.
- Pure geometry tests cover logical-to-physical scaling, top-edge anchoring, upward correction near the work-area bottom, actual-height clamping, work areas smaller than the minimum, and the invariant that clamping never changes durable preference state.
- Resize-completion tests cover axis semantics: left/right width changes update preferred width while preserving the current Manual Height Override state; top/bottom and corner changes create or replace an override only when logical height changes; programmatic growth is never classified as user resize.
- Automated tests do not reproduce layout constants in several modules. The sizing policy is the single decision source, while installed UI evidence confirms that the rendered card rows actually fit the product tiers.
- Existing application-seam and persistence tests are prior art. Existing native resize, DPI, work-area, and popup-placement tests are prior art for geometry. Tests whose only purpose is to require stable bounds after accepted Shelf Batch additions are replaced by the new hybrid contract rather than retained.
- Installed self-contained Release smoke is the acceptance authority for WinUI and native-window behavior that automated seams cannot prove. The smoke must exercise real Explorer/Desktop drops and observe actual window bounds, responsive columns, scrolling, focus, popup closure, pointer resizing, and rendered row fit.
- Installed Release smoke at one-column width verifies one row at `180`, two at `236`, three at `292`, four at `348`, and five or more remaining at `348` with vertical scrolling.
- Installed Release smoke at wider responsive breakpoints verifies that batches filling an existing row do not grow the shelf and that only a newly required row advances to the next height tier.
- Installed Release smoke verifies width-only resize with no vertical jump, later row-aware growth using the new width, vertical and corner resize creating Manual Height Override, and subsequent accepted drops respecting that override.
- Installed Release smoke verifies removal without live shrink, hide/show recalculation without an override, restart recalculation without an override, and exact restoration with an override.
- Installed Release smoke verifies empty-shelf resize remains pending only in the running instance, first accepted content makes it durable, final-item removal resets all sizing state, and restart before first content returns to `180 × 180`.
- Installed Release smoke verifies top-edge anchoring where space permits and reachable work-area clamping near a monitor edge. Multi-monitor and non-100% DPI observations are reported only when actually exercised.
- Existing accessibility checks verify that focus remains usable after growth and that closing an open popup for growth does not strand keyboard focus. Reduced-motion behavior is verified against the running app.
- The existing `100` Shelf Batch / `1,000` Shelf Item installed Release performance gate remains unchanged: visible and post-dismissal working set stay below `160 MB`, idle CPU stays at or below `0.1%`, pointer/scroll/drag interaction remains responsive, and shelf-show latency p95 stays at or below `150 ms`.

## Out of Scope

- Automatic width changes or aspect-ratio locking.
- Live vertical resizing caused by dragging only the left or right shelf edge.
- Immediate automatic shrink after Shelf Batch or Shelf Item removal while the shelf remains visible.
- More than four automatically visible rows or monitor-percentage-based row caps.
- User-configurable growth tiers, row cap, card dimensions, grid gaps, or minimum shelf size.
- Render-measure-resize feedback loops or arbitrary size-to-content behavior.
- Restoring Compact/Expanded modes, hidden presentation breakpoints, or a manual/automatic mode toggle.
- A command to clear Manual Height Override while Shelf Items remain; the override ends when the final Shelf Item is removed.
- Per-monitor preferred widths or Manual Height Overrides.
- Changes to Edge Rail geometry, hover expansion, placement, scrolling, or flyouts.
- Changes to multi-item popup content, dimensions, placement order, or item-management semantics beyond closing it before shelf growth.
- Changes to Path Reference, Shelf Batch, Shelf Item, Temporary Item, Pinned Item, Missing Item, Unavailable Item, or Successful Drag-Out semantics.
- New telemetry, polling, background measurement, dependencies, or development toolchain requirements.
- A broader motion or visual-style redesign.

## Further Notes

- `CONTEXT.md` defines **Automatic Shelf Growth** and **Manual Height Override** as the canonical product terms. Avoid describing the behavior as a restored Compact mode, size-to-content mode, or locked size.
- The historical heights `180/236/292/348` are an explicit product contract selected for this feature. They are not inferred from current card height or row spacing. Current layout evidence shows that one through four complete responsive rows fit within those tiers; future visual changes must preserve that acceptance or deliberately revise this specification.
- The current implementation already treats an empty-shelf resize as an in-memory pending choice that becomes durable with the first accepted item and is lost on restart before content. Final-item removal resets in-memory preferred size and deletes the persisted record. This specification preserves that lifecycle while splitting width preference from optional manual height semantics.
- Existing databases cannot distinguish default first-acceptance writes, width-only resizes, and explicit height choices. The confirmed cutover deliberately preserves every valid existing record with retained content as a manual size rather than guessing intent; Automatic Shelf Growth begins for those users after the shelf next becomes empty.
- No ADR is required. The change is a reversible product interaction contract recorded in this specification rather than a system-wide architectural decision.
- All repository changes remain uncommitted until the user explicitly authorizes a commit.
