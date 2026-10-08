# Edge Rail Thumbnail and Management Parity

Status: ready-for-agent

## Problem Statement

The expanded Edge Rail currently presents each Shelf Batch as a glyph-and-text summary. It does not show the image thumbnails or bounded multi-item previews already available on the Drop Shelf. Its Manage Items flyout is also a separate drag-only presentation: it shows coarse file/folder glyphs and pinned indicators, but not the Drop Shelf's thumbnail, path, availability, lifecycle, Bulk Pinning, Pin/Unpin, or Remove Item experience.

This difference makes the same Shelf Batch look and behave like two unrelated objects depending on whether the Drop Shelf or Edge Rail is visible. Users must open the full Drop Shelf to pin, unpin, inspect, or remove references even though the Edge Rail is intended to remain directly usable. The duplicate presentation paths also create a maintenance risk: adding thumbnails and management independently to the Edge Rail would repeat visual-projection, cancellation, fallback, and resource-lifetime logic that already exists for the Drop Shelf.

The redesign must make the two surfaces coherent without turning the compact Edge Rail into a full Drop Shelf card, changing the established no-activate interaction, or increasing resident native resources without bounds. DropCove's total-process Working Set remains subject to the existing strict `<160 MB` release gate at 100 Shelf Batches and 1,000 Shelf Items, so thumbnail parity must preserve deferred realization and release hidden visual resources.

## Solution

Give the expanded Edge Rail the same content hierarchy and management semantics as the Drop Shelf while preserving the Edge Rail's compact layout and window behavior.

Each fixed-height Shelf Batch row uses the Drop Shelf's bounded preview language: a single item shows one 42-pixel visual, while a multi-item Shelf Batch stacks visuals for at most its first three Shelf Items. Eligible images use Windows Shell thumbnails; folders, non-image files, unavailable thumbnails, and failed thumbnail requests use the existing native icon fallback. The row uses the same title and subtitle projection as the Drop Shelf and exposes the same always-visible 24-pixel actions: Pin/Unpin and Remove Item for a single-item Shelf Batch; tri-state Bulk Pinning, Manage Items, and Remove Batch for a multi-item Shelf Batch.

Manage Items opens a desktop-facing, work-area-clamped flyout using the Drop Shelf's item-management content contract. It includes a fixed Bulk Pinning header and virtualized item rows with thumbnail/native icon, name, Path Reference, availability, lifecycle state, Pin/Unpin, Remove Item, and direct drag-out. Item removal keeps the flyout open while at least two Shelf Items remain, closes it when the Shelf Batch becomes a single-item row, and removes the Shelf Batch or hides the Edge Rail when no content remains.

Reuse shared presentation primitives, projections, styles, and the existing visual coordinator/provider pipeline. Keep Drop Shelf and Edge Rail shells, layout ownership, popup/flyout ownership, and event handlers surface-specific. Both surfaces continue to call the existing DropShelfManager and DragDropService seams directly; no forwarding coordinator, public mutation interface, generic command framework, or rail-specific lifecycle implementation is introduced.

Thumbnail requests remain lazy and realization-bound. The Edge Rail cancels and releases row visuals when it collapses and releases item visuals when the Manage Items flyout closes. It retains no application-wide thumbnail cache or persisted thumbnail bytes. Pin and Remove mutations use durable manager operations before presentation refresh, disable the affected action while pending, report failure through the existing resident notification path, and never modify source filesystem objects.

## User Stories

1. As an Edge Rail user, I want Shelf Batch rows to use the same visual language as Drop Shelf cards, so that the same held content is immediately recognizable on both surfaces.
2. As an Edge Rail user, I want a single-item Shelf Batch to show its image thumbnail when available, so that I can identify visual assets without reading the filename.
3. As an Edge Rail user, I want a non-image file to show its Windows native icon, so that every supported reference has a useful visual.
4. As an Edge Rail user, I want a folder to show its Windows native folder icon, so that folders remain distinguishable from files.
5. As an Edge Rail user, I want thumbnail failure to fall back to the native icon, so that a failed Shell request never leaves a blank row.
6. As an Edge Rail user, I want a multi-item Shelf Batch to show a bounded stacked preview, so that I can recognize that it contains several items.
7. As an Edge Rail user, I want the stacked preview to use at most the first three Shelf Items, so that large batches do not create unbounded visual work.
8. As a Drop Shelf user, I want the preview ordering to match the Shelf Batch's item ordering, so that both surfaces represent the same content consistently.
9. As an Edge Rail user, I want each preview to retain the existing rounded card treatment and theme-aware boundary, so that thumbnails remain legible over Light, Dark, and High Contrast surfaces.
10. As an Edge Rail user, I want row visuals to update asynchronously without blocking pointer interaction, so that expanding the rail remains responsive.
11. As an Edge Rail user, I want the native icon fallback available while image content cannot be resolved, so that visual loading never makes content unusable.
12. As an Edge Rail user, I want a single-item row title to be the Shelf Item name, so that the primary identity matches the Drop Shelf.
13. As an Edge Rail user, I want a single-item row subtitle to show its type and pinned lifecycle when relevant, so that essential state is visible without opening another surface.
14. As an Edge Rail user, I want a multi-item row title to show the Shelf Item count, so that batch size is clear.
15. As an Edge Rail user, I want a multi-item row subtitle to show the first two names and pinned count when relevant, so that the row provides the same bounded context as the Drop Shelf.
16. As an Edge Rail user, I want long or localized text trimmed within the row, so that content never changes rail width or overlaps actions.
17. As an Edge Rail user, I want the full text available through existing tooltip and accessibility patterns, so that trimming does not discard information.
18. As an Edge Rail user, I want a single-item row to expose Pin/Unpin directly, so that I do not need to open the full Drop Shelf for a common lifecycle action.
19. As an Edge Rail user, I want a single-item row to expose Remove Item directly, so that I can remove an unwanted reference in place.
20. As an Edge Rail user, I want a multi-item row to expose Bulk Pinning directly, so that I can pin or unpin the entire Shelf Batch efficiently.
21. As an Edge Rail user, I want a multi-item row to expose Manage Items directly, so that I can inspect and manage individual references without opening the full Drop Shelf.
22. As an Edge Rail user, I want a multi-item row to expose Remove Batch directly, so that I can remove one complete workflow context in place.
23. As an Edge Rail user, I want row actions ordered Pin, Manage Items, and Remove Batch for multi-item content, so that they match the Drop Shelf interaction pattern.
24. As an Edge Rail user, I want row actions to remain visible rather than appearing only on hover, so that management features are discoverable and stable.
25. As an Edge Rail user, I want each row action to retain a 24 by 24 logical-pixel interaction rectangle, so that compactness does not reduce the established target size.
26. As an Edge Rail user, I want Remove Item and Remove Batch to use the same `×` affordance as the Drop Shelf, so that the action is not confused with shelf-wide Clear Temporary Items.
27. As a DropCove user, I want the trash glyph to remain reserved for Clear Temporary Items, so that removal scope is visually distinct.
28. As a DropCove user, I want Remove Item to remove only one Shelf Item reference, so that other items in the Shelf Batch remain intact.
29. As a DropCove user, I want Remove Batch to remove every reference in the selected Shelf Batch, so that its scope matches the Drop Shelf.
30. As a DropCove user, I want Remove Item and Remove Batch never to delete source files or folders, so that DropCove remains a reference workspace rather than a file manager.
31. As an Edge Rail user, I want Remove Item and Remove Batch to remain immediate actions without a new confirmation dialog, so that their behavior matches the Drop Shelf.
32. As an Edge Rail user, I want Clear Temporary Items to keep its existing confirmation and shelf-wide meaning, so that the new row actions do not change an established lifecycle command.
33. As an Edge Rail user, I want a single-item Pin action to retain that Shelf Item after Successful Drag-Out, so that pinning has one meaning across surfaces.
34. As an Edge Rail user, I want unpinning a Shelf Item to restore Temporary Item behavior, so that accepted drag-out consumes it as expected.
35. As an Edge Rail user, I want Bulk Pinning Off to pin every Shelf Item in the target Shelf Batch, so that all items become reusable together.
36. As an Edge Rail user, I want Bulk Pinning Mixed to pin every Shelf Item in the target Shelf Batch, so that the action completes the partially pinned group.
37. As an Edge Rail user, I want Bulk Pinning On to make every Shelf Item temporary, so that the operation is reversible.
38. As an Edge Rail user, I want mixed Bulk Pinning state distinguished by shape as well as color, so that it remains perceivable in High Contrast.
39. As an assistive-technology user, I want Bulk Pinning to expose Off, On, and Indeterminate states accurately, so that automation reflects the real child-item lifecycle.
40. As an Edge Rail user, I want Bulk Pinning to remain item-owned rather than creating a Pinned Batch state, so that lifecycle semantics stay consistent.
41. As an Edge Rail user, I want Manage Items to appear only for multi-item Shelf Batches, so that single-item rows remain direct and simple.
42. As an Edge Rail user, I want Manage Items to open an anchored flyout rather than the full Drop Shelf, so that I can stay in the compact workflow.
43. As an Edge Rail user, I want the Manage Items flyout to open toward the desktop on both Left and Right rails, so that it remains on-screen.
44. As an Edge Rail user, I want the flyout width to follow the newly confirmed 320-to-480 logical-pixel contract, so that paths and actions have enough room.
45. As an Edge Rail user, I want the flyout height bounded at 480 logical pixels, so that large Shelf Batches remain manageable without covering the work area.
46. As an Edge Rail user, I want the flyout clamped inside the current monitor work area, so that all controls remain reachable.
47. As an Edge Rail user, I want the flyout to retain vertical scrolling and disable horizontal scrolling, so that long batches and paths do not destabilize layout.
48. As an Edge Rail user, I want a fixed Bulk Pinning header above the item scroller, so that batch lifecycle management remains reachable while I scroll.
49. As an Edge Rail user, I want the flyout header to show `X of N pinned`, so that mixed state is explicit rather than icon-only.
50. As an Edge Rail user, I want the flyout header Bulk Pinning action to match the row action, so that the same command never changes meaning by location.
51. As an Edge Rail user, I want each flyout row to show the Shelf Item thumbnail or native icon, so that item recognition matches the Drop Shelf popup.
52. As an Edge Rail user, I want each flyout row to show the Shelf Item name, so that I can identify the reference.
53. As an Edge Rail user, I want each flyout row to show its Path Reference, so that similarly named items can be distinguished.
54. As an Edge Rail user, I want each flyout row to show Available, Missing, or Unavailable state, so that access problems are visible before drag-out.
55. As an Edge Rail user, I want each flyout row to show Pinned or Temporary lifecycle state, so that accepted-drag behavior is predictable.
56. As an Edge Rail user, I want each flyout row to expose Pin/Unpin, so that I can manage one Shelf Item without changing its siblings.
57. As an Edge Rail user, I want each flyout row to expose Remove Item, so that I can prune one reference without removing the Shelf Batch.
58. As an Edge Rail user, I want each flyout row to remain directly draggable, so that inspection and drag-out stay in one surface.
59. As an Edge Rail user, I want canceled or rejected item drag-out to retain valid references, so that management parity does not weaken drag safety.
60. As an Edge Rail user, I want Successful Drag-Out to consume participating Temporary Items and retain participating Pinned Items, so that existing lifecycle rules remain authoritative.
61. As an Edge Rail user, I want starting a whole-batch drag from non-action row content to preserve current batch drag behavior, so that new controls do not remove an established workflow.
62. As an Edge Rail user, I want pointer actions on Pin, Manage Items, and Remove to take precedence over whole-row drag, so that clicking a control cannot accidentally begin drag-out.
63. As an Edge Rail user, I want the Manage Items flyout to remain open after Pin/Unpin and Bulk Pinning, so that I can continue inspecting and managing the Shelf Batch.
64. As an Edge Rail user, I want the flyout to remain open after Remove Item while at least two Shelf Items remain, so that repetitive cleanup remains efficient.
65. As an Edge Rail user, I want focus to remain near the action used after an item is removed, so that keyboard position does not jump unpredictably.
66. As an Edge Rail user, I want the flyout to close when removal leaves one Shelf Item, so that the row returns to the direct single-item interaction model.
67. As an Edge Rail user, I want the remaining single-item row to expose Pin/Unpin and Remove Item immediately, so that no management capability is lost when the flyout closes.
68. As an Edge Rail user, I want the flyout and row to disappear when the Shelf Batch becomes empty, so that empty containers are never displayed.
69. As an Edge Rail user, I want removal of the final Shelf Item across the shelf to hide the Edge Rail immediately, so that an empty workspace leaves no meaningless handle.
70. As an Edge Rail user, I want removal that reduces batch count to refresh content without live window shrink, so that targets do not move beneath my pointer.
71. As an Edge Rail user, I want the next collapse and expansion to recalculate a smaller height after removals, so that unused space is reclaimed at a stable boundary.
72. As an Edge Rail user, I want a pending Pin or Remove action disabled until it completes, so that repeated clicks cannot queue ambiguous mutations.
73. As an Edge Rail user, I want durable persistence to complete before the row or flyout refreshes, so that visible state never claims an uncommitted mutation.
74. As an Edge Rail user, I want a persistence failure to leave the manager, database, row, and flyout state unchanged, so that I can safely retry manually.
75. As an Edge Rail user, I want pinning failures reported as `Couldn’t update pinning. Nothing changed.`, so that the failure scope and safety are clear.
76. As an Edge Rail user, I want item-removal failures reported as `Couldn’t remove item. Nothing changed.`, so that I know the reference remains.
77. As an Edge Rail user, I want batch-removal failures reported as `Couldn’t remove batch. Nothing changed.`, so that I know the Shelf Batch remains.
78. As an Edge Rail user, I want mutation failures reported through the existing resident tray-warning path, so that the no-activate rail does not need a new status surface.
79. As an Edge Rail user, I do not want automatic retries after persistence failure, so that one action causes at most one intentional mutation attempt.
80. As a Windows user, I want Pin, Manage Items, Remove, and flyout interaction not to activate DropCove, so that my foreground application retains focus.
81. As a Windows user, I want Open Shelf to remain the only rail command that activates the full Drop Shelf, so that activation remains explicit.
82. As an Edge Rail user, I want an open Manage Items flyout to keep the Edge Rail expanded, so that its owner cannot collapse while I use it.
83. As an Edge Rail user, I want a pending mutation to keep the Edge Rail and relevant flyout open, so that the target does not disappear during durable work.
84. As an Edge Rail user, I want normal collapse timing to resume after the flyout closes or a mutation completes, so that the rail returns to its established behavior.
85. As a keyboard user, I want Enter and Space to activate focused Pin, Manage Items, and Remove controls, so that pointer use is not required when the existing keyboard route reaches the rail.
86. As a keyboard user, I want keyboard-opened Manage Items to focus the Bulk Pinning control first, so that the primary batch action is immediately available.
87. As a keyboard user, I want Tab and Shift+Tab to traverse the fixed header and every realized item action in both directions, so that all management controls are reachable.
88. As a keyboard user, I want off-screen item rows realized only as focus traversal reaches them, so that accessibility does not eagerly materialize the whole Shelf Batch.
89. As a keyboard user, I want Escape to close Manage Items and restore focus to its originating control, so that transient navigation is predictable.
90. As a pointer user, I do not want pointer-opened Manage Items to force keyboard focus, so that direct interaction remains non-disruptive.
91. As an assistive-technology user, I want stable accessible names that describe each action and target, so that icon-only controls are understandable.
92. As an assistive-technology user, I want item visuals identified as thumbnails or native icons, so that the rendered content has meaningful automation context.
93. As a High Contrast user, I want previews, row boundaries, focus cues, mixed pin state, and action glyphs to use system theme resources, so that the complete management surface remains legible.
94. As a user with reduced-motion settings, I want existing animation preferences respected, so that this feature adds no unavoidable motion.
95. As a performance-conscious user, I want thumbnails loaded only for realized Edge Rail rows and open flyout items, so that large collections do not materialize hidden visuals.
96. As a performance-conscious user, I want row thumbnail requests canceled and image sources released when the Edge Rail collapses, so that the hidden handle does not retain unnecessary visual resources.
97. As a performance-conscious user, I want flyout thumbnail requests canceled and image sources released when Manage Items closes, so that inspected items do not remain resident indefinitely.
98. As a performance-conscious user, I want native icon caching to reuse the existing bounded-by-type behavior, so that common file types avoid redundant Shell work.
99. As a performance-conscious user, I do not want an application-wide thumbnail cache, so that image content cannot grow resident memory with collection size.
100. As a performance-conscious user, I do not want thumbnail bytes persisted by DropCove, so that the application remains a Path Reference workspace rather than a content store.
101. As a performance-conscious user, I want the Edge Rail batch list to retain virtualization and zero excess cache, so that only visible rows own visuals.
102. As a performance-conscious user, I want full flyout item projections deferred until Manage Items opens, so that closed Shelf Batches remain cheap.
103. As a performance-conscious user, I want the canonical 100 Shelf Batch and 1,000 Shelf Item scenario to remain responsive, so that parity does not reduce established scale.
104. As a performance-conscious user, I want idle CPU to remain at or below 0.1%, so that visual parity adds no background work or polling.
105. As a performance-conscious user, I want shelf-show latency p95 to remain at or below 150 ms, so that shared presentation code does not slow primary activation.
106. As a performance-conscious user, I want total process Working Set to remain strictly below 160 MB under the established Release protocol, so that the feature does not worsen the active release blocker.
107. As a maintainer, I want the Drop Shelf and Edge Rail to share visual primitives and projection rules, so that thumbnail, fallback, and item-row behavior cannot drift.
108. As a maintainer, I want each surface to retain its own shell, layout, flyout ownership, and event handlers, so that compact and full presentations do not become tightly coupled.
109. As a maintainer, I want both surfaces to call the existing manager and drag service directly, so that lifecycle and persistence have one authoritative implementation.
110. As a maintainer, I do not want a shared action controller or forwarding abstraction, so that reuse does not add an unnecessary layer over existing seams.
111. As a maintainer, I want installed Release evidence from the exact tested payload, so that source declarations and stale artifacts are not mistaken for user-visible proof.
112. As a maintainer, I want thumbnail and action behavior qualified separately from memory measurement on the same payload identity, so that UI Automation peer realization cannot contaminate residency evidence.
113. As a maintainer, I want screenshots used as human-review evidence rather than permanent pixel baselines, so that tests do not pin operating-system rendering details.
114. As a maintainer, I want presentation tests to assert behavior, lifecycle, focus, accessibility, and resource outcomes rather than XAML strings or event forwarding, so that refactoring remains safe.

## Implementation Decisions

- This specification deliberately changes the prior compact Edge Rail management asymmetry. Earlier requirements that excluded Pin/Unpin, Bulk Pinning, Remove Item, and Remove Batch from the Edge Rail are superseded only for those actions and their required visual-management support. Existing geometry, placement, hover, drag, focus, fullscreen, sizing, and Clear Temporary Items contracts remain governing unless explicitly changed here.
- The current source contract is preserved: the Rail Handle is 16 by 280 logical pixels, the expanded Edge Rail is 280 logical pixels wide, every Shelf Batch row is 64 logical pixels high, row gaps remain 4 logical pixels, and expanded height uses the existing 280-pixel floor and 640-pixel cap. This feature does not redesign Edge Rail geometry.
- The Edge Rail remains a compact presentation. It does not embed the complete Drop Shelf batch card or reuse a full-card layout that changes row height or rail width.
- Each row reserves a 42 by 42 logical-pixel preview area. A single-item Shelf Batch renders one visual. A multi-item Shelf Batch renders the same three-layer stacked composition used by the Drop Shelf, bounded to the first three Shelf Items in batch order.
- Preview eligibility, Windows Shell thumbnail retrieval, native icon fallback, expected Shell/IO failure suppression, and cancellation use the existing visual-provider and visual-coordinator behavior. The Edge Rail does not create a second thumbnail service or file-type policy.
- Eligible image formats remain governed by the existing shared image classification. Folders and non-image files request native icons only. If an image thumbnail is unavailable or fails through an expected access/Shell path, the native icon remains authoritative.
- Shared presentation work is bounded to reusable visual primitives, presentation projections, styles, and realization/lifetime behavior. Drop Shelf and Edge Rail keep separate card/row shells, layout rules, native-window ownership, popup/flyout ownership, and surface event handlers.
- The shared boundary may move Drop Shelf-local item and bounded-preview presentation types into an application-level presentation area when needed for reuse. The types remain presentation projections over Shelf Item and Shelf Batch; they do not become domain models or persisted state.
- No shared action controller, shelf-management coordinator, public mutation interface, generic command bus, compatibility wrapper, or pass-through abstraction is introduced. Each surface calls DropShelfManager and DragDropService directly through the ownership/callback seams it already has.
- The Edge Rail obtains the existing visual provider/coordinator capability through its composition path rather than constructing an unrelated thumbnail pipeline. Provider lifetime must not cause the Edge Rail to retain Drop Shelf visual trees or vice versa.
- The Edge Rail batch list remains virtualized. Row visual realization begins when a row is prepared and ends when the row is cleared, the rail collapses, the rail hides, or its content projection is replaced.
- At most three preview visual requests exist per realized Shelf Batch row. Closing or collapsing the Edge Rail cancels in-flight row requests, clears ImageSource references, and releases row realization bookkeeping. The plain Rail Handle retains no row thumbnails.
- Full flyout item projections and visual requests are created only when Manage Items opens. Closing the flyout, removing its owner Shelf Batch, hiding the Edge Rail, or replacing the open owner cancels item requests, clears ImageSource references, and releases item projections.
- Thumbnail results are not cached across collapsed or closed presentation lifetimes. The existing native-icon cache by file type/folder may remain shared. No application-wide path-keyed thumbnail cache or persisted thumbnail storage is added.
- A single-item row uses the Drop Shelf title/subtitle rules: title is the Shelf Item name; subtitle is its type, prefixed by pinned lifecycle when pinned.
- A multi-item row uses the Drop Shelf title/subtitle rules: title is `N items`; subtitle is the first two item names and includes `K pinned` when one or more Shelf Items are pinned.
- Row text remains two lines, trims without wrapping, and never participates in native-window size measurement. Full names and relevant details remain available through tooltip and accessibility properties.
- A single-item row exposes two always-visible 24 by 24 controls in this order: Pin/Unpin, then Remove Item.
- A multi-item row exposes three always-visible 24 by 24 controls in this order: Bulk Pinning, Manage Items, then Remove Batch.
- The existing pin glyph and mixed-state marker remain the visual basis for Pin/Unpin and Bulk Pinning. Mixed state must be shape-distinct and theme-aware.
- Remove Item and Remove Batch use the same `×` affordance as the Drop Shelf. The trash glyph remains reserved for Clear Temporary Items.
- The non-action part of the row remains the whole-Shelf-Batch drag source. Action controls consume pointer activation and do not initiate whole-batch drag.
- Single-item Pin/Unpin calls the existing item pin mutation. Bulk Pinning calls the existing all-items pin mutation. Mixed and all-Temporary states target all Pinned; all-Pinned targets all Temporary.
- Pin state remains owned by Shelf Items. The Shelf Batch receives no pinned field, persistence column, or secondary source of truth.
- Remove Item calls the existing exact-item mutation. If it removes the final Shelf Item in a Shelf Batch, the empty Shelf Batch is pruned by the manager.
- Remove Batch calls the existing exact-batch mutation and removes pinned and temporary references in that Shelf Batch. It does not reuse Clear Temporary Items or accepted-drag cleanup.
- Remove Item and Remove Batch remain immediate reference-removal actions without a new confirmation dialog. They never delete, move, rename, copy, overwrite, or otherwise modify source filesystem objects.
- Clear Temporary Items remains the existing confirmed shelf-wide action, retains pinned references, and remains visually and semantically distinct from row removal.
- Each Pin or Remove handler prevents duplicate activation for its affected target while the mutation is in progress. The rail and an owning Manage Items flyout remain open during the operation.
- Presentation state remains manager-derived until durable mutation succeeds. A ToggleButton activation must not leave an optimistic checked state visible while persistence is pending; the current manager state is retained or restored until refresh after success.
- Successful mutations refresh the Edge Rail through the existing post-mutation callback. The refresh updates row text, preview ownership, action state, flyout contents, batch ordering, and Hidden transition as one coherent projection.
- Pinning persistence failure retains current durable and in-memory state, restores manager-derived control state, reports `Couldn’t update pinning. Nothing changed.` through the existing resident warning callback, and performs no automatic retry.
- Remove Item persistence failure retains current durable and in-memory state, keeps the row/flyout content visible, reports `Couldn’t remove item. Nothing changed.` through the existing resident warning callback, and performs no automatic retry.
- Remove Batch persistence failure retains current durable and in-memory state, keeps the Shelf Batch visible, reports `Couldn’t remove batch. Nothing changed.` through the existing resident warning callback, and performs no automatic retry.
- Manage Items replaces the current fixed 260-logical-pixel-wide read/drag-only rail flyout with the Drop Shelf item-management content contract while retaining Edge Rail-native ownership and desktop-facing placement. The 260-pixel width is superseded for this feature.
- Manage Items is available only for Shelf Batches containing more than one Shelf Item. Activating it toggles one anchored flyout; only one rail management flyout may be open at a time.
- The new flyout contract sizes to content between 320 and 480 logical pixels wide, uses a maximum height of 480 logical pixels, disables horizontal scrolling, and scrolls item rows vertically inside the remaining height.
- The flyout opens toward the desktop for both Left and Right Edge Rail placement and clamps inside the selected monitor work area. Placement does not activate the full Drop Shelf or alter rail placement settings.
- A fixed, non-scrolling header shows `X of N pinned` and the tri-state Bulk Pinning control. Only the item list scrolls.
- Item rows use the shared item presentation: 24-pixel thumbnail/native icon, name, Path Reference, availability/lifecycle details, Pin/Unpin, Remove Item, and direct drag-out.
- Available, Unavailable, and confirmed-Missing Shelf Items still present in the Shelf Batch participate in Pin/Unpin and Bulk Pinning. These actions change lifecycle only and do not reclassify availability.
- Existing Missing cleanup, Unavailable retention, partial-availability confirmation, Copy-only drag effects, Successful Drag-Out consumption, and canceled/rejected drag behavior remain authoritative.
- Pin/Unpin and Bulk Pinning from inside the flyout keep it open and retain focus on or near the action used. Item drag-out retains the established flyout lifetime guard.
- Remove Item keeps the flyout open when at least two Shelf Items remain. Focus moves to the nearest remaining logical action without eagerly realizing all rows.
- When Remove Item leaves exactly one Shelf Item, the flyout closes, its visuals are released, and the owner row refreshes into the direct single-item presentation.
- When Remove Item leaves zero Shelf Items, the Shelf Batch and flyout disappear. Removing the final Shelf Item across all batches transitions the shelf to Hidden and hides the Edge Rail immediately.
- Reductions in Shelf Batch count do not live-shrink an expanded non-empty rail. The list refreshes inside current bounds; the next collapse and expansion recomputes the lower existing target height.
- An open flyout, pending mutation, active drag, or existing Clear Temporary Items confirmation suppresses rail collapse. When the hold condition ends and the pointer is outside, the established 300 ms collapse timing resumes.
- The Edge Rail keeps its no-activate native-window behavior. Pointer interaction with preview, Pin, Manage Items, Remove, flyout controls, or drag sources must not activate DropCove or steal focus from the foreground application.
- Open Shelf remains the only Edge Rail command that activates the full Drop Shelf. It is not repurposed as a row expansion, flyout, or rail-lock control.
- When controls are reached through the existing keyboard/accessibility route, Enter and Space activate focused actions. Keyboard-opened Manage Items focuses the fixed Bulk Pinning control; pointer opening does not force focus.
- Flyout Tab and Shift+Tab traversal visits the fixed header, then each item's Pin/Unpin and Remove Item actions, wrapping in both directions and realizing only the target off-screen row when necessary.
- Escape closes Manage Items and restores focus to its originating Manage Items control. It does not dismiss or activate the full Drop Shelf as part of the same action.
- Accessible names remain action- and target-specific: Pin/Unpin names include the Shelf Item; Bulk Pinning names include item count and mixed count; Remove names identify item or Shelf Batch; Manage Items identifies its Shelf Batch.
- Existing theme resources, focus resources, control styles, and High Contrast behavior remain authoritative. No fixed RGB/ARGB palette or Edge Rail-only color system is introduced.
- Existing animation preference handling remains authoritative. This feature adds no timer, polling loop, background thumbnail prefetcher, or mandatory new motion.
- No Shelf Item, Shelf Batch, display-state, rail-placement, sizing-state, settings, or SQLite schema change is required.
- No new runtime package, development dependency, UI test framework, telemetry system, thumbnail database, or filesystem watcher is introduced.
- The strict performance gates remain unchanged: total process Working Set strictly below 160 MB under the established canonical protocol, idle CPU at or below 0.1%, and shelf-show latency p95 at or below 150 ms.

## Testing Decisions

- Good permanent tests assert consumer-visible lifecycle, persistence, fallback, cancellation, sizing, and state-transition behavior. They do not assert XAML source text, resource-key names, private field shapes, handler forwarding, visual-tree child counts, exact control implementation classes, or duplicated literals.
- DropShelfManager backed by a real temporary SQLite database remains the highest automated seam for Pin/Unpin, Bulk Pinning, Remove Item, Remove Batch, final-item Hidden transition, mutation serialization, database-first state replacement, restart durability, and atomic failure behavior.
- Existing manager tests are prior art for item pinning, item removal, batch removal, Clear Temporary Items, accepted/canceled drag-out, Missing cleanup, EdgeDocked lifecycle, final-item hiding, and sizing reset. Existing persistence tests are prior art for immediate commit, restart restoration, Bulk Pinning atomicity, deterministic SQLite aborts, and unchanged state after failed mutations.
- Manager and persistence semantics are not duplicated merely because the Edge Rail becomes a new caller. Add or change core tests only when implementation changes manager behavior or exposes an uncovered consumer-visible failure. Do not add tests that only prove a rail handler passed an ID and boolean to an already-tested manager method.
- ShelfVisualCoordinator remains the highest automated seam for thumbnail/native-icon selection and cancellation. Existing tests are prior art for non-image native icons, image thumbnail selection, thumbnail-first completion, fallback when thumbnail is unavailable, failure non-caching, and cancellation propagation.
- Extend visual-coordinator tests only if the shared visual behavior itself changes. Surface realization and ImageSource release are proved through installed behavior and residency evidence rather than tests that inspect presentation dictionaries or XAML events.
- EdgeRailSizingPolicy tests remain the authority for the current 16-by-280 handle height, 280-pixel expanded floor, 64-pixel rows, deterministic growth tiers, 640-pixel cap, grow-only expanded mutation behavior, and later recalculation. This feature must keep them green rather than rewriting geometry expectations.
- The complete existing automated suite must remain green. Existing Drop Shelf, drag, persistence, sizing, placement, shake, settings, window interop, and visual coordinator tests are regression coverage and are not weakened to accommodate presentation changes.
- Installed self-contained Release behavior is the acceptance authority for actual thumbnails, stacked previews, glyphs, control placement, flyout geometry, pointer hit testing, no-activate behavior, keyboard focus, UI Automation state, drag interactions, and native resource lifetime.
- Installed evidence uses one explicit payload fingerprint. Source inspection, unit tests, screenshots from another build, or historical geometry artifacts cannot substitute for behavior observed from that payload.
- Row visual smoke covers a single image item, single non-image file, folder, multi-item batch with two items, multi-item batch with at least three items, and a batch larger than three. It verifies one visual for single items, at most three stacked visuals for multi-item batches, correct ordering, native fallback, and stable 64-pixel row height.
- Thumbnail fallback smoke covers unsupported image content, a missing path, an unavailable path, and an expected Shell thumbnail failure. Every row and flyout item must remain actionable with a native icon or fallback glyph and must not display an unrecoverable blank visual.
- Row content smoke verifies Drop Shelf-equivalent single and multi title/subtitle rules, trimming, tooltips/accessibility context, and always-visible 24-pixel actions in Pin/Manage Items/Remove order without overlap at the fixed 280-pixel width.
- Action smoke covers single Pin, single Unpin, mixed-to-all-Pinned Bulk Pinning, all-Temporary-to-all-Pinned Bulk Pinning, all-Pinned-to-all-Temporary Bulk Pinning, Remove Item, Remove Batch, and Clear Temporary Items remaining separate.
- Action results are verified in both visible presentation and durable SQLite state. Restart smoke confirms successful Pin/Unpin and removal persist, while deterministic failure leaves both current and reopened state unchanged.
- Failure smoke uses an isolated test profile and deterministic SQLite abort conditions. It verifies affected controls cannot queue duplicates, rail/flyout remain coherent, data remains unchanged, no automatic retry occurs, and the exact resident warning category/message is observable.
- Source filesystem fixtures are checked before and after Remove Item, Remove Batch, Bulk Pinning, Clear Temporary Items, and drag scenarios to prove DropCove changed references only.
- Manage Items smoke verifies 320-to-480-pixel width behavior, maximum 480-pixel height, desktop-facing placement on both Left and Right rails, work-area clamping, fixed header, vertical scrolling, and absence of horizontal scrolling.
- Flyout content smoke verifies thumbnail/native icon, name, Path Reference, availability, lifecycle, Pin/Unpin, Remove Item, and drag-out for real item rows. Closed flyouts expose no retained item-row presentation through the installed inspection seam.
- Shape-transition smoke removes an item from batches that end with three, two, one, and zero items. It verifies flyout retention at two or more, closure and single-item row conversion at one, batch pruning at zero, and immediate Edge Rail hiding when the shelf becomes empty.
- Drag smoke covers whole-batch drag from non-action row content, pointer activation of each action without accidental drag, item drag from Manage Items, accepted Temporary consumption, Pinned retention, cancellation, rejection, Missing cleanup, and Unavailable retention.
- No-activate smoke records the foreground window before and after Pin, Manage Items, Remove, scrolling, and item drag. Direct rail interaction must preserve foreground focus; Open Shelf must remain the explicit activation path.
- Keyboard smoke verifies Enter/Space activation, pointer opening without forced focus, keyboard opening with initial Bulk Pinning focus, Tab/Shift+Tab traversal and wrapping across virtualized rows, Escape closure, focus restoration, and no focus trap.
- UI Automation verifies stable accessible names, enabled/busy state, ToggleState Off/On/Indeterminate, visible focus, action rectangles, flyout ownership, and the visual automation distinction between thumbnail and native icon where exposed.
- Light and Dark evidence uses the exact qualification payload and records representative single/multi rows plus the Manage Items header and item rows. High Contrast runs only in an isolated or safely reversible environment and verifies boundaries, mixed state, glyphs, thumbnails/icons, and focus by more than color alone.
- Reduced-motion smoke verifies that row management, flyout opening/closing, refresh, and collapse remain functional when non-essential animation is disabled.
- Virtualization smoke uses 100 Shelf Batches and 1,000 Shelf Items. It verifies bounded row realization, at most three preview requests per realized batch, deferred full item projections, responsive scrolling, and on-demand off-screen keyboard realization.
- Resource-lifetime smoke expands the rail, observes row visuals, opens and closes Manage Items, collapses and re-expands the rail, and verifies visuals reload correctly rather than relying on retained hidden ImageSource state. Instrumentation used for diagnosis must not become production telemetry or a test-only product hook.
- Residency qualification runs against the installed Release payload with exactly 100 Shelf Batches and 1,000 Shelf Items using the established isolated profile and total process WorkingSet64 metric. The strict threshold remains `<160 MB`; averages or alternative memory metrics do not waive an individual required sample over the limit.
- The canonical residency run preserves the active release protocol and adds representative Edge Rail expansion/collapse and Manage Items open/close checkpoints sufficient to detect thumbnail or visual-peer accumulation. Any added rail-specific measurement supplements rather than replaces the canonical release gate.
- Repeated Edge Rail interaction includes enough expansion/collapse and flyout open/close cycles to detect retained thumbnail, projection, or native peer growth. Every reported checkpoint names whether the rail is collapsed, expanded, flyout-open, or post-release.
- Idle CPU and shelf-show latency are measured on the same payload identity using established procedures. Thumbnail parity adds no polling or background prefetch work; idle CPU must remain at or below 0.1%, and shelf-show p95 must remain at or below 150 ms.
- UI Automation/visual qualification and canonical residency measurement run separately against the same build fingerprint so automation-peer realization cannot contaminate memory evidence.
- Screenshots and captured bounds are qualification evidence for human review. No screenshot-baseline framework or pixel-perfect permanent test is introduced.
- Qualification records unavailable DPI, monitor, High Contrast, or work-area configurations as unobserved. It never infers installed behavior from resources, source values, tests, or a different payload.

## Out of Scope

- Changing the current 16-by-280 Rail Handle dimensions, 280-pixel expanded width, 64-pixel row height, row gaps, adaptive height tiers, 640-pixel cap, centered placement, or grow-versus-hold sizing policy.
- Adding content, labels, counts, thumbnails, or controls to the resting Rail Handle.
- Replacing hover expansion, changing the 200 ms expansion delay or 300 ms collapse delay, adding a manual rail-lock toggle, or repurposing Open Shelf.
- Changing topmost behavior, no-activate native styles, fullscreen policy, monitor selection, remembered placement, display/DPI event handling, or reduced-motion policy.
- Embedding the complete Drop Shelf card in the Edge Rail or making both surfaces use one geometry/layout component.
- Redesigning the Drop Shelf's user-visible behavior beyond extracting shared visual presentation primitives without semantic change.
- Adding confirmations for Remove Item or Remove Batch, adding undo history, adding automatic retry, or changing Clear Temporary Items confirmation and scope.
- Deleting, moving, renaming, copying, overwriting, or storing source filesystem content.
- Adding a Pinned Batch domain object, batch-owned pin state, new lifecycle state, new availability state, database column, schema migration, or settings record.
- Generalizing Pin, Remove Item, Remove Batch, Clear Temporary Items, Missing cleanup, and accepted-drag cleanup into one generic mutation abstraction.
- Adding a shared action controller, coordinator, public repository interface, generic command framework, compatibility layer, or feature flag.
- Adding an application-wide thumbnail cache, persisting thumbnail bytes, preloading hidden batches, background thumbnail refresh, filesystem watchers, or thumbnail telemetry.
- Changing Windows Shell thumbnail eligibility or adding custom image decoding unless the existing provider cannot satisfy the accepted formats and a separate approved decision changes that policy.
- Changing drag effects, destination acceptance rules, Missing/Unavailable semantics, Copy-only behavior, partial batch confirmation, or source-integrity guarantees.
- Adding search, filters, arbitrary selection, cross-batch Bulk Pinning, batch editing, named groups, or multiple shelves.
- Adding Mica, Acrylic, custom shadows, custom window regions, a new theme palette, decorative animation, or unrelated Edge Rail shell redesign.
- Introducing a new runtime dependency, development profiler dependency, UI test framework, screenshot regression framework, telemetry system, or background worker.
- Solving the independent native-residency root cause, lowering the `<160 MB` threshold, changing its metric, forcing garbage collection, trimming the process working set, or otherwise waiving the active release blocker. This feature must requalify and must not worsen it.
- Declaring the current 280-pixel/8-pixel-corner source contract installed-release verified without matching same-payload evidence.

## Further Notes

- `CONTEXT.md` now defines **Manage Items**, **Remove Item**, and **Remove Batch**. This specification uses those canonical terms and avoids using “expand” for the per-batch management action or “delete” for reference removal.
- No ADR is required. The selected shared-visual-primitive boundary is a reversible presentation refactor over existing Shelf Item, Shelf Batch, DropShelfManager, DragDropService, and ShelfVisualCoordinator ownership. It does not change domain boundaries, persistence ownership, native-window ownership, deployment technology, or a hard-to-reverse integration contract. The trade-off and rejected full-card/action-controller alternatives are recorded directly in this specification.
- The historical 16-by-96/16-pixel-corner/136-204-272-tier artifact at `artifacts/adaptive-rail-release-qualification.json` is preserved as historical evidence only.
- Feature delivery progress for Tickets 01 through 05 is recorded in `artifacts/parity-feature-qualification-record.json` using the fingerprinted build payload (`7b1a7264ad71457ead90480520ce2a7d6e48e3bd6c613b095f3dbc48e3f9f238`).
- Ticket 06 remains open and blocked: production qualification requires an installed self-contained Release run and same-payload `<160 MB` residency clearance without disturbing ambient user processes (e.g. PID 25196). Ticket 19 remains the governing residency requirement.
- All changes remain uncommitted until explicit user authorization.
- This specification supersedes prior compact-rail exclusions of Pin/Unpin, Bulk Pinning, Remove Item, and Remove Batch, plus the current fixed 260-pixel read/drag-only flyout width. It does not reopen the remainder of the compact Edge Rail contract.
- The testing seams were confirmed during design: shared visual primitives/projections and ShelfVisualCoordinator for visual policy, direct DropShelfManager/SQLite behavior for lifecycle and durability, existing sizing policy for geometry invariants, and installed Release smoke/performance for real WinUI behavior. No new public test seam is planned.
- All specification and glossary changes remain uncommitted until explicit user authorization. This specification does not authorize implementation, ticket creation, or a Git commit.
