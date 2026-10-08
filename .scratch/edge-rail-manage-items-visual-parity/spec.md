# Edge Rail Manage Items Visual Parity

Status: ready-for-agent

## Problem Statement

The same multi-item Shelf Batch currently presents two visibly different Manage Items surfaces. The Drop Shelf opens a flat, solid popup with a subtle one-pixel boundary, eight-pixel corners, six-pixel inner padding, and its established item-row composition. The Edge Rail opens a native Flyout whose default presenter uses different background, padding, elevation, and subtle row alignment. Although both surfaces expose the same Shelf Items and lifecycle actions, they do not look like the same DropCove experience.

The difference is most visible in the outer chrome: the Edge Rail Flyout uses platform presenter defaults, including acrylic-style fill and default shadow/elevation, while the Drop Shelf popup is a flat solid surface. Smaller differences in header spacing, text inset, effective row separation, tooltips, and control-state rendering allow the two presentations to drift further.

Users need the Edge Rail Manage Items surface to match the Drop Shelf popup visually without changing the Edge Rail's compact workflow, desktop-facing placement, no-activate ownership, focus behavior, motion, drag behavior, lifecycle rules, or established popup bounds.

## Solution

Treat the Drop Shelf Manage Items popup as the visual source of truth and restyle the existing Edge Rail Manage Items Flyout to match its complete rendered presentation.

The Edge Rail surface uses the same solid theme-aware background, one-pixel card-stroke boundary, eight-pixel corner radius, six-pixel padding, fixed-header composition, item-row spacing, text alignment, typography, visual sizes, action sizes, and interaction-state language as the Drop Shelf popup. The native Flyout's default shadow is disabled so the Edge Rail surface remains flat rather than retaining a second elevation language.

Keep the Edge Rail Flyout as the owner of placement and lifetime. Preserve its current 320-to-480 logical-pixel width, maximum 480 logical-pixel height, desktop-facing Left/Right placement, vertical-only scrolling, rail-collapse suppression, no-activate behavior, keyboard focus contract, light dismissal, drag and mutation guards, and existing open/close motion. The Drop Shelf popup remains visually and behaviorally unchanged.

## User Stories

1. As a DropCove user, I want Manage Items to look like one product feature on the Drop Shelf and Edge Rail, so that changing shelf presentation does not make the same Shelf Batch feel unrelated.
2. As an Edge Rail user, I want the Drop Shelf popup to be the visual reference, so that the compact surface follows an already familiar presentation.
3. As an Edge Rail user, I want the Manage Items Flyout to use a solid theme-aware surface, so that it does not look like an unrelated acrylic system menu.
4. As an Edge Rail user, I want the Flyout background to match the Drop Shelf popup background, so that the content hierarchy is consistent.
5. As an Edge Rail user, I want the Flyout boundary to match the Drop Shelf's subtle one-pixel card stroke, so that both surfaces have the same edge definition.
6. As an Edge Rail user, I want the Flyout to use the same eight-pixel corner radius as the Drop Shelf popup, so that its geometry is consistent.
7. As an Edge Rail user, I want the Flyout to use the same six-pixel inner padding as the Drop Shelf popup, so that header and item content align consistently.
8. As an Edge Rail user, I want the native Flyout shadow disabled, so that it does not retain elevation absent from the Drop Shelf popup.
9. As an Edge Rail user, I do not want a custom replacement shadow, so that visual parity does not add another rendering treatment.
10. As a Drop Shelf user, I want the existing popup left unchanged, so that Edge Rail alignment does not regress the canonical presentation.
11. As an Edge Rail user, I want the fixed pin-summary header to align exactly like the Drop Shelf header, so that the same information appears in the same hierarchy.
12. As an Edge Rail user, I want the pin summary to retain the same typography as the Drop Shelf popup, so that the header does not appear heavier or lighter by surface.
13. As an Edge Rail user, I want the header gap and Bulk Pinning control alignment to match the Drop Shelf popup, so that mixed and all-pinned states occupy the same visual position.
14. As an Edge Rail user, I want the Bulk Pinning control to retain its existing 24-by-24 logical-pixel target, so that visual alignment does not reduce usability.
15. As an Edge Rail user, I want each Shelf Item row to use the same background, padding, and rounded geometry as the Drop Shelf popup, so that rows read as the same presentation.
16. As an Edge Rail user, I want the same effective separation between adjacent Shelf Item rows, so that list density does not change when I switch surfaces.
17. As an Edge Rail user, I want each row's visual, text, and actions to use the same column spacing as the Drop Shelf popup, so that content aligns consistently.
18. As an Edge Rail user, I want the text block inset to match the Drop Shelf popup, so that names and Path References do not sit closer to the visual in one surface.
19. As an Edge Rail user, I want the Shelf Item name to retain the same semibold size and trimming behavior, so that primary identity is consistent.
20. As an Edge Rail user, I want the Path Reference and availability/lifecycle details to retain the same secondary typography, so that metadata has one hierarchy.
21. As an Edge Rail user, I want trimmed names, paths, and details to expose the same tooltip behavior, so that visual parity does not hide information.
22. As an Edge Rail user, I want the item thumbnail or native icon to remain 24 logical pixels, so that visual recognition remains unchanged.
23. As an Edge Rail user, I want Pin/Unpin and Remove Item controls to retain their existing 24-by-24 logical-pixel targets, so that matching the Drop Shelf does not reduce interaction size.
24. As an Edge Rail user, I want Pin/Unpin and Remove Item glyph sizes and alignment to match the Drop Shelf popup, so that row actions do not appear offset or differently weighted.
25. As a pointer user, I want normal, hover, pressed, and disabled action states to match the Drop Shelf popup, so that control feedback is predictable.
26. As a keyboard user, I want visible focus treatment to remain legible and consistent with the Drop Shelf popup, so that the visual update does not weaken navigation.
27. As an assistive-technology user, I want existing accessible names, ToggleState values, and item/action relationships preserved, so that restyling does not change meaning.
28. As a Light-mode user, I want both Manage Items surfaces to resolve through the same theme-aware resources, so that parity is not limited to Dark mode.
29. As a Dark-mode user, I want both Manage Items surfaces to preserve the same background, boundary, text, and control hierarchy, so that the Flyout does not revert to acrylic styling.
30. As a High Contrast user, I want the Flyout background, boundary, row surfaces, glyphs, mixed pin state, and focus cues to remain system-resolved and distinguishable, so that parity does not reduce accessibility.
31. As an Edge Rail user, I want Manage Items to retain its current 320-to-480 logical-pixel width contract, so that visual alignment does not change usable content width.
32. As an Edge Rail user, I want Manage Items to remain bounded to 480 logical pixels in height, so that long Shelf Batches do not cover the work area.
33. As an Edge Rail user, I want only Shelf Item rows to scroll while the header remains fixed, so that Bulk Pinning stays available.
34. As an Edge Rail user, I want vertical scrolling retained and horizontal scrolling disabled, so that long paths cannot destabilize the layout.
35. As a Left-edge user, I want Manage Items to continue opening toward the desktop, so that visual changes do not place it off-screen.
36. As a Right-edge user, I want Manage Items to continue opening toward the desktop, so that placement remains mirrored correctly.
37. As an Edge Rail user, I want the Flyout to remain constrained by the selected monitor work area through its existing placement behavior, so that every action remains reachable.
38. As a Windows user, I want direct Edge Rail interaction to preserve the foreground application's focus, so that visual parity does not change the no-activate workflow.
39. As a pointer user, I do not want opening Manage Items to force keyboard focus, so that pointer interaction remains non-disruptive.
40. As a keyboard user, I want keyboard-opened Manage Items to focus Bulk Pinning first, so that the existing focus contract remains unchanged.
41. As a keyboard user, I want Tab and Shift+Tab traversal, wrapping, and on-demand row realization preserved, so that restyling cannot create a focus trap.
42. As a keyboard user, I want Escape to close Manage Items and restore focus to the originating Manage Items control, so that transient navigation remains predictable.
43. As an Edge Rail user, I want light dismissal and toggle-close behavior preserved, so that the Flyout remains familiar.
44. As an Edge Rail user, I want an open Manage Items Flyout to continue holding the Edge Rail expanded, so that its owner cannot collapse while I use it.
45. As an Edge Rail user, I want active item drag and pending mutation guards preserved, so that the surface cannot disappear during an operation.
46. As a user with reduced-motion preferences, I want the existing Flyout and Edge Rail motion policy preserved, so that this visual change adds or removes no motion contract.
47. As an Edge Rail user, I want Pin/Unpin, Bulk Pinning, Remove Item, direct drag-out, Successful Drag-Out, cancellation, rejection, Missing cleanup, and Unavailable retention to behave exactly as before, so that presentation parity does not alter lifecycle semantics.
48. As a DropCove user, I want all presentation changes to remain reference-only and never modify source filesystem objects, so that DropCove remains a temporary workspace rather than a file manager.
49. As a performance-conscious user, I want the visual update to add no timer, polling loop, background worker, custom backdrop, retained visual cache, or new runtime dependency, so that a styling change does not increase idle work or native residency.
50. As a maintainer, I want existing surface-specific popup/flyout ownership and event handlers preserved, so that visual parity does not couple the Drop Shelf and Edge Rail lifecycles.
51. As a maintainer, I want existing shared theme resources, action styles, visual templates, and presentation projections reused, so that parity does not create a second design system.
52. As a maintainer, I do not want a shared interactive row controller, command bus, forwarding coordinator, or new public interface introduced for a presentation-only change, so that the implementation remains direct.
53. As a maintainer, I want the existing automated suite to remain the regression authority for lifecycle, persistence, sizing, placement, drag, and visual-resource behavior, so that visual acceptance does not weaken established tests.
54. As a maintainer, I want the installed self-contained Release to be the acceptance authority for actual WinUI rendering, default-shadow suppression, geometry, focus, accessibility, and interaction, so that source declarations are not mistaken for rendered proof.
55. As a maintainer, I want same-build screenshots used only as human-review evidence, so that DropCove does not acquire a brittle pixel-baseline framework tied to Windows rendering revisions.
56. As a maintainer, I want unavailable theme, DPI, monitor, or High Contrast scenarios recorded as unobserved rather than inferred, so that qualification claims remain evidence-based.

## Implementation Decisions

- This specification deliberately supersedes only the prior requirement that the existing Edge Rail multi-item Flyout visual treatment remain unchanged. The established Flyout dimensions, placement, ownership, focus, dismissal, motion, scrolling, drag behavior, mutation behavior, deferred projections, resource lifetime, no-activate policy, and rail-collapse suppression remain governing.
- The Drop Shelf Manage Items popup is the visual source of truth. Its user-visible presentation is not redesigned by this feature.
- The Edge Rail continues to use its existing native Flyout and FlyoutPresenter ownership. It is not converted to the Drop Shelf's custom Popup, and the Drop Shelf is not converted to a Flyout.
- The FlyoutPresenter uses the same existing solid base fill and card-stroke theme resources as the Drop Shelf popup. It does not use the default acrylic-style Flyout background or introduce a new palette.
- The outer Edge Rail presenter uses a one-logical-pixel boundary, eight-logical-pixel corner radius, and six-logical-pixel content padding, matching the Drop Shelf popup.
- The FlyoutPresenter's default shadow is disabled through the platform's existing default-shadow control. No ThemeShadow, custom composition shadow, helper window, Mica, Acrylic, or backdrop replacement is added.
- Existing Flyout open/close motion remains enabled and follows current reduced-motion behavior. This feature does not add a custom transition and does not remove the current native Flyout transition.
- The Flyout's width remains content-sized from 320 through 480 logical pixels. Its total height remains bounded at 480 logical pixels. The header consumes part of that bound and the item list receives the remaining height.
- Placement remains desktop-facing for both Left and Right Edge Rail configurations and continues using the current monitor/work-area behavior. Visual changes do not add a second geometry algorithm.
- The fixed header matches the Drop Shelf popup's rendered composition: 12-logical-pixel summary text, eight-logical-pixel header column gap, 24-by-24 tri-state Bulk Pinning control, and four-logical-pixel separation from the item scroller.
- Edge Rail Shelf Item rows match the Drop Shelf popup's rendered composition: four-logical-pixel row padding, four-logical-pixel corners, the existing secondary control fill, 24-logical-pixel visual, four-logical-pixel column spacing, matching text inset, 10-logical-pixel semibold name, 9-logical-pixel secondary metadata, and 24-by-24 Pin/Unpin and Remove Item controls.
- The list produces the same effective row-to-row separation as the Drop Shelf popup. Implementation must compare rendered geometry rather than independently copying one margin or layout-spacing value that produces a different total gap.
- Name, Path Reference, and details trimming and tooltip behavior match the Drop Shelf popup. Existing accessible names and automation semantics remain authoritative.
- Normal, pointer-over, pressed, disabled, checked, indeterminate, and focused control states continue to use the existing shared action-style and theme-resource language. The Edge Rail's no-activate and pointer-focus rules remain unchanged even where the rendered state matches the Drop Shelf.
- Light, Dark, and High Contrast resolve through existing WinUI theme resources. No fixed RGB/ARGB values or theme-specific exception is introduced.
- The current surface-specific Shelf Item templates, popup/flyout ownership, and event handlers remain separate. This feature aligns their rendered contract but does not extract a shared interactive row template or add a new presentation controller merely to remove small XAML duplication.
- Existing shared visual templates, presentation projections, button styles, and Bulk Pinning visual resources remain the reuse seams. A new public module, interface, adapter, command abstraction, or compatibility layer is not introduced.
- No Shelf Item, Shelf Batch, Path Reference, lifecycle, availability, settings, display-state, sizing-state, or persistence schema changes are required.
- No new NuGet package, runtime dependency, UI test framework, screenshot-baseline framework, telemetry, timer, polling loop, cache, or background worker is introduced.
- Existing performance contracts remain unchanged: total process WorkingSet64 stays below the established 160 MB gate under the canonical 100 Shelf Batch / 1,000 Shelf Item protocol, idle CPU remains at or below 0.1%, and shelf-show latency p95 remains at or below 150 ms. This feature does not claim to solve, lower, or waive those gates.
- User-facing documentation is updated after implementation to state that Manage Items now shares one visual language across Drop Shelf and Edge Rail while retaining surface-specific placement and behavior.

## Testing Decisions

- Good tests and qualification evidence assert user-observable rendered output and behavior. They do not assert XAML source text, private element names, resource-key forwarding, exact template structure, handler forwarding, or duplicated style constants.
- The single authoritative acceptance seam is the installed self-contained Release running with an isolated test profile. This is the highest existing seam that can prove WinUI Flyout rendering, default-shadow suppression, theme resolution, geometry, focus, accessibility, no-activate behavior, and interaction together.
- The seam was confirmed with the user. No new permanent visual-test seam is planned.
- The acceptance fixture contains one representative mixed-lifecycle multi-item Shelf Batch with an image, a non-image file, a folder, long names, a long Path Reference, Pinned and Temporary Items, and enough Shelf Items to require vertical scrolling.
- The same Shelf Batch is opened through Manage Items on the Drop Shelf and Edge Rail from the same build and isolated profile. Human-reviewed screenshots and captured bounds compare the complete outer surface, header, first item row, adjacent-row gap, long-text trimming, final visible row, and action alignment.
- Visual evidence verifies that the Edge Rail Flyout uses the same solid background, one-pixel boundary, eight-pixel corners, six-pixel padding, and flat no-shadow treatment as the Drop Shelf popup. Default Flyout acrylic and elevation must not remain visible.
- Visual evidence verifies matching header typography, header gap, Bulk Pinning alignment, row background, row padding, row corner radius, text inset, visual size, title and metadata hierarchy, effective row separation, and Pin/Unpin and Remove Item alignment.
- Pointer smoke verifies normal, hover, pressed, disabled, checked, and indeterminate states where applicable without changing hit targets or initiating unintended drag.
- UI Automation verifies the existing accessible names, enabled/busy state, ToggleState Off/On/Indeterminate, logical 24-by-24 action rectangles at the exercised DPI, focus visibility, Flyout ownership, and the originating Manage Items focus return.
- Keyboard smoke verifies Enter and Space activation, pointer opening without forced focus, keyboard opening with initial Bulk Pinning focus, Tab and Shift+Tab traversal and wrapping, Escape closure, and absence of a focus trap.
- Placement smoke opens the Flyout from both Left and Right Edge Rail configurations and verifies desktop-facing placement, work-area reachability, unchanged 320-to-480 logical-pixel width, maximum 480 logical-pixel height, fixed header, vertical scrolling, and disabled horizontal scrolling.
- Lifecycle smoke verifies that the Flyout keeps the Edge Rail expanded, remains coherent during Pin/Unpin and Bulk Pinning, respects pending mutation and drag guards, closes under its existing owner rules, releases item projections on close, and resumes normal collapse timing afterward.
- Drag smoke verifies direct Shelf Item drag-out, cancellation, rejection, Successful Drag-Out handling, Temporary consumption, Pinned retention, Missing cleanup, and Unavailable retention remain unchanged.
- Motion smoke verifies the existing Flyout transition remains present under ordinary animation settings and that established reduced-motion behavior remains functional. No new animation timing is introduced.
- Light and Dark evidence uses the same qualified payload and records both Manage Items surfaces with the representative fixture. High Contrast runs only in an isolated or safely reversible environment and verifies boundaries, text, mixed pin state, action glyphs, and visible focus by more than color alone.
- Screenshots and bounds are human-review evidence, not permanent pixel baselines. No screenshot-regression framework is introduced.
- The complete existing automated suite remains green and remains the regression authority for DropShelfManager lifecycle and persistence, ShelfVisualCoordinator resource behavior, Edge Rail sizing, placement calculations, drag lifecycle, settings, and window interop. Existing tests are not rewritten to assert presentation implementation details.
- No permanent unit test is added merely to prove that a presenter sets a brush, border, corner radius, padding, or default-shadow property. The installed Release proves those rendered outcomes.
- Qualification records the executable and application payload fingerprints, Windows build, theme, contrast mode, DPI, rail edge, fixture identity, exercised scenarios, observed outcomes, and unobserved configurations. Evidence from another build cannot substitute.
- The established performance gate remains inherited. Final qualification must not use forced collection, working-set trimming, reduced fixtures, alternate memory metrics, or automation-contaminated samples to hide a regression.

## Out of Scope

- Restyling or behaviorally changing the Drop Shelf Manage Items popup.
- Changing the Manage Items glyph, trigger position, accessible action meaning, or multi-item-only visibility.
- Replacing the Edge Rail Flyout with the Drop Shelf Popup, replacing the Drop Shelf Popup with a Flyout, or making both surfaces share one owner control.
- Extracting a shared interactive Shelf Item row template, shared action controller, management coordinator, command bus, public mutation interface, or compatibility wrapper.
- Changing the Flyout's 320-to-480 logical-pixel width, 480 logical-pixel maximum height, desktop-facing placement, work-area behavior, fixed header, or vertical-only scrolling.
- Changing no-activate behavior, topmost policy, monitor selection, fullscreen policy, Rail Handle geometry, expanded Edge Rail geometry, adaptive sizing, hover timing, or collapse timing.
- Changing keyboard activation, focus order, focus restoration, light dismissal, toggle-close behavior, owner-close rules, pending-operation guards, or rail-collapse suppression.
- Removing the existing native Flyout open/close transition, adding custom popup animation, or changing reduced-motion policy.
- Changing Pin/Unpin, Bulk Pinning, Remove Item, Remove Batch, Clear Temporary Items, drag-out, Successful Drag-Out, Missing, Unavailable, or Path Reference semantics.
- Changing thumbnail/native-icon policy, deferred Shelf Item projections, visual request cancellation, cache policy, or resource lifetime.
- Adding search, filtering, selection, launch/open actions, new metadata, labels, confirmations, undo, retries, or notifications.
- Adding Mica, Acrylic, a custom backdrop, custom shadow, helper window, custom window region, hard-coded palette, or decorative effect.
- Adding a new runtime dependency, UI test framework, screenshot-regression framework, telemetry, timer, polling loop, cache, or background worker.
- Changing, lowering, replacing, or waiving the established residency, idle CPU, or shelf-show latency gates.
- Creating a Git commit. Repository changes remain uncommitted until the user explicitly authorizes a commit.

## Further Notes

- `CONTEXT.md` already defines **Drop Shelf**, **Edge Rail**, **Shelf Batch**, **Shelf Item**, and **Manage Items**. This specification uses those canonical terms. “Chevron” remains only an informal description of the current icon and is not introduced into the domain glossary.
- The current Windows App SDK exposes FlyoutPresenter default-shadow control, so the accepted flat result can be implemented without replacing Flyout ownership. Installed observation remains mandatory because setting the property is not itself rendered proof.
- This specification is the deliberate later requirement permitted by the compact Edge Rail specification. It supersedes only the clauses that freeze the multi-item Flyout's visual treatment and item-layout presentation. All established bounds, placement, ownership, lifecycle, focus, motion, drag, scrolling, and Drop Shelf behavior remain unchanged.
- The earlier Edge Rail Thumbnail and Management Parity specification remains authoritative for Shelf Item content, visual-provider lifetime, lifecycle actions, focus traversal, drag behavior, and performance constraints except where this specification explicitly changes outer chrome and rendered spacing/alignment.
- No ADR is required. The change is a reversible presentation contract using existing WinUI styling seams and does not change domain boundaries, persistence ownership, native-window ownership, deployment, or a hard-to-reverse integration.
- The confirmed testing seam is installed self-contained Release qualification with same-build visual evidence and UI Automation. Existing automated tests remain regression coverage; no new public test seam is introduced.
- This specification does not authorize implementation, ticket creation, or a Git commit. Those require the subsequent ticket workflow and explicit user approval.