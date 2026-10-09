# Compact Manage Items Controls Specification

Status: ready-for-agent

## Problem Statement

Manage Items can become disproportionately wide when a Shelf Item has a long Path Reference. Both the Drop Shelf popup and Edge Rail Flyout currently permit a 320-to-480-logical-pixel width, which allows one long path to produce a surface wider than the 280-logical-pixel expanded Edge Rail. The result consumes unnecessary desktop space and makes the same compact workflow feel detached from its owning surface.

The current end-trimming treatment can also hide the final item name, which is usually the most useful part of a long Path Reference. The fixed header places the Bulk Pinning control to the right of its count instead of reading naturally as one compact status-and-action group. Neither Manage Items surface exposes a visible close control, so pointer users must rely on outside click or the originating Manage Items control even when they are already interacting inside the surface.

Users need both Manage Items surfaces to remain compact, preserve useful path identity, present Bulk Pinning consistently, and provide an explicit close action without changing Shelf Item lifecycle, popup ownership, placement, scrolling, drag behavior, or existing dismissal routes.

## Solution

Set the Drop Shelf Manage Items popup and Edge Rail Manage Items Flyout to a stable outer width of 280 logical pixels, matching the expanded Edge Rail. A long Path Reference uses a tail-preserving representation such as `…\parent_folder\file_name.png`; the complete Path Reference remains available through tooltip and accessibility metadata. Short Path References remain complete when they fit, and pathological long names retain normal ellipsis as the final visual safety.

Recompose each fixed header as `[Bulk Pinning] [X/N pinned] [flexible space] [Close]`. The tri-state Bulk Pinning control remains item-centric and keeps its current behavior, tooltip, accessible action, and 24-by-24-logical-pixel target. The visible summary changes from `X of N pinned` to the compact `X/N pinned` form. Item-level Pin/Unpin and Remove Item controls remain on the right side of each Shelf Item row.

Add an explicit Close button at the upper right of both surfaces. It supplements rather than replaces `Escape`, outside click or light dismissal, and activation of the originating Manage Items control. Closing restores focus to the originating control when appropriate and never bypasses existing pending-mutation or active-drag guards.

## User Stories

1. As a Drop Shelf user, I want Manage Items to stay compact, so that a single long Path Reference does not cover unnecessary desktop space.
2. As an Edge Rail user, I want Manage Items no wider than the expanded Edge Rail, so that the transient surface remains visually connected to its owner.
3. As a user switching between the Drop Shelf and Edge Rail, I want both Manage Items surfaces to use the same width, so that the same workflow has stable geometry.
4. As a user opening different Shelf Batches, I want the Manage Items width to remain stable, so that it does not jump according to item path length.
5. As a user with a short Path Reference, I want to see the complete path, so that compact sizing does not discard information unnecessarily.
6. As a user with a long Path Reference, I want the displayed path to preserve the final item name, so that I can identify the referenced file or folder.
7. As a user with similarly named Shelf Items, I want a compact path to preserve the immediate parent folder, so that I can distinguish references from different locations.
8. As a user with a long local path, I want to see a tail form such as `…\parent_folder\file_name.png`, so that the most relevant location information remains visible.
9. As a user with a long folder Path Reference, I want the compact tail to preserve the folder name and its immediate parent, so that folder references remain identifiable.
10. As a user with a root-level or already short Path Reference, I want it left intact when it fits, so that shortening is not applied without benefit.
11. As a user with a UNC or unavailable Path Reference, I want display compaction to remain presentation-only, so that DropCove does not normalize, probe, or rewrite the referenced location.
12. As a user whose item name is itself wider than the available line, I want normal ellipsis to keep the row bounded, so that an exceptional name cannot widen the popup.
13. As a pointer user, I want the full Path Reference in a tooltip, so that compact display does not prevent inspection.
14. As an assistive-technology user, I want the full Path Reference exposed through accessibility metadata, so that visual compaction does not remove information.
15. As a user with a large Shelf Batch, I want vertical scrolling without horizontal scrolling, so that long paths cannot destabilize the compact layout.
16. As a user, I want the Bulk Pinning icon at the left of its status text, so that the action and the state read as one group.
17. As a user, I want the header to read visually as `[pin] X/N pinned`, so that the compact meaning is immediately clear.
18. As a user, I want the count rendered as `0/N pinned`, `X/N pinned`, or `N/N pinned`, so that the lifecycle status remains explicit in less space.
19. As a user, I want the count to update immediately after individual Pin/Unpin, Bulk Pinning, Remove Item, and lifecycle refreshes, so that the header never becomes stale.
20. As a user, I want the Bulk Pinning control to remain tri-state, so that all-Temporary, mixed, and all-Pinned Shelf Batches remain accurately represented.
21. As a user with a mixed Shelf Batch, I want activating Bulk Pinning to continue pinning every item, so that moving the control does not change its action.
22. As a user with an all-Pinned Shelf Batch, I want activating Bulk Pinning to continue making every item Temporary, so that the action remains reversible.
23. As an assistive-technology user, I want the Bulk Pinning control to retain its count-aware accessible action name and Off, On, or Indeterminate state, so that the compact header remains understandable.
24. As a High Contrast user, I want the mixed Bulk Pinning state to remain distinguishable by shape, so that the layout change does not reduce state visibility.
25. As a user, I want each Shelf Item row's Pin/Unpin and Remove Item controls to remain on the right, so that only the fixed header changes order.
26. As a user, I want a visible Close button in Manage Items, so that I can dismiss the surface without moving back to its owner.
27. As a pointer user, I want the Close button at the upper right, so that it follows the expected transient-surface convention.
28. As a user, I want the Close button to use a standard close icon, so that its purpose is recognizable without a text label consuming width.
29. As a user, I want a `Close Manage Items` tooltip, so that the icon has explicit pointer guidance.
30. As an assistive-technology user, I want the Close button named `Close Manage Items`, so that an icon-only control remains accessible.
31. As a keyboard user, I want the Close button reachable through `Tab` and `Shift+Tab`, so that the new action is not pointer-only.
32. As a keyboard user, I want `Enter` and `Space` to activate the focused Close button, so that it follows standard WinUI button behavior.
33. As a keyboard user, I want `Escape` to continue closing Manage Items, so that the new button does not remove the faster keyboard route.
34. As a pointer user, I want outside click or native light dismissal to continue closing Manage Items, so that existing transient behavior remains familiar.
35. As a user, I want activating the originating Manage Items control again to continue closing its surface, so that toggle-close behavior remains available.
36. As a user who closes Manage Items explicitly, I want focus returned to the originating Manage Items control when that control still exists, so that context is preserved.
37. As a user whose originating Shelf Batch was removed, I want closure to complete safely without attempting to focus a stale control, so that lifecycle changes cannot cause an error.
38. As a user performing Pin/Unpin, Remove Item, or drag-out, I do not want the Close button to bypass existing pending-operation guards, so that the surface cannot disappear in an unsafe state.
39. As an Edge Rail user, I want an open Manage Items Flyout to continue holding the Edge Rail expanded, so that adding a Close button does not let the owner collapse underneath it.
40. As an Edge Rail user, I want direct interaction to preserve the foreground application's focus policy, so that the new control does not activate DropCove unexpectedly.
41. As a pointer user, I do not want opening Manage Items to force keyboard focus, so that the existing non-disruptive pointer workflow remains unchanged.
42. As a keyboard user, I want keyboard-opened Manage Items to focus Bulk Pinning first, so that the primary lifecycle action remains the initial target.
43. As a keyboard user, I want the fixed-header focus order to proceed from Bulk Pinning to Close, so that navigation follows the visible left-to-right order.
44. As a keyboard user, I want focus to continue from Close through Pin/Unpin and Remove Item for every row, so that every action remains reachable.
45. As a keyboard user, I want forward and reverse traversal to wrap across both header controls and all realized or virtualized row actions, so that the compact surface remains a bounded keyboard context.
46. As a keyboard user, I want traversal to realize only the target virtualized row, so that the added header control does not eagerly materialize the whole Shelf Batch.
47. As a user scrolling a long Shelf Batch, I want the header to remain fixed, so that Bulk Pinning status and Close remain reachable.
48. As a Light-mode user, I want the compact layout to retain the established theme-aware surface and control states, so that sizing changes do not introduce a new visual language.
49. As a Dark-mode user, I want text, icons, boundary, and focus states to remain legible at 280 logical pixels, so that compact sizing remains usable.
50. As a High Contrast user, I want the Close icon, Bulk Pinning state, text, boundary, and visible focus to remain system-resolved, so that the new composition remains accessible.
51. As a text-scaling user, I want essential actions to remain reachable and text to trim rather than overlap controls, so that the fixed width does not create an unusable header or row.
52. As a user, I want the maximum Manage Items height, fixed header, and vertical item scrolling to remain unchanged, so that this feature affects width and controls only.
53. As a Left Edge Rail user, I want Manage Items to continue opening toward the desktop, so that its narrower width does not change placement direction.
54. As a Right Edge Rail user, I want Manage Items to continue opening toward the desktop, so that mirrored placement remains correct.
55. As a user near a monitor work-area boundary, I want existing placement and reachability rules preserved, so that the compact surface remains actionable.
56. As a user, I want Shelf Item drag-out, cancellation, rejection, Successful Drag-Out, Missing cleanup, and Unavailable retention unchanged, so that this presentation change does not alter lifecycle semantics.
57. As a user, I want Pin/Unpin, Bulk Pinning, Remove Item, and source-filesystem safety unchanged, so that no presentation action modifies the referenced filesystem object.
58. As a performance-conscious user, I want path compaction computed without filesystem access, background work, polling, or repeated layout-driven state mutation, so that the compact surface adds no idle cost.
59. As a maintainer, I want both surfaces to reuse the existing Shelf Item presentation projection for compact path text, so that Drop Shelf and Edge Rail cannot drift into separate formatting rules.
60. As a maintainer, I want the existing popup and Flyout ownership retained, so that a small UX improvement does not create a shared lifecycle abstraction.
61. As a maintainer, I want the existing expanded Edge Rail width to remain the single source for the 280-logical-pixel contract, so that the two Manage Items surfaces cannot diverge through duplicated independent bounds.
62. As a maintainer, I want installed Release evidence to remain authoritative for rendered width, focus, accessibility, and dismissal, so that source declarations are not mistaken for observable WinUI behavior.
63. As a maintainer, I want the existing MSTest suite to remain regression coverage without a new UI test project, so that a compact presentation change does not add brittle permanent infrastructure.
64. As a maintainer, I want no new dependency, schema, setting, timer, cache, telemetry path, or background worker, so that the implementation remains a small presentation cutover.

## Implementation Decisions

- Both Manage Items surfaces use an outer width of exactly 280 logical pixels under normal work-area conditions. This width matches the established expanded Edge Rail width and replaces the former 320-to-480-logical-pixel content-sized contract.
- The existing expanded Edge Rail width policy is the authoritative value for this feature. The implementation must not leave independent Drop Shelf and Edge Rail width limits that can drift.
- Existing monitor work-area reachability remains authoritative. If the operating environment cannot present the full logical width, reachability takes precedence; the application does not introduce a second placement algorithm.
- The Drop Shelf retains its existing custom Popup owner. The Edge Rail retains its existing native Flyout and FlyoutPresenter owner. Neither surface is converted to the other's control type.
- Existing maximum height, fixed-header composition, vertical item scrolling, disabled horizontal scrolling, desktop-facing Edge Rail placement, and popup anchoring remain unchanged.
- Long Path References cannot determine or enlarge the outer surface width.
- The Shelf Item presentation projection gains one shared visible path value used by both Manage Items surfaces. The canonical Path Reference remains unchanged and continues to drive lifecycle and drag behavior.
- A short Path Reference remains complete when it fits the constrained path line.
- When the complete path would hide the final item name, the visible value preserves the immediate parent folder and item name with a leading ellipsis, using Windows path separators: `…\parent_folder\file_name.png`.
- Folder references use the same tail rule. Root paths, root-level items, UNC paths, trailing directory separators, unavailable locations, and confirmed-Missing locations must be handled as strings without probing the filesystem.
- If the preserved parent and item name still exceed the available path line, standard character ellipsis remains the final visual fallback. The row never gains horizontal scrolling and never widens.
- The full canonical Path Reference remains in the path tooltip and accessibility metadata even when the visible text is compacted.
- Compact path formatting is deterministic, side-effect free, and computed when the Shelf Item presentation changes rather than through filesystem I/O or continuous layout callbacks.
- Both headers use the visual order Bulk Pinning control, pinned-count text, flexible space, Close button.
- The visible count format is exactly `X/N pinned`, where `X` is the number of Pinned Items and `N` is the current Shelf Item count. This replaces `X of N pinned` only in the Manage Items headers.
- The existing tri-state Bulk Pinning projection and item-centric semantics remain authoritative. The Shelf Batch does not gain a pinned state.
- The Bulk Pinning control retains its current 24-by-24-logical-pixel target, compact pin visual, tooltip, action-specific accessible name, ToggleState behavior, busy state, and mutation path.
- Item-row Pin/Unpin and Remove Item controls retain their current right-side position, dimensions, semantics, and accessible names.
- Each header adds one icon-only Close button at the upper right, using the existing compact action-button styling and a standard WinUI close glyph.
- The Close button has the tooltip and accessible name `Close Manage Items` and exposes ordinary button invocation semantics.
- The explicit Close action supplements existing dismissal. `Escape`, outside click or native light dismissal, and activation of the originating Manage Items control remain supported.
- Explicit closure returns focus to the originating Manage Items control when it remains loaded, visible, and focusable. Existing safe fallback behavior remains authoritative when the owner changed or disappeared.
- The Close action must route through each surface's existing close and cleanup path. It must not duplicate projection release, owner-state reset, rail-collapse resumption, or focus restoration logic.
- The Close action must not bypass pending mutation, active drag, or owner-lifetime guards. No force-close path is introduced.
- Keyboard-opened Manage Items continues to focus Bulk Pinning first. The complete logical focus cycle becomes two fixed header actions followed by two actions per Shelf Item row: Bulk Pinning, Close, row Pin/Unpin, row Remove Item.
- `Tab` and `Shift+Tab` traverse and wrap through the complete logical sequence. Virtualized rows continue to be realized only when targeted.
- Pointer-opened behavior remains non-disruptive. The Edge Rail's no-activate policy and pointer-focus behavior remain unchanged.
- Light, Dark, High Contrast, pointer states, pressed states, disabled states, checked and indeterminate states, and visible keyboard focus continue to resolve through existing WinUI controls, styles, and theme resources.
- No Path Reference, Shelf Item, Shelf Batch, lifecycle, availability, persistence, settings, display-state, sizing-state, or database schema changes are required.
- No new public interface, shared popup controller, command bus, adapter, custom control, package, UI test project, screenshot-baseline framework, telemetry, timer, cache, polling loop, or background worker is introduced.
- User-facing documentation is updated after implementation to describe the compact 280-logical-pixel Manage Items surfaces, tail-preserving path display, left-aligned Bulk Pinning summary, and explicit Close action.
- This specification deliberately supersedes prior requirements that fixed either Manage Items surface at a 320-to-480-logical-pixel width, kept the `X of N pinned` header copy, placed Bulk Pinning to the right of the count, modeled only one fixed header focus action, prohibited Drop Shelf header changes, or required the existing Edge Rail header composition to remain unchanged. All unrelated placement, lifecycle, drag, accessibility, performance, visual-parity, and ownership requirements remain governing.

## Testing Decisions

- Good tests and qualification evidence assert user-observable rendered width, path identity, header order, accessible state, focus order, dismissal, and lifecycle preservation. They do not assert XAML source text, private element names, duplicated numeric constants, exact Grid columns, event-handler forwarding, resource-key forwarding, or helper-call structure.
- The user-confirmed authoritative acceptance seam is the installed self-contained Release running with an isolated test profile. This is the highest existing seam that can prove actual WinUI Popup and Flyout geometry, text rendering, DPI conversion, focus, UI Automation, no-activate behavior, and dismissal together.
- The complete existing MSTest suite remains regression coverage for manager lifecycle, persistence, Edge Rail sizing, placement, drag behavior, settings, visual-resource lifetime, and native interop. No new permanent UI test project is added.
- No permanent test is added merely to prove that a width property, column order, tooltip property, glyph, or event handler exists. Installed observation proves those outcomes.
- The representative fixture contains a mixed-lifecycle multi-item Shelf Batch with a short local Path Reference, a deeply nested long file Path Reference, a deeply nested folder Path Reference, a long item name, an unavailable or UNC-style Path Reference, and enough Shelf Items to require vertical scrolling.
- The same fixture is opened through Manage Items on the Drop Shelf and both Left and Right Edge Rail configurations from the same installed payload and isolated profile.
- UI Automation captures each surface's physical bounds and exercised DPI, converts them to logical dimensions, and verifies an outer width of 280 logical pixels within normal DPI rounding tolerance.
- Visual and UI Automation evidence verifies that different path lengths do not change either surface's width.
- Path evidence verifies that a short path remains complete, a long file path preserves `…\parent\file.ext`, a long folder path preserves `…\parent\folder`, and a pathological long tail remains bounded without horizontal scrolling.
- Tooltip and accessibility evidence verifies that the complete canonical Path Reference remains available when the visible value is compacted.
- Header evidence verifies the visual order `[Bulk Pinning] [X/N pinned] [Close]` on both surfaces and verifies `0/N`, mixed `X/N`, and `N/N` states.
- UI Automation verifies Bulk Pinning retains Off, On, and Indeterminate states and its existing count-aware accessible action name.
- UI Automation verifies the Close button is exposed as an enabled button named `Close Manage Items` with its established compact logical interaction rectangle at the exercised DPI.
- Keyboard smoke verifies keyboard-opened initial focus on Bulk Pinning, forward traversal to Close and then every row Pin/Unpin and Remove Item action, reverse traversal, virtualized-row realization, wrapping, `Enter` and `Space` activation, and absence of a focus trap.
- Dismissal smoke independently exercises Close, `Escape`, outside click or native light dismissal, and Manage Items toggle-close. Each route must use normal cleanup and return focus when the originating control still exists.
- Guard smoke verifies the new Close action does not bypass active-drag or pending-mutation protections and does not leave stale projections, stale owner-open state, or a prematurely collapsing Edge Rail.
- Edge Rail smoke verifies the Flyout still holds the rail expanded, opens toward the desktop from both configured edges, preserves the foreground application's focus policy, and resumes normal collapse timing after closure.
- Drop Shelf smoke verifies the Popup remains anchored to its Shelf Batch, preserves one-at-a-time ownership, closes on shelf resize or owner removal under existing rules, and does not resize the Drop Shelf.
- Lifecycle smoke verifies individual Pin/Unpin, Bulk Pinning, Remove Item, accepted and canceled drag-out, Temporary consumption, Pinned retention, Missing cleanup, and Unavailable retention remain unchanged.
- Theme evidence covers Light and Dark from the same payload. High Contrast and non-default text scaling are exercised only in an isolated or safely reversible environment; unavailable configurations are recorded as unobserved rather than inferred.
- Long-list evidence verifies the fixed header remains visible, the item list scrolls vertically to the final row and back, horizontal scrolling remains unavailable, and closing releases deferred Shelf Item projections.
- Screenshots and captured bounds are human-review and qualification evidence, not permanent pixel baselines. No screenshot-regression framework is introduced.
- The established performance and residency gates remain inherited. UI Automation and performance measurement run separately against the same payload identity so automation-peer realization cannot contaminate memory evidence.
- Qualification records the executable and application payload fingerprints, Windows build, theme, contrast and text-scaling state, DPI, Edge Rail side, fixture identity, observed outcomes, failures, and unobserved scenarios. Evidence from another build cannot substitute.

## Out of Scope

- Changing the 280-logical-pixel expanded Edge Rail width, Rail Handle geometry, adaptive Edge Rail height, hover timing, collapse timing, monitor selection, topmost policy, fullscreen policy, or no-activate ownership.
- Changing the Manage Items maximum height, fixed-header behavior, vertical-only scrolling, desktop-facing placement, anchoring, one-at-a-time ownership, or open/close motion.
- Replacing the Drop Shelf Popup with a Flyout, replacing the Edge Rail Flyout with a Popup, or extracting one shared popup owner or lifecycle controller.
- Reordering item-row Pin/Unpin and Remove Item actions, changing their sizes, or moving them to the left side.
- Changing Bulk Pinning state transitions, lifecycle semantics, persistence, error handling, or the item-centric domain model.
- Adding a Pinned Batch concept, batch-owned pin state, schema migration, settings option, user-configurable popup width, or user-configurable path format.
- Opening, copying, editing, normalizing, resolving, probing, or otherwise acting on a Path Reference from the visible path text.
- Adding search, filtering, sorting, selection, context menus, launch actions, confirmations, undo, retry, notifications, or additional metadata to Manage Items.
- Changing source filesystem objects, drag engine behavior, destination acceptance, same-integrity limitations, Missing classification, or Unavailable retention.
- Adding a custom control library, CommunityToolkit dependency, new runtime package, permanent UI automation project, screenshot baseline system, telemetry, or background work.
- Creating implementation tickets, implementing the change, or creating a Git commit as part of this specification step. Those require the subsequent ticket workflow and explicit user approval.

## Further Notes

- `CONTEXT.md` already defines **Drop Shelf**, **Edge Rail**, **Shelf Batch**, **Shelf Item**, **Path Reference**, **Pinned Item**, **Temporary Item**, **Bulk Pinning**, and **Manage Items**. No new domain term is introduced.
- No ADR is required. The decisions are reversible presentation changes using existing WinUI Popup, Flyout, projection, styling, focus, and qualification seams; they do not change an architectural boundary or persistence owner.
- The authoritative rendered-UI seam was confirmed by the user: installed self-contained Release plus isolated profile and UI Automation, with the existing MSTest suite retained as regression coverage and no new UI test project.
- Earlier specifications remain historical and governing except for the width, path presentation, header order/copy, header focus-cycle, explicit-close, and narrowly conflicting out-of-scope clauses explicitly superseded here.
- All repository changes remain uncommitted until the user explicitly authorizes a Git commit.
