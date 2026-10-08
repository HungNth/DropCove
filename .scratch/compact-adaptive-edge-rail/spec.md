# Compact Adaptive Edge Rail

Status: ready-for-agent

## Problem Statement

The Edge Rail currently occupies a `64 × 112` logical-pixel resting footprint and expands to a fixed `320 × 640` logical-pixel window after hover. The resting surface is visually heavy for an always-available screen-edge tool, while the fixed expanded height leaves substantial empty space when only a few Shelf Batches exist. The fixed geometry also makes the rail feel unlike the compact edge interaction that motivated it.

The expanded rail uses vertically stacked batch summaries, a large centered Open Shelf control, rectangular outer window geometry, and no Clear Temporary Items control. Users must reopen the Drop Shelf to perform the same shelf-wide temporary-reference cleanup already available there. Adding that control independently would risk duplicating lifecycle, persistence, confirmation, and failure behavior that already belongs to the shared shelf model.

The Edge Rail needs a smaller visible Rail Handle, content-driven expanded height, compact horizontal Shelf Batch rows, rounded desktop-facing corners, edge-aware Open Shelf placement, and Clear Temporary Items at the bottom-left. These changes must preserve no-activate focus behavior, hover timing, drag workflows, flyout lifetime, monitor placement, fullscreen policy, lifecycle semantics, accessibility, and event-driven operation.

## Solution

Replace the current large resting footprint with a visible `16 × 96` logical-pixel Rail Handle centered on the selected monitor work area. The Rail Handle remains a direct hover and drag-entry target, uses the existing theme-aware surface language, contains no text or controls, keeps its screen-edge corners square, and rounds only its two desktop-facing corners by `16` logical pixels.

After the existing `200 ms` hover or drag-entry delay, expand inward to a fixed `280` logical-pixel width. Derive expanded height from Shelf Batch count using fixed `64` logical-pixel horizontal batch rows, `4` logical-pixel gaps, `28` logical-pixel header and footer controls, and `4` logical pixels of outer padding. One through eight Shelf Batches use deterministic heights from `136` through `612` logical pixels. Nine or more use the existing `640` logical-pixel maximum and vertical scrolling. The expanded rail remains vertically centered, grows immediately for newly accepted Shelf Batches, and defers shrink after removal until the next collapse and expansion.

Restyle each rail summary as a compact horizontal row. Place the existing Open Shelf command in the upper corner facing the desktop: top-left for a right-docked rail and top-right for a left-docked rail. Add the existing Clear Temporary Items behavior as a `28 × 28` logical-pixel bottom-left control. Both controls use `11` logical-pixel Fluent glyphs, accessible names, tooltips, visible focus, and existing theme resources.

Keep `DropShelfManager.ClearTemporaryItemsAsync` as the sole lifecycle and persistence mutation seam. The Drop Shelf and Edge Rail reuse that operation and the existing confirmation callback and wording; `RemoveBatchAsync` remains a separate exact-batch operation. Do not add a forwarding coordinator, public interface, generic mutation framework, or rail-specific deletion path.

## User Stories

1. As a Windows user, I want the resting Edge Rail to occupy much less screen space, so that it remains available without visually dominating the desktop.
2. As an Edge Rail user, I want a visible Rail Handle rather than an invisible hot zone, so that I can discover and target DropCove directly.
3. As a pointer user, I want the Rail Handle to be `16` logical pixels wide, so that it is compact but still targetable.
4. As a pointer user, I want the Rail Handle to be `96` logical pixels high, so that hover and drag entry do not require pixel-perfect vertical positioning.
5. As a multi-monitor user, I want the Rail Handle centered on its selected monitor work area, so that its placement remains predictable.
6. As a returning user, I want the remembered monitor and Left or Right edge preserved, so that the redesign does not move my rail unexpectedly.
7. As a Windows user, I want the Rail Handle to use the current theme-aware surface language, so that it remains coherent in Light, Dark, and High Contrast modes.
8. As a Windows user, I want the Rail Handle to remain visually plain, so that a persistent utility does not add unnecessary labels, counts, or decorative icons.
9. As a Windows user, I want the two corners facing the desktop rounded by `16` logical pixels, so that the rail has a softer Windows 11 silhouette.
10. As a Windows user, I want the two corners against the monitor edge to remain square, so that the Rail Handle joins the screen edge without a visual gap.
11. As an Edge Rail user, I want hovering the Rail Handle to expand it after the established `200 ms` delay, so that compactness does not make activation unpredictable.
12. As an Edge Rail user, I want leaving the rail to collapse it after the established `300 ms` delay, so that I have time to correct small pointer movements.
13. As a drag-and-drop user, I want drag entry to trigger the same expansion path as pointer hover, so that I can add files without opening the Drop Shelf.
14. As a Windows user, I want rail expansion to preserve the foreground application's focus, so that inspecting or dragging from the rail does not interrupt my current application.
15. As a Windows user, I want only Open Shelf to activate the full Drop Shelf, so that activation remains an explicit action.
16. As an Edge Rail user, I want the expanded rail to be `280` logical pixels wide, so that it is meaningfully smaller than the current rail while retaining readable summaries.
17. As an Edge Rail user, I want width to remain stable while content changes, so that batch names and controls do not shift horizontally.
18. As an Edge Rail user, I want expanded height based on Shelf Batch count, so that the window matches the summaries actually rendered in its main list.
19. As an Edge Rail user, I do not want total Shelf Item count to enlarge the main rail, so that a multi-item batch still occupies one summary row.
20. As an Edge Rail user, I want one Shelf Batch to open at `136` logical pixels high, so that a small workflow stays compact.
21. As an Edge Rail user, I want each additional Shelf Batch through eight batches to add exactly one compact row and one inter-row gap, so that growth remains predictable.
22. As an Edge Rail user, I want two Shelf Batches to use `204` logical pixels, so that both complete rows are visible.
23. As an Edge Rail user, I want three Shelf Batches to use `272` logical pixels, so that the rail exposes all three summaries without unused fixed-height space.
24. As an Edge Rail user, I want four Shelf Batches to use `340` logical pixels, so that moderate workflows remain fully visible.
25. As an Edge Rail user, I want five Shelf Batches to use `408` logical pixels, so that the rail continues to grow one row at a time.
26. As an Edge Rail user, I want six Shelf Batches to use `476` logical pixels, so that growth remains proportional to visible content.
27. As an Edge Rail user, I want seven Shelf Batches to use `544` logical pixels, so that all seven rows remain directly available.
28. As an Edge Rail user, I want eight Shelf Batches to use `612` logical pixels, so that the last complete pre-overflow state remains smaller than the maximum.
29. As an Edge Rail user, I want nine or more Shelf Batches bounded at `640` logical pixels, so that the rail never grows indefinitely.
30. As an Edge Rail user, I want excess Shelf Batches to scroll vertically, so that all retained batches remain reachable after the height cap.
31. As an Edge Rail user, I do not want horizontal scrolling, so that the rail remains a simple vertical workflow.
32. As an Edge Rail user, I want the standard vertical scroll indicator available when overflow exists, so that additional content is discoverable.
33. As a Windows user, I want the expanded rail to remain vertically centered, so that it grows equally upward and downward from the Rail Handle's center.
34. As a multi-monitor user, I want target dimensions expressed in logical pixels, so that the rail feels consistent across display scales.
35. As a multi-monitor user, I want actual dimensions clamped to the selected monitor work area, so that the rail remains reachable on short or narrow displays.
36. As a multi-monitor user, I want work-area clamping to affect only actual bounds, so that a temporary monitor constraint does not become persistent rail state.
37. As an Edge Rail user, I want a newly accepted Shelf Batch to grow the expanded rail immediately when another row is required, so that the new batch is visible.
38. As an Edge Rail user, I want unsupported or rejected drops to leave rail height unchanged, so that failed input cannot move the window.
39. As an Edge Rail user, I want removal to avoid live shrink while the rail remains expanded, so that controls and drag targets do not move beneath my pointer.
40. As an Edge Rail user, I want the next collapse and expansion to recalculate a smaller target after removals, so that unused space is reclaimed at a stable interaction boundary.
41. As an Edge Rail user, I want removal of the final Shelf Item to hide the rail immediately, so that an empty workspace does not leave a meaningless handle.
42. As a returning user, I want restart or restored Edge Rail display to derive height from current Shelf Batch count, so that stale expanded height is not persisted.
43. As an Edge Rail user, I want every Shelf Batch summary to occupy a fixed `64` logical-pixel horizontal row, so that content density and height calculation agree.
44. As an Edge Rail user, I want each row to show the representative icon at the left, so that file and folder context remains quickly recognizable.
45. As an Edge Rail user, I want title and subtitle text in the center of each row, so that the relevant batch context remains readable.
46. As an Edge Rail user, I want pinned status and the Items affordance at the right, so that secondary state and actions remain available without increasing row height.
47. As an Edge Rail user, I want long names trimmed rather than widening the rail, so that content cannot destabilize window geometry.
48. As a user of localized Windows text, I want row content to tolerate longer translated strings, so that localization does not overlap controls or change the sizing contract.
49. As an Edge Rail user, I want multi-item flyouts to retain their established bounded scrolling, so that Shelf Item inspection remains independent from main rail height.
50. As an Edge Rail user, I want an open flyout to keep the rail expanded, so that its owner does not disappear while I interact with it.
51. As an Edge Rail user, I want flyout placement to continue opening toward the desktop, so that details remain on-screen for both Left and Right rails.
52. As an Edge Rail user, I want the Open Shelf control visually smaller, so that shell chrome does not dominate the compact rail.
53. As an Edge Rail user, I want Open Shelf to retain a `28 × 28` logical-pixel interaction rectangle and an `11` logical-pixel glyph, so that visual compactness does not make it too difficult to target.
54. As a right-edge user, I want Open Shelf at the top-left of the expanded rail, so that it sits in the upper corner facing the desktop.
55. As a left-edge user, I want Open Shelf at the top-right of the expanded rail, so that its placement mirrors the selected edge.
56. As an Edge Rail user, I want Open Shelf to retain its current command meaning, so that the icon does not become an ambiguous manual expand toggle.
57. As an Edge Rail user, I want Clear Temporary Items at the bottom-left, so that cleanup aligns with the Drop Shelf's established control placement.
58. As an Edge Rail user, I want Clear Temporary Items to use a `28 × 28` logical-pixel interaction rectangle and an `11` logical-pixel trash glyph, so that the footer remains compact and consistent.
59. As an assistive-technology user, I want Open Shelf and Clear Temporary Items to expose stable accessible names and help text, so that icon-only controls remain understandable.
60. As a keyboard user, I want shell controls to retain a logical tab order and visible focus treatment, so that the rail remains operable without a pointer.
61. As a High Contrast user, I want controls, rows, outlines, and focus cues to resolve through system resources, so that the redesign remains legible.
62. As a DropCove user, I want Clear Temporary Items on the Edge Rail to mean the same thing as it does on the Drop Shelf, so that the same command never changes scope by surface.
63. As a DropCove user, I want Clear Temporary Items to remove every Temporary Item reference across the shelf, so that I can clean short-lived content in one action.
64. As a DropCove user, I want every Pinned Item preserved by Clear Temporary Items, so that reusable references survive cleanup.
65. As a cautious user, I want the existing confirmation before Clear Temporary Items, so that an accidental trash-button press does not remove many references immediately.
66. As a Windows user, I want the confirmation to state that source files on disk are not touched, so that cleanup cannot be mistaken for filesystem deletion.
67. As a DropCove user, I want canceling confirmation to leave every Shelf Batch and Shelf Item unchanged, so that dismissal is safe.
68. As a DropCove user, I want clearing a mixed Shelf Batch to retain its Pinned Items and remove only its Temporary Items, so that lifecycle remains item-owned.
69. As a DropCove user, I want empty Shelf Batches pruned after their Temporary Items are cleared, so that the rail never displays empty containers.
70. As a DropCove user, I want a persistence failure during Clear Temporary Items to preserve current in-memory and durable state, so that a failed cleanup cannot partially remove references.
71. As a DropCove user, I want a clear failure reported through the currently active surface, so that the action never fails silently.
72. As a maintainer, I want the Drop Shelf and Edge Rail to reuse `DropShelfManager` lifecycle behavior, so that pinned, temporary, empty-batch, hidden-state, and persistence rules cannot diverge.
73. As a maintainer, I want Remove Batch to remain a separate exact-batch operation, so that shelf-wide lifecycle cleanup is not conflated with removing one workflow context.
74. As a maintainer, I do not want a rail-specific deletion path or forwarding coordinator, so that reuse does not add speculative indirection.
75. As a drag-and-drop user, I want item and whole-batch drag-out behavior unchanged, so that the rail redesign does not alter Copy-only lifecycle semantics.
76. As a drag-and-drop user, I want accepted drag-out to consume participating Temporary Items and retain participating Pinned Items exactly as before, so that visual changes cannot weaken lifecycle rules.
77. As a drag-and-drop user, I want cancellation, rejection, and failure to retain valid references, so that native interoperability remains safe.
78. As a user with Missing or Unavailable Items, I want current availability classification and cleanup behavior unchanged, so that adaptive sizing does not reinterpret path state.
79. As a fullscreen user, I want the current hide-over-fullscreen policy and its setting preserved, so that the rail does not unexpectedly cover games or presentations.
80. As a performance-conscious user, I want sizing recalculated only at content mutation and rail state boundaries, so that adaptive behavior adds no polling or idle work.
81. As a performance-conscious user, I want large collections to retain virtualization and deferred flyout item projections, so that compact rows do not materialize every Shelf Item.
82. As a performance-conscious user, I want the existing `100` Shelf Batch and `1,000` Shelf Item interaction gate preserved, so that the redesign remains responsive at the established scale.
83. As a performance-conscious user, I want the existing native residency target to remain visible and unwaived, so that visual improvement cannot hide the unresolved release blocker.
84. As a reduced-motion user, I want existing animation preference handling preserved, so that non-essential rail transitions remain disabled when Windows requests it.
85. As a maintainer, I want actual installed Release evidence for window geometry, focus, scrolling, corners, and controls, so that XAML declarations are not mistaken for observed behavior.
86. As a maintainer, I want automated tests focused on sizing decisions and lifecycle outcomes, so that refactoring presentation code does not break tests tied to private structure.
87. As a maintainer, I want Light, Dark, High Contrast, Left, Right, DPI, and work-area limitations reported only when exercised, so that qualification does not fabricate coverage.
88. As a maintainer, I want product documentation updated after implementation, so that users and future agents see the new Rail Handle and Adaptive Rail Sizing contract.

## Implementation Decisions

- This specification supersedes the prior Edge Rail geometry contract of `64 × 112` logical pixels at rest and fixed `320 × 640` logical pixels when expanded. It also supersedes later specifications that froze those dimensions or declared Edge Rail geometry, controls, or visual treatment unchanged. It does not reopen the established hover timing, focus, monitor, fullscreen, flyout, or drag contracts.
- The Edge Rail has two presentation states:
  - **Rail Handle**: a visible `16 × 96` logical-pixel resting surface.
  - **Expanded Edge Rail**: a `280` logical-pixel-wide surface whose target height derives from Shelf Batch count.
- The Rail Handle remains the complete hover and drag-entry hit zone. No separate proximity sensor, invisible trigger window, full-display-height transparent window, or continuous cursor polling is introduced.
- Both Rail Handle and expanded states remain centered vertically in the selected monitor work area. Expansion grows inward horizontally from the configured screen edge and equally upward and downward around the same vertical center.
- Target dimensions are expressed in logical pixels. Native actual bounds are DPI-scaled and then clamped to the selected monitor work area through the existing window-placement path.
- If a work area is narrower than the requested logical width after scaling, actual width clamps to the work-area width while retaining the selected edge. If a work area is shorter than the requested target height, actual height clamps to the work-area height and remains centered. Work-area reachability wins over nominal target size.
- Work-area clamping is transient. Edge Rail dimensions are not persisted, do not alter Drop Shelf preferred width or Manual Height Override, and do not create a new settings record.
- The Rail Handle uses the existing theme-aware layer fill and a subtle theme-aware stroke. It contains no glyph, label, batch count, badge, or button.
- Only the two corners facing the desktop use a `16` logical-pixel radius. A right-docked rail rounds top-left and bottom-left; a left-docked rail rounds top-right and bottom-right. The two screen-edge corners remain square.
- Selective corner treatment retains the existing WinUI surface and theme resources. Following installed observation that XAML radius alone leaves a rectangular HWND, the user approved an Edge Rail-only native window region to clip the two desktop-facing corners. Region geometry is DPI-scaled, reapplied only when size/radius/edge changes, and transferred to Windows ownership on successful application. This does not alter Drop Shelf DWM rounding or add reverse-curve flares, bespoke assets, Mica, Acrylic, a custom shadow window, or an Electron-style transparent window architecture. Visible installed output remains the acceptance authority.
- The expanded shell uses `4` logical pixels of outer padding, one `28` logical-pixel header control row, a `4` logical-pixel header-to-list gap, a vertical list of fixed `64` logical-pixel Shelf Batch rows with `4` logical pixels between adjacent rows, a `4` logical-pixel list-to-footer gap, and one `28` logical-pixel footer control row.
- For `n >= 1` Shelf Batches, list extent is `64n + 4(n - 1)`. Fixed outer padding and shell chrome consume `72` logical pixels: `4` top padding, `28` header, `4` header/list gap, `4` list/footer gap, `28` footer, and `4` bottom padding. There is no trailing inter-card gap. Total target height is therefore `72 + 64n + 4(n - 1) = 68n + 68` logical pixels before the maximum is applied.
- The target-height tiers are explicit product constants derived from that accounting:

| Shelf Batch count | Target height |
| ---: | ---: |
| 1 | `136` |
| 2 | `204` |
| 3 | `272` |
| 4 | `340` |
| 5 | `408` |
| 6 | `476` |
| 7 | `544` |
| 8 | `612` |
| 9 or more | `640` maximum |

- The formula is `min(640, 68 × ShelfBatchCount + 68)` for every non-empty rail. No zero-batch tier exists because removing the final Shelf Item transitions the shelf to Hidden and removes the rail.
- At the `640` logical-pixel maximum, fixed shell chrome consumes `72` logical pixels and the batch viewport receives `568` logical pixels before any smaller-work-area clamp. Overflow scrolls vertically; horizontal scrolling remains disabled.
- Target height is deterministic. The implementation does not measure rendered text, card `DesiredSize`, or content width to resize the native window, and it does not introduce a render-measure-resize feedback loop.
- One small pure Edge Rail sizing policy is permitted to own the fixed logical dimensions, target-height calculation, maximum, and grow-versus-hold decision. It should follow the existing pure Drop Shelf sizing-policy precedent rather than placing duplicated constants in XAML, native interop, and tests.
- While the Edge Rail is collapsed, restored, or newly shown, target height derives from current Shelf Batch count before the next expansion.
- While expanded, an accepted drop that creates a Shelf Batch recalculates target height and grows immediately if the new target exceeds current logical height. Rejected, unsupported, or empty accepted results do not resize the rail.
- While expanded, Shelf Batch or Shelf Item removal, accepted drag-out, Missing cleanup, Clear Temporary Items, or any other mutation that reduces the derived target does not shrink the window. The batch list refreshes inside current bounds. The next collapse and expansion recalculates and may open shorter.
- Final-item removal remains the exception to the no-live-shrink rule: the lifecycle manager transitions to Hidden and the Edge Rail disappears immediately.
- Shelf Batch rows cut over to a fixed `64` logical-pixel horizontal layout. The representative file/folder icon is left-aligned; title and subtitle occupy the flexible middle region; pinned indication and the multi-item Items affordance occupy the right region.
- Row content uses trimming and localization-safe layout rather than changing rail width. Single-item and multi-item summaries share the same fixed row height. The Items affordance is absent for single-item batches without leaving a second height variant.
- The rail batch list uses a clean virtualized collection layout. Following proven truncation of ItemsRepeater's EffectiveViewport to 176×312 (realizing only 6 rows), the user approved cutting over to WinUI's stock virtualized ListView with SelectionMode=None, transparent unpadded ItemContainerStyle, and internal ScrollViewer ownership. Eight complete 64px rows fit within the 612px window (4px padding, 28px header, 4px gap, 8 × 64px rows + 7 × 4px gaps, 4px gap, 28px footer, 4px padding = 612px). Full Shelf Item projections remain deferred until a multi-item flyout requests them.
- The existing multi-item flyout dimensions, item rows, placement direction, vertical scrolling, drag actions, and open-state collapse suppression remain unchanged unless a later specification deliberately replaces them.
- Open Shelf remains the existing command; it is not repurposed as a manual rail expand/collapse toggle. It uses a `28 × 28` logical-pixel target and `11` logical-pixel Fluent glyph.
- For a right-docked rail, Open Shelf occupies the top-left position. For a left-docked rail, it occupies the top-right position. In both cases it is the upper control facing the desktop rather than the screen edge.
- Clear Temporary Items uses a `28 × 28` logical-pixel target and `11` logical-pixel trash glyph at the bottom-left in both Left and Right rail configurations.
- Open Shelf and Clear Temporary Items appear only in the expanded state. The Rail Handle contains no interactive child controls.
- Both shell controls use existing theme resources for normal, hover, pressed, disabled, focus, and High Contrast states. They retain accessible names, tooltips, keyboard activation, and visible focus.
- **Clear Temporary Items** retains its canonical domain meaning: a confirmed shelf-wide lifecycle action that removes every Temporary Item reference while preserving every Pinned Item and never deleting filesystem objects.
- `DropShelfManager.ClearTemporaryItemsAsync` remains the sole mutation seam for Clear Temporary Items. It continues to own mutation serialization, database-first persistence, in-memory state replacement, empty-batch pruning, final-empty Hidden transition, and sizing reset behavior.
- `DropShelfManager.RemoveBatchAsync` remains a distinct exact-batch operation that removes pinned and temporary references in the selected Shelf Batch. It is not called by the rail footer and is not generalized together with Clear Temporary Items or accepted-drag cleanup.
- The Drop Shelf and Edge Rail invoke the same manager operation through their existing manager ownership and use the existing confirmation callback. Confirmation title and body come from one shared application source so wording cannot drift between surfaces:
  - Title: `Clear Temporary Items`
  - Body: `Remove all temporary item references from DropCove?\n\nSource files on disk will not be touched.`
- No new `ShelfCleanupCoordinator`, public deletion interface, generic mutation command bus, compatibility wrapper, or pass-through module is introduced. If a small private callback is needed to avoid duplicate confirmation orchestration, it remains an application-local implementation detail rather than a new architectural seam.
- Each presentation retains its existing refresh responsibility after successful mutation. The Drop Shelf refreshes cards, popup ownership, status, and dismissal state. The Edge Rail uses its existing mutation-refresh callback to update summaries or hide when no batches remain.
- Expected persistence failure leaves manager memory and durable storage unchanged because durable mutation precedes in-memory replacement. The active presentation reports failure through its established status surface or resident notification path; failure is never converted into a success count.
- Clearing all-Temporary content hides the rail immediately through the existing final-empty lifecycle. Clearing mixed or partially pinned content refreshes the remaining batches inside the current expanded bounds and defers geometric shrink until the next expansion.
- Current no-activate native window styles, foreground focus preservation, topmost/tool-window behavior, `200 ms` expansion delay, `300 ms` collapse delay, flyout hold rule, event-driven display/DPI reflow, monitor and edge settings, fullscreen policy, and setting persistence remain unchanged.
- Current drag-in, item drag-out, batch drag-out, Copy-only operations, partial-availability confirmation, Missing cleanup, Unavailable retention, accepted-drag consumption, and cancellation/rejection behavior remain unchanged.
- Current animation preference handling remains authoritative. Existing rail transition motion may adapt to the new bounds, but no new spring engine, polling animation driver, or Edge-Drop asset is introduced.
- No database schema, App Settings schema, Shelf Batch model, Shelf Item model, lifecycle state, placement state, or Drop Shelf sizing state changes.
- Existing dependencies, the unpackaged self-contained WinUI 3 architecture, and the CLI toolchain remain sufficient. No new NuGet or runtime dependency is introduced.
- User-facing product documentation is updated after implementation to describe the Rail Handle, Adaptive Rail Sizing tiers, compact batch rows, control placement, rounded inward corners, scrolling cap, and Clear Temporary Items behavior.

## Testing Decisions

- Good tests assert consumer-visible decisions and state transitions: requested logical size, grow-versus-hold behavior, actual installed bounds, row fit, overflow, control semantics, lifecycle results, focus preservation, and error outcomes. They do not assert private fields, XAML source text, element names, event forwarding, resource-key plumbing, or duplicated literal values outside the canonical sizing policy.
- One small pure Edge Rail sizing-policy seam is the automated authority for decision-rich sizing behavior. Data-driven tests cover every `136/204/272/340/408/476/544/612/640` tier, the `68n + 68` formula, one through eight complete rows, the nine-batch cap, no zero-batch target, grow-after-addition, hold-after-removal, and recalculation on the next expansion.
- Sizing-policy tests cover invalid or impossible inputs explicitly rather than relying on presentation behavior. A non-positive Shelf Batch count cannot produce a visible target; counts above the cap always return `640` logical pixels without overflow or arithmetic wraparound.
- Existing logical-to-physical scaling and native window-placement tests remain prior art for DPI conversion and work-area constraints. New lower-level abstractions are not added solely to mock native calls.
- Installed self-contained Release behavior is the highest seam for actual Edge Rail geometry because only the running WinUI/native window can prove selective corners, theme rendering, physical bounds, no-activate behavior, pointer timing, UI Automation rectangles, scrolling, and focus.
- Installed smoke verifies the `16 × 96` Rail Handle on both Left and Right edges at the exercised DPI and confirms it is vertically centered in the selected work area.
- Installed smoke verifies one batch expands to `280 × 136`, eight batches to `280 × 612`, and nine or more batches to `280 × 640` before work-area clamping.
- Installed smoke verifies the center point remains stable across Rail Handle, growth, collapse, and re-expansion within physical rounding tolerance at the exercised DPI.
- Installed smoke verifies a work area shorter or narrower than the nominal target clamps actual bounds completely inside the work area. Where the environment cannot supply such a work area, pure geometry evidence may prove the arithmetic while installed clamp behavior remains explicitly unobserved.
- Installed smoke verifies eight complete `64`-pixel rows fit at `612`, nine batches use bounded vertical scrolling at `640`, wheel or equivalent scroll reaches the final batch and returns to the first, and horizontal scrolling remains disabled.
- Installed smoke verifies an accepted drop while expanded grows by one tier, an unsupported or rejected drop does not resize, a removal refreshes content without live shrink, and the next collapse/expansion opens at the lower derived tier.
- Installed smoke verifies final-item removal hides the Edge Rail rather than displaying a zero-batch Rail Handle.
- Installed visual evidence verifies compact horizontal row hierarchy, representative icon, title/subtitle trimming, pinned indication, right-side Items affordance, and fixed height for both single-item and multi-item batches.
- Installed Light and Dark evidence uses the same qualified build to verify theme-aware Rail Handle and expanded shell surfaces, subtle boundary, two rounded desktop-facing corners, two square screen-edge corners, and readable controls.
- High Contrast evidence is gathered only in an isolated or safely reversible environment. It verifies shell separation, row hierarchy, control glyphs, accessible focus, and scroll discoverability. Unavailable High Contrast execution remains unobserved rather than inferred.
- UI Automation verifies Open Shelf and Clear Temporary Items accessible names, help text where present, enabled/focusable state, logical `28 × 28` interaction rectangles at the exercised DPI, and edge-dependent Open Shelf placement.
- Keyboard smoke verifies logical focus order, visible focus, Open Shelf activation, Clear confirmation, cancellation, confirmation acceptance, and no focus trap in the batch flyout.
- Pointer smoke verifies `200 ms` expansion and `300 ms` collapse using continuous movement, not teleport-only input. It verifies drag entry expands the rail and direct interaction does not activate or steal focus from the foreground application.
- Flyout smoke explicitly verifies the full lifecycle that earlier qualification did not complete: opening a real multi-item flyout holds the expanded rail, item rows remain observable and actionable, pointer movement between rail and flyout does not collapse the owner, and closing or completing drag restores normal collapse behavior.
- Existing `DropShelfManager` tests remain the highest automated lifecycle seam for Clear Temporary Items. Coverage includes all-Temporary content, all-Pinned no-op, mixed items within a batch, multiple batches, empty-batch pruning, final-empty Hidden transition, and unchanged source paths.
- Real temporary SQLite tests remain the persistence authority. They verify successful clear survives restart, pinned references remain, final-empty sizing state resets, and deterministic transaction failure leaves both the current manager and reopened database unchanged.
- Clear Temporary Items presentation wiring is proven through installed behavior, not tests that merely verify a button forwarded a call. Drop Shelf and Edge Rail smoke use identical confirmation wording, exercise cancel and accept, and observe the correct remaining Shelf Items.
- Rail clear smoke verifies all-Temporary content hides the rail, mixed content retains Pinned Items and refreshes without live shrink, and persistence failure reports an error without removing visible references.
- Existing drag lifecycle tests remain authoritative for Temporary, Pinned, mixed, Missing, Unavailable, accepted Copy, cancellation, rejection, and failure semantics. They are not duplicated as Edge Rail-specific manager tests.
- Installed rail drag smoke covers one-item drag, whole-batch drag, item drag from a multi-item flyout, cancellation, accepted Temporary consumption, Pinned retention, focus preservation, and coherent flyout closure.
- Settings and placement regression coverage verifies Left/Right and monitor selection round-trip, fullscreen visibility policy, remembered-monitor fallback, display/DPI event-driven repositioning, and no new rail-size setting.
- Reduced-motion smoke verifies state changes and resizing remain functional when non-essential Composition animation is disabled.
- Performance smoke retains deferred flyout projections, bounded batch realization, responsive scrolling, pointer entry, drag, and collapse/expand transitions with `100` Shelf Batches and `1,000` Shelf Items.
- The established release gates remain unchanged: idle CPU at or below `0.1%`, shelf-show latency p95 at or below `150 ms`, responsive pointer/scroll/drag interaction, and visible/post-dismissal total process working set below `160 MB`. This specification does not waive or claim to solve the current native residency blocker.
- UI Automation and residency measurement run separately against the same Release payload identity so automation-peer realization cannot contaminate memory evidence.
- No screenshot-baseline framework is introduced. Screenshots and captured bounds are qualification evidence for human review, not pixel-perfect permanent tests tied to OS rendering revisions.
- Build success or passing unit tests alone are insufficient. Final acceptance requires an installed self-contained Release run that exercises the changed Edge Rail surface and records exact payload identity, environment, bounds, interactions, unobserved scenarios, and cleanup.

## Out of Scope

- An invisible `3`-pixel Edge-Drop-style hot zone, a full-work-area transparent input window, global cursor polling, or proximity detection outside the visible Rail Handle.
- Changing the established `200 ms` expansion delay or `300 ms` collapse delay.
- User-configurable Rail Handle size, expanded width, height tiers, card height, row gap, corner radius, or hover timing.
- Content-measured width, text-measured height, arbitrary size-to-content behavior, or render-measure-resize feedback loops.
- Manual resizing of the Edge Rail, a persisted rail size, per-monitor rail dimensions, or a rail equivalent of Manual Height Override.
- Expanding width according to content, adding horizontal scrolling, or changing the `280` logical-pixel expanded width.
- Showing more than eight complete rows before the established `640` logical-pixel cap.
- Changing Drop Shelf preferred width, Automatic Shelf Growth, Manual Height Override, responsive columns, row tiers, or no-live-shrink behavior.
- Rounding all four Edge Rail corners, adding reverse-curve flares, custom assets, Mica, Acrylic, custom shadows, or a separate decorative helper window.
- Replacing Open Shelf with a manual rail expand/collapse toggle or adding a second expand-lock control.
- Adding Settings, Pin/Unpin, Remove Item, Remove Batch, Bulk Pinning, search, filters, arbitrary selection, or file-launch actions to the rail shell.
- Changing multi-item flyout dimensions, item layout, drag actions, or ownership semantics beyond preserving its established lifecycle under the new shell geometry.
- Changing Clear Temporary Items to remove Pinned Items, remove only visible batches, remove a selected batch, bypass confirmation, or delete source filesystem objects.
- Generalizing Remove Item, Remove Batch, Clear Temporary Items, accepted-drag cleanup, and Missing cleanup into one deletion-scope abstraction.
- Adding a `ShelfCleanupCoordinator`, command bus, repository interface, compatibility wrapper, or other forwarding layer without a second implementation that requires a real seam.
- Changing database schema, App Settings schema, shelf lifecycle states, monitor-placement persistence, or migration behavior.
- Changing no-activate behavior, topmost policy, fullscreen policy, drag effects, availability semantics, or source filesystem ownership.
- Adding telemetry, background polling, new runtime dependencies, a new UI test framework, or a screenshot-regression framework.
- Solving, lowering, replacing, or waiving the independent native residency release gate.
- Supporting operating systems, architectures, or packaging models outside the existing product targets.

## Further Notes

- `CONTEXT.md` defines **Edge Rail**, **Rail Handle**, **Adaptive Rail Sizing**, **Shelf Batch**, **Shelf Item**, **Temporary Item**, **Pinned Item**, and **Clear Temporary Items**. The specification uses those canonical terms and avoids “sidebar,” “launcher,” “hidden hot zone,” “collapsed shelf,” “fixed rail size,” “delete all,” and “clear batches.”
- Primary-source Edge-Drop comparison research is recorded in `.scratch/research/edge-drop-edge-rail.md`. Edge-Drop is a behavioral reference, not a numeric or architectural authority: it uses a fixed viewport and transparent clipped window rather than DropCove's adaptive native-window geometry.
- The exact adaptive-height accounting is intentional: `72` logical pixels of fixed shell chrome plus `64n + 4(n - 1)` logical pixels of list extent yields `68n + 68`. There is no trailing row gap. Work-area clamping occurs only after logical target selection and DPI scaling.
- The historical `64 × 112` / `320 × 640` contract remains valid evidence of prior behavior but is no longer the target after this specification. Earlier qualification marked incomplete or release-unqualified is not reclassified as acceptance evidence for the redesign.
- No ADR is required. The feature is a reversible product interaction and presentation contract, keeps existing lifecycle and native-window ownership, introduces no durable schema or cross-context boundary, and records its trade-offs directly in this specification.
- All specification and glossary changes remain uncommitted until the user explicitly authorizes a commit.
