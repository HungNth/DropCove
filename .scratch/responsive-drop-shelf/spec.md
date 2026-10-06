# Responsive Resizable Drop Shelf Specification

Status: ready-for-agent

> **Superseded in part:** [Automatic Shelf Growth](../automatic-shelf-growth/spec.md) replaces the requirements that Shelf Batch additions never resize the visible shelf, that all content mutations preserve stable bounds, that pointer resizing is the only window-level expansion behavior, and that automatic content-driven growth is out of scope. All other responsive layout, popup, resizing, persistence, Edge Rail, accessibility, and performance requirements remain governing unless the newer specification explicitly changes them.

## Problem Statement

The Drop Shelf currently exposes separate Compact and Expanded presentations with fixed window sizes and footer controls for switching between them. This makes the surface feel like a mode-driven management window rather than a lightweight temporary drag-and-drop shelf. Users cannot shape the shelf to match their current workspace, monitor, or number of Shelf Batches.

Inline expansion of multi-item Shelf Batches also couples item management to the Drop Shelf's window geometry. Opening a batch consumes shelf space and moves surrounding content, while the fixed Compact and Expanded sizes force users into layouts chosen by the application.

The Drop Shelf needs one continuous, modern presentation that users can resize directly. Content must respond to the chosen width without introducing another hidden mode, and detailed multi-item management must remain available without changing the shelf's preferred size.

## Solution

Replace Compact and Expanded presentations with one resizable Drop Shelf. The user can resize the borderless window from every edge and corner, down to a fixed minimum of `180 × 180` logical pixels and up to the current monitor work area. The latest user-selected logical size becomes the preferred size while the shelf holds content and survives hide/show, monitor changes, and application restart.

Shelf Batches use a responsive wrapping grid. Every card keeps the same compact information and actions; increasing the window width only changes how many cards fit in each row. Multi-item Shelf Batches open a non-modal anchored popup for item details and actions instead of expanding items inline. The popup can extend beyond the Drop Shelf but remains inside the current work area.

Remove the window-level Expand/Compact controls and all corresponding presentation state. Preserve the existing Edge Rail, Path Reference, Shelf Batch, Shelf Item, drag-in, drag-out, pinning, removal, persistence, accessibility, and performance contracts except where this specification explicitly changes presentation behavior.

## User Stories

1. As a Windows user, I want to resize the Drop Shelf directly, so that it fits the workspace I am using now.
2. As a Windows user, I want to resize from any edge or corner, so that I do not have to target one special handle.
3. As a Windows user, I want the resize target to be discoverable on hover, so that the borderless window does not hide an essential interaction.
4. As a Windows user, I want resizing to preserve the window's opposite edge or corner naturally, so that it behaves like a normal desktop window.
5. As a Windows user, I want a minimum shelf size, so that controls and drag targets remain usable.
6. As a Windows user, I want the shelf limited to the active monitor's work area, so that content and controls do not become unreachable.
7. As a multi-monitor user, I want resizing and restoration to respect the current monitor, so that a size chosen elsewhere cannot place the shelf outside the visible work area.
8. As a high-DPI user, I want preferred sizing expressed in logical pixels, so that the shelf feels physically consistent across DPI changes.
9. As a Windows user, I want the shelf to remember my chosen size while it contains items, so that I do not have to resize it repeatedly.
10. As a Windows user, I want my preferred size restored after hiding and reopening the shelf, so that dismissal does not discard my layout choice.
11. As a Windows user, I want my preferred size restored after restarting DropCove, so that the layout remains stable across sessions.
12. As a multi-monitor user, I want one preferred size shared across monitors, so that the shelf does not maintain surprising monitor-specific layouts.
13. As a multi-monitor user, I want temporary work-area clamping not to overwrite my preferred size, so that returning to a larger monitor restores the size I selected.
14. As a Windows user, I want the first shelf opening to use `180 × 180` logical pixels, so that the initial surface remains compact.
15. As a Windows user, I want removing the final Shelf Item from either the visible Drop Shelf or Edge Rail to hide the shelf and reset its preferred size, so that no empty surface or stale layout survives the workflow.
16. As a Windows user, I want adding a Shelf Batch not to resize the window, so that content changes do not override my explicit sizing decision.
17. As a Windows user, I want removing a Shelf Batch not to resize the window while content remains, so that management actions do not move the surface unexpectedly.
18. As a Windows user, I want opening item details not to resize the window, so that inspection does not disturb the shelf layout.
19. As a Windows user, I want one Drop Shelf presentation, so that I do not need to understand Compact and Expanded modes.
20. As a Windows user, I want the footer Expand/Compact control removed, so that window resizing is the only window-level expansion model.
21. As a Windows user, I want Clear Temporary Items to remain available, so that removing the mode switch does not remove bulk cleanup.
22. As a Windows user, I want Shelf Batch cards to retain one consistent design at every shelf size, so that resizing does not silently switch interfaces.
23. As a Windows user, I want wider shelves to show more batch columns, so that additional space increases visible capacity.
24. As a Windows user, I want cards to stretch evenly within a row, so that the shelf uses available width without ragged unused space.
25. As a Windows user, I want Shelf Batches ordered left-to-right and then top-to-bottom, so that newest-first ordering remains predictable in a grid.
26. As a Windows user, I want vertical scrolling when batches exceed the available height, so that all batches remain reachable without changing the window size.
27. As a Windows user, I want no horizontal scrolling, so that resizing does not create a two-axis navigation problem.
28. As a Windows user, I want every card to remain at least `164` logical pixels wide, so that its primary actions remain usable.
29. As a Windows user, I want exact responsive column boundaries, so that small resize movements produce deterministic layouts.
30. As a user with a single-item Shelf Batch, I want direct drag, Pin/Unpin, and Remove actions on the card, so that the quickest workflow remains immediate.
31. As a user with a single-item Shelf Batch, I do not want an unnecessary item popup trigger, so that the card remains simple.
32. As a user with a multi-item Shelf Batch, I want its chevron to open an item popup, so that I can inspect and manage the batch without expanding the shelf content inline.
33. As a Windows user, I want the popup anchored to its Shelf Batch, so that the relationship between summary and details is clear.
34. As a Windows user, I want the popup allowed to extend outside the Drop Shelf, so that the shelf can remain compact while details have usable space.
35. As a Windows user, I want the popup kept inside the monitor work area, so that its controls remain reachable.
36. As a Windows user, I want only one batch popup open at a time, so that transient management surfaces do not accumulate.
37. As a Windows user, I want opening another batch to replace the current popup, so that changing context takes one action.
38. As a pointer user, I want the same chevron to open and close the popup, so that the interaction is reversible and obvious.
39. As a pointer user, I want clicking outside the popup to close it, so that dismissal follows normal non-modal behavior.
40. As a keyboard user, I want `Enter` or `Space` on the chevron to open the popup, so that item management does not require a pointer.
41. As a keyboard user, I want focus moved into a keyboard-opened popup, so that I can act on its first item immediately.
42. As a keyboard user, I want `Tab` to traverse all popup item actions, so that every management action is reachable.
43. As a keyboard user, I want `Esc` to close the popup and restore focus to its chevron, so that transient context has predictable focus behavior.
44. As a keyboard user, I want a second `Esc` to dismiss the Drop Shelf after the popup closes, so that popup dismissal takes precedence over shelf dismissal.
45. As a Windows user, I want the popup closed when the shelf hides, its batch is deleted, or resizing starts, so that it never remains detached from its owner.
46. As a Windows user, I want popup width to adapt to content within bounded limits, so that short content is compact and long content remains readable.
47. As a Windows user, I want long popup lists to scroll vertically, so that large batches remain manageable.
48. As a Windows user, I want long names and paths truncated visually but preserved in tooltips and accessibility names, so that layout remains bounded without losing information.
49. As a Windows user, I want each popup item to show its icon, name, path, availability, and pinned state, so that I can identify and evaluate it before acting.
50. As a Windows user, I want to Pin/Unpin and Remove individual items from the popup, so that current management capabilities remain available.
51. As a Windows user, I want to drag an individual item from the popup, so that multi-item batches remain useful as drag sources.
52. As a Windows user, I want to drag the whole batch from its card, so that batch drag-out remains a direct operation.
53. As a Windows user, I want starting a whole-batch drag to close its popup, so that the transient surface cannot obscure or outlive the drag source.
54. As a Windows user, I want an individual item drag not to close the popup automatically, so that I can continue managing the remaining items.
55. As a Windows user, I want canceled item drags to leave references unchanged, so that cancellation remains safe.
56. As a Windows user, I want a successful drag-out to preserve existing Temporary and Pinned Item semantics, so that resizing does not change lifecycle rules.
57. As a Windows user, I want the popup updated or closed when drag-out removes items or its batch, so that it never displays stale state.
58. As an Edge Rail user, I want rail docking, delayed hover expansion, scrolling, and drag interactions unchanged, so that this redesign affects only the Drop Shelf.
59. As a shake-to-open user, I want successful, canceled, and unsupported drag outcomes to preserve their current state restoration behavior, so that the new visible state does not regress invocation workflows.
60. As an accessibility user, I want named, focusable controls and visible focus indicators throughout the responsive shelf and popup, so that all actions remain operable with assistive technology.
61. As a reduced-motion user, I want resize and popup behavior to respect Windows animation preferences, so that the redesign does not reintroduce unwanted motion.
62. As a performance-conscious user, I want responsive resizing and scrolling with 100 Shelf Batches and 1,000 Shelf Items, so that dynamic layout does not block pointer interaction.
63. As a performance-conscious user, I want deferred item materialization and virtualization retained, so that unopened multi-item batches do not retain full item view models.
64. As a performance-conscious user, I want idle CPU to remain at or below `0.1%`, so that the resident utility remains effectively idle.
65. As a performance-conscious user, I want visible and post-dismissal working set to remain below `150 MB`, so that the more flexible UI remains lightweight.
66. As a Windows user, I want the shelf-show latency target to remain at or below `150 ms` p95, so that restoring preferred geometry still feels immediate.

## Implementation Decisions

- The Drop Shelf display model is a clean cutover from `Hidden` / `Compact` / `Expanded` / `EdgeDocked` to `Hidden` / one visible Drop Shelf state / `EdgeDocked`. Compact and Expanded are removed rather than retained as aliases, compatibility paths, or hidden internal modes.
- The current Compact and Expanded surfaces, their window-level toggle controls, and their fixed-size selection logic are removed. The completed historical Compact/Expanded ticket remains historical evidence; this specification supersedes that presentation contract.
- The Drop Shelf window supports native pointer resizing from every edge and corner without aspect-ratio locking. The borderless resize hit target is approximately `8` logical pixels around the window perimeter. A subtle resize affordance appears on relevant edge or corner hover without permanently adding heavy chrome.
- The minimum preferred Drop Shelf size is `180 × 180` logical pixels. There is no application-defined maximum below the current monitor work area. If a work area is smaller than the minimum, keeping the full window reachable takes precedence over the minimum.
- Resizing never permits the actual window bounds to leave the current monitor work area. DPI conversion and work-area clamping use the monitor containing the shelf during the resize.
- The system distinguishes **preferred logical size** from **actual bounds**. Preferred logical size records the user's unclamped choice. Actual bounds are the temporary DPI-scaled and work-area-clamped geometry used on the current monitor.
- One preferred logical size is shared across all monitors. Moving to a smaller work area may clamp actual bounds but must not overwrite the preferred size. Returning to a sufficiently large work area restores the preferred size.
- The first opening uses `180 × 180`. A preferred size is valid only when both logical dimensions are integers of at least `180`; there is no stored application-defined maximum.
- A completed user-initiated resize updates the preferred logical size. It is persisted while at least one Shelf Item exists; if the user resizes an empty visible shelf, accepting the first item makes that current preference durable. Programmatic restoration, DPI conversion, work-area clamping, positioning, content reflow, and popup changes never overwrite it.
- Preferred-size persistence uses a dedicated additive singleton settings record containing logical width and height. Absence of the record means `180 × 180`. Existing Shelf Batch, Shelf Item, and Edge Rail placement schemas and records are not repurposed or rewritten.
- Startup restores the preferred-size record only when persisted Shelf Items remain. An empty restored shelf deletes or ignores any stale preferred-size record and opens at `180 × 180` when next summoned.
- Every mutation that removes the final Shelf Item—including item or batch removal, Clear Temporary Items, accepted drag-out, and Missing cleanup—transitions the manager from either the visible Drop Shelf or `EdgeDocked` to `Hidden`, closes any popup, and deletes the preferred-size record as part of the persisted state change. Remembered Edge Rail placement remains intact.
- Adding batches, removing batches or items while content remains, opening or closing a popup, and changing item state do not automatically resize the Drop Shelf. Content reflows or scrolls inside the user-selected bounds.
- Shelf Batches use one responsive wrapping grid and one card presentation. There is no content-driven breakpoint that switches to a different card design or reveals a former Expanded presentation.
- The content surface keeps `8` logical pixels of horizontal padding on each side. A card has a minimum width of `164` logical pixels and adjacent columns have a `4` logical-pixel gap. Cards stretch evenly to consume the usable row width.
- The column count is the largest positive integer `n` for which `n × 164 + (n - 1) × 4` fits within the window width minus `16` logical pixels of horizontal padding. This produces one column for widths `180–347`, two for `348–515`, three for `516–683`, and continues by the same formula.
- Shelf Batches remain newest-first and fill left-to-right, then top-to-bottom. Overflow scrolls vertically only; horizontal scrolling is disabled.
- Every card retains the compact information density and primary actions. Width changes column count and available text width, not card semantics or action availability.
- A single-item Shelf Batch has no popup trigger. It retains direct whole-card item drag, Pin/Unpin, and Remove actions.
- A multi-item Shelf Batch uses its existing chevron location as a popup trigger. Inline item-list expansion and collapse are removed from the Drop Shelf.
- The multi-item popup is non-modal, anchored to its batch card, and may extend outside the Drop Shelf. It remains inside the monitor work area with at least `16` logical pixels of margin where the work area permits.
- Popup placement preference is right of the card, then left, then below, then above, followed by work-area clamping.
- Popup width is size-to-content between `320` and `480` logical pixels. Popup height is size-to-content up to `480` logical pixels. A work area too small for these minimums may override them to keep the popup reachable.
- Popup content scrolls vertically after reaching the height limit. Horizontal scrolling is disabled. Long visible text uses truncation while tooltips and accessibility names retain the complete value.
- Only one multi-item popup may be open. Activating another batch replaces the current popup.
- The popup opens through pointer activation or `Enter` / `Space` on the chevron. It closes through the same chevron, outside click, or `Esc`.
- When opened from the keyboard, focus enters the first available item or action. `Tab` traverses popup controls. Closing returns focus to the originating chevron when that control still exists.
- `Esc` closes an open popup before it can dismiss the Drop Shelf. A subsequent `Esc` follows the existing shelf-dismissal behavior.
- Hiding the shelf, deleting the owning batch, beginning a shelf resize, or starting whole-batch drag closes the popup. Opening or using Settings must not leave an orphaned popup.
- Popup rows display icon or thumbnail, name, path, availability, and pinned state. They preserve individual Pin/Unpin, Remove Item, and drag-out actions. Remove Batch and whole-batch drag remain on the card; Clear Temporary Items remains in the footer.
- Starting an individual item drag does not close the popup. Canceled, rejected, or failed drags preserve existing references. Successful drag-out applies the existing Temporary/Pinned lifecycle and updates the popup or closes it if its batch no longer exists.
- Full item view models and visuals remain deferred until a popup opens. Closing a popup, hiding the shelf, or recycling its owner releases popup item projections and visible visual requests. The compact card keeps only bounded preview data.
- The Edge Rail remains a separate surface with its current collapsed/expanded hover geometry and behavior. Removing Drop Shelf Compact/Expanded terminology must not remove or rename Edge Rail hover expansion.
- Existing drag-in, drag-out, Path Reference, Shelf Batch, Shelf Item, persistence, Missing/Unavailable classification, pinning, cleanup, single-instance, hotkey, shake, tray, fullscreen, and same-integrity contracts remain unchanged.
- Existing motion and accessibility behavior is adapted to the single responsive surface and popup. No resize animation may lag behind the pointer or block drag interaction.
- No standalone UI-mock architecture is introduced solely for testing. Decision-rich geometry calculations may be exposed as pure logic when that keeps Win32 and WinUI event code thin; control forwarding and duplicated constants are not test seams.

## Testing Decisions

- Good permanent tests assert user-observable state transitions, persistence boundaries, geometry invariants, and lifecycle semantics. They do not assert XAML source text, event forwarding, duplicate constants, template structure, or mock echoes.
- The existing application seam around the shelf manager remains the highest automated state seam. Current tests that explicitly require Compact or Expanded are replaced with the single visible Drop Shelf contract rather than preserved.
- Application-seam tests verify `Hidden`, visible Drop Shelf, and `EdgeDocked` transitions; empty versus non-empty dismissal; final-item removal from both visible and Edge Rail states; popup closure; and unchanged drag/pin/remove semantics. Compact/Expanded assertions are replaced, not retained.
- Real temporary SQLite tests verify the additive preferred-size settings record without changing Shelf Batch, Shelf Item, or Edge Rail placement data. They cover default absence, completed user-resize save, first-item persistence after resizing an empty shelf, restart restoration only while content exists, deletion with the final persisted item, stale-record cleanup for an empty database, and preservation of Edge Rail placement.
- Preferred-size tests assert the persisted logical preference independently from actual clamped window bounds. Programmatic positioning, DPI conversion, restoration, and small-monitor clamping must not rewrite the record.
- If implementation exposes pure decision-rich geometry or layout calculations, focused tests cover minimum sizing, logical-to-physical DPI conversion, preferred-versus-actual work-area clamping, and exact column boundaries at `347/348`, `515/516`, and subsequent formula boundaries. Existing native interop scaling tests are prior art. The implementation must not create a lower-level seam only to test wiring.
- Existing visual-coordinator tests remain the prior art for deferred icon/thumbnail requests and cancellation. Popup materialization must retain the same lifecycle guarantees without duplicating tests for provider forwarding.
- Installed self-contained Release smoke is the acceptance authority for behavior that core tests cannot prove: borderless edge/corner hit testing, cursor/hover affordances, live native resizing, WinUI wrapping and virtualization, popup placement outside the shelf, focus restoration, outside-click dismissal, real drag initiation, and monitor/DPI work-area behavior.
- Installed Release smoke covers resizing from all edges and corners; `180 × 180` minimum behavior; large-size work-area limits; temporary clamping without preferred-size loss; hide/show and restart restoration; and reset after the final item transitions the shelf to Hidden.
- Installed Release smoke covers one/two/three/four-column boundaries, even card stretching, newest-first row order, vertical-only scrolling, stable window bounds when content changes, and absence of Compact/Expanded controls.
- Installed Release smoke covers single-item direct actions and the complete multi-item popup lifecycle: pointer and keyboard opening, one-at-a-time replacement, placement fallback, bounded sizing, vertical overflow, outside click, `Esc` precedence, focus return, resize closure, batch deletion, Pin/Unpin, item removal, item drag, and whole-batch drag.
- The repository's documented installed-EXE/UI Automation smoke procedures are prior art. No standalone smoke harness currently exists for this surface, so manual and UI Automation observations must record the exercised scenario and actual result without claiming unobserved multi-monitor or DPI cases.
- The existing installed Release performance gate remains unchanged. With `100` Shelf Batches and `1,000` Shelf Items, visible and post-dismissal `WorkingSet64` must each remain below `150 MB`, idle CPU must remain at or below `0.1%`, pointer/scroll/drag interaction must remain responsive, and shelf-show latency p95 must remain at or below `150 ms`.
- The verified pre-change baseline is approximately `134.57 MB` visible, `141.05 MB` post-dismissal, and `0.00%` idle CPU. Regression against the acceptance limits blocks completion even if functional smoke passes.

## Out of Scope

- Resizing or redesigning the Edge Rail.
- Changing Edge Rail delayed hover expansion, collapse timing, docking, fullscreen policy, focus behavior, or settings.
- Restoring Compact/Expanded through aliases, hidden breakpoints, automatic mode switches, or compatibility shims.
- Arbitrary cross-batch selection, file launching, file-manager behavior, clipboard management, or file storage.
- Automatic size-to-content growth or shrinkage after drops, removals, or popup actions.
- Per-monitor preferred sizes.
- User-configurable minimum size, grid gap, card width, popup size, or resize-border thickness.
- Horizontal batch scrolling.
- Multiple simultaneous batch popups, modal item management, detachable management windows, or a separate management page.
- A popup for single-item batches.
- Changing drag-out lifecycle semantics, filesystem ownership, database corruption recovery, or same-integrity limitations.
- New telemetry, background polling, network traffic, or periodic cursor inspection.

## Further Notes

- This specification supersedes the Drop Shelf presentation portions of the V1 specification and the completed Compact/Expanded shelf-mode contract. It does not rewrite historical completion records.
- `CONTEXT.md` already removes the obsolete `Bounded Unified Drop Shelf` term. No new glossary term is required: this design remains the `Drop Shelf`.
- Dropover and Yoink provide product-level precedent for lightweight transient shelves, while Holdem's reviewed implementation does not expose freeform resizing. The exact responsive resize behavior here is a DropCove product decision, not a claim of identical competitor behavior.
- The implementation must preserve the established WinUI 3 CLI toolchain and existing dependencies unless a concrete requirement proves them insufficient.
- All repository changes remain uncommitted until the user explicitly authorizes a commit.
