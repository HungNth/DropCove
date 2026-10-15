# DropCove — Lucide Vue Migration and Shelf Batch Spacing Correction

Status: ready-for-human
Design confirmation: approved by user on 2026-10-15
Testing seams: actual Windows Resident Instance and real WebView2 surfaces, supported by the existing frontend production build and native Rail geometry checks; approved by user on 2026-10-15
Implementation: authorized by user through the implement workflow on 2026-10-15; source implementation and automated verification complete, final interactive acceptance pending; no separate Git commit authorization
Predecessor: [DropCove 0.0.12 — Compact Scrolling and Lucide Icon Consistency](../compact-scrolling-lucide-consistency/spec.md)
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)
Research: [Lucide package migration research](research.md)
Verification: [Implementation evidence, review, and cleanup limits](verification.md)

## Problem Statement

DropCove currently depends on `lucide-vue-next`, which npm has deprecated in favor of `@lucide/vue`. The application uses the deprecated package through its existing action-icon adapter even though the supported replacement provides the same Vue component API and all nine icon exports DropCove needs. Keeping the deprecated package leaves the frontend on an obsolete package identity and prevents the dependency graph from following Lucide's supported Vue package.

The populated Drop Shelf and expanded Edge Rail also place Shelf Batch cards too close to the native vertical scrollbar. The current horizontal collection insets differ between the two surfaces, and the right-side inset does not provide the approved visual separation before the scrollbar channel. Their vertical collection clearances also do not present the newly approved balanced 8-pixel top and bottom relationship. The Drop Shelf currently uses different top and bottom values, while the Edge Rail's earlier 4-pixel boundary contract is now intentionally superseded for these two Shelf surfaces.

The spacing correction must not undo the scrolling work completed in 0.0.12. Shelf Batch content must remain clipped behind a fixed boundary while scrolling, virtual coordinates must remain zero-based against the real scrollport, short collections must stay top-aligned, and the final Shelf Batch must remain reachable. The expanded Edge Rail height budget must also grow with the new clearance. Keeping its old 640-pixel maximum would make eleven batches overflow by two pixels, contradicting the accepted five-through-eleven no-scroll tier.

Users need the supported Lucide package and visibly balanced Shelf surfaces without an icon redesign, a custom scrollbar, changed Shelf behavior, altered collection density, premature Edge Rail scrolling, or rewritten historical acceptance records.

## Solution

Replace the deprecated Lucide Vue dependency with `@lucide/vue` at the approved compatible range and regenerate the pnpm lockfile. Keep the existing action-icon adapter and all of its consumers unchanged apart from the package import source. Preserve the current icon roles, glyph sizes, filled and mixed pin treatments, decorative semantics, accessible button ownership, and control dimensions.

Give the Drop Shelf symmetric 7-pixel collection insets and the expanded Edge Rail symmetric 6-pixel collection insets. Measure equality from the inner surface border to the leftmost card and from the rightmost card to the native scrollbar channel. These insets remain present with and without overflow; the scrollbar appears only when needed.

For both surfaces, establish 8 pixels of top and bottom collection clearance. Keep the top clearance outside the actual scrollport so Shelf Batch cards cannot paint into it at intermediate scroll positions. Keep the bottom clearance inside the scroll range so the final Shelf Batch has the matching 8-pixel clearance at maximum scroll. Preserve top alignment for short collections rather than centering or distributing Shelf Batches through unused height.

Retain the Drop Shelf's 4-pixel Shelf Batch gap, 164-pixel minimum card width, responsive auto-fill columns, horizontal-overflow suppression, resizing, and virtualization behavior. Retain the expanded Edge Rail's 48-pixel rows, 4-pixel gaps, 280-pixel width, header and footer controls, Left and Right placement, and no-activate behavior.

Update expanded Edge Rail height to `max(280, min(642, 52 × batch count + 70))`. One through four batches remain at 280 logical pixels, five batches use 330 pixels, eleven batches use 642 pixels without vertical overflow, and twelve or more batches clamp at 642 pixels and scroll. Synchronize the living Product Specification with the new 642-pixel maximum, `+70` static budget, and twelve-or-more scrolling contract when the implementation lands. Preserve the resolved 0.0.12 specification, tickets, roadmap evidence, measurements, and artifact hashes as historical records of the earlier implementation.

## User Stories

1. As a DropCove user, I want the application to use Lucide's supported Vue package, so that the interface no longer depends on a deprecated package.
2. As a DropCove maintainer, I want the deprecated package removed completely, so that the dependency graph has one authoritative Lucide Vue package.
3. As a DropCove maintainer, I want the replacement package recorded in the pnpm lockfile, so that installations resolve the same supported dependency graph.
4. As a DropCove maintainer, I want the existing icon adapter retained, so that package ownership stays centralized instead of spreading imports across components.
5. As a user viewing action controls, I want the same Pin, Minus, List, Circle Minus, Close, Trash, directional arrow, and downward arrow glyphs, so that the package migration does not change icon language.
6. As a user viewing a pinned action, I want the Pin glyph to retain its filled state, so that pinned and unpinned references remain visually distinct.
7. As a user viewing mixed Bulk Pinning state, I want the composed Pin and Minus treatment preserved, so that the visual state continues to match the mixed checkbox state.
8. As a screen-reader user, I want buttons to retain their accessible names and states while glyphs remain decorative, so that the package migration does not weaken accessibility.
9. As a keyboard user, I want focus order and activation behavior to remain unchanged, so that replacing the package does not alter interaction.
10. As a pointer user, I want existing tooltips, hover states, and 24-pixel action targets preserved, so that compact controls remain understandable and practical.
11. As a user viewing the empty Drop Shelf, I want the existing 28-pixel downward glyph and `Drop here` message preserved, so that the empty state remains familiar.
12. As a Drop Shelf user, I want equal 7-pixel left-border and right-scrollbar-channel clearances, so that Shelf Batch cards appear horizontally balanced.
13. As an Edge Rail user, I want equal 6-pixel left-border and right-scrollbar-channel clearances, so that Shelf Batch rows appear horizontally balanced.
14. As a user with content that fits without scrolling, I want the right inset to remain stable, so that Shelf Batch cards do not shift when overflow later appears.
15. As a user with overflowing content, I want the native scrollbar to appear automatically, so that the spacing correction does not force an unnecessary scrollbar channel.
16. As a Drop Shelf user, I want the existing 7-pixel left inset preserved, so that the collection's accepted left alignment remains stable.
17. As an Edge Rail user, I want the existing 6-pixel left inset preserved, so that compact Rail rows retain their accepted left alignment.
18. As a Drop Shelf user, I want the first Shelf Batch separated from the top viewport edge by 8 pixels, so that the collection does not feel pressed against the surface.
19. As a Drop Shelf user at maximum scroll, I want the final Shelf Batch separated from the bottom viewport edge by 8 pixels, so that top and bottom collection clearance feel balanced.
20. As an Edge Rail user, I want the first Shelf Batch row separated from the top viewport edge by 8 pixels, so that the expanded Rail has the same balanced collection treatment.
21. As an Edge Rail user at maximum scroll, I want the final Shelf Batch row separated from the bottom viewport edge by 8 pixels, so that it remains fully visible and balanced.
22. As a user scrolling through partially visible Shelf Batches, I want content clipped behind the fixed top boundary, so that cards never paint against the containing border.
23. As a user with a short collection, I want Shelf Batches to remain top-aligned, so that unused height does not move or spread the collection unexpectedly.
24. As a Drop Shelf user resizing the window, I want the existing 164-pixel minimum card width preserved, so that the spacing correction does not make cards too narrow.
25. As a Drop Shelf user at representative one-, two-, and three-column widths, I want responsive auto-fill columns preserved, so that the grid continues to adapt naturally.
26. As a Drop Shelf user near a responsive threshold, I want native automatic-growth capacity to match the grid's symmetric insets, so that resizing does not introduce fragile width compensation.
27. As a Drop Shelf user, I want the existing 4-pixel gap between Shelf Batch cards preserved, so that the correction does not redesign collection density.
28. As an Edge Rail user, I want 48-pixel Shelf Batch rows and 4-pixel row gaps preserved, so that compact Rail density remains unchanged.
29. As a user with an ordinary collection, I want the corrected spacing to work without virtualization, so that small collections behave correctly.
30. As a user with more than fifty Shelf Batches, I want the same spacing while virtualization is active, so that collection size does not change presentation.
31. As a keyboard user traversing virtualized Shelf Batches, I want focused controls revealed within the corrected viewport, so that focus is not hidden by the fixed top boundary.
32. As a user at maximum scroll, I want the final Shelf Batch fully reachable without extra blank range, so that corrected padding does not corrupt scroll bounds.
33. As a user dragging Shelf Items or Shelf Batches, I want drag-in and Copy-only drag-out behavior preserved, so that visual spacing does not affect file safety.
34. As a Drop Shelf user with Temporary Items, I want the Clear Temporary Items control to remain in its existing footer, so that collection spacing does not move or redesign the action.
35. As a user viewing a status message, I want status presentation to remain outside the collection contract, so that balanced viewport clearance does not alter error or warning layout.
36. As an Edge Rail user with one through four batches, I want the expanded Rail to remain 280 pixels high, so that small collections keep the compact minimum tier.
37. As an Edge Rail user with five batches, I want the expanded Rail to use 330 pixels, so that the new 8-pixel clearances fit without premature scrolling.
38. As an Edge Rail user with five through eleven batches, I want the Rail to grow through bounded tiers without a scrollbar, so that each Shelf Batch remains directly visible.
39. As an Edge Rail user with eleven batches, I want the Rail to reach 642 pixels without overflow, so that the previous two-pixel height deficit is not reintroduced.
40. As an Edge Rail user with twelve or more batches, I want the Rail capped at 642 pixels with vertical scrolling, so that the surface remains bounded.
41. As an Edge Rail user whose Shelf receives a new batch while expanded, I want the existing live-growth behavior preserved, so that the Rail follows the updated height tiers.
42. As an Edge Rail user removing content, I want the existing no-live-shrink behavior preserved, so that removal does not unexpectedly move the active surface.
43. As a user docking the Edge Rail on either side, I want Left and Right placement and desktop-facing controls preserved, so that spacing does not change placement semantics.
44. As a user interacting with the Edge Rail, I want it to remain no-activate and avoid stealing application focus, so that the geometry change does not alter native window behavior.
45. As a user switching between collapsed and expanded Rail states, I want the Rail Handle unchanged, so that only the expanded collection receives the spacing correction.
46. As a user viewing compact Rail rows, I want titles and action controls to remain readable within the 280-pixel width, so that the larger right inset does not cause unacceptable truncation or overlap.
47. As a user in Dark presentation, I want the corrected clearances visible without changing scrollbar colors or surface styling, so that the change remains geometric.
48. As a user in Light presentation, I want the corrected clearances visible without changing the existing semantic theme treatment, so that presentation remains consistent.
49. As a Windows Contrast user, I want the existing system-color scrollbar and control treatment preserved, so that the spacing correction does not regress forced-colors behavior.
50. As a maintainer, I want virtual collection width derived from computed scrollport padding, so that the new right inset requires no hardcoded component offset.
51. As a maintainer, I want virtual coordinates to remain zero-based against the real scrollport, so that top clearance does not leak into shared collection calculations.
52. As a maintainer, I want the Edge Rail height formula updated with the vertical spacing in the same change, so that CSS geometry and native window geometry cannot diverge.
53. As a maintainer, I want the living Product Specification synchronized with the shipped geometry, so that canonical product documentation does not retain the old 640-pixel and `+62` contract.
54. As a maintainer, I want the completed 0.0.12 planning and acceptance records preserved unchanged, so that historical measurements and artifact evidence remain truthful.
55. As a project owner, I want specification, ticket approval, implementation approval, final acceptance, and Git commit authorization to remain separate gates, so that publishing this spec does not authorize unrelated work.

## Implementation Decisions

- Replace `lucide-vue-next` with `@lucide/vue` using the approved compatible range `^1.57.0`, and regenerate the pnpm lockfile without upgrading unrelated dependencies.
- Remove the deprecated package completely. Do not retain an alias, compatibility shim, fallback import, or parallel package installation.
- Keep the existing action-icon adapter as the sole direct Lucide import boundary. Consumers continue using the adapter's current role-based interface.
- Preserve the nine current icon components and their existing mapping: Pin, Minus, List, Circle Minus, Close, Trash, Left, Right, and Down.
- Preserve 14-by-14-pixel action glyphs, the 28-by-28-pixel empty-state glyph, 24-by-24-pixel action targets, filled Pin state, mixed Pin/Minus composition, tooltips, accessible names, pressed and checked states, disabled states, and focus behavior.
- Accept Lucide v1's default decorative SVG behavior because accessible names and interaction states remain owned by the surrounding controls.
- Limit collection-spacing changes to the populated Drop Shelf and expanded Edge Rail. Manage Items, Settings, and the collapsed Rail Handle remain unchanged.
- Keep native WebView2 scrollbars, automatic overflow, the existing 8-pixel interaction channel, approximately 4-pixel rounded visible thumb, semantic colors, hover/active states, transparent track, and Windows Contrast system-color mappings. Only the Drop Shelf and expanded Edge Rail thumb painting is aligned to the content-side channel edge.
- Set both Drop Shelf horizontal collection insets to 7 logical pixels, with and without overflow.
- Set both expanded Edge Rail horizontal collection insets to 6 logical pixels, with and without overflow.
- Require right clearance to both the scrollbar channel and the painted thumb to equal the left card clearance. On Drop Shelf and expanded Edge Rail only, redistribute the thumb's transparent horizontal border to the right side so the 4-pixel painted thumb starts at the channel's content edge. Preserve its rounded painted shape and 8-pixel hit channel; do not change collection padding or add overflow-measurement logic.
- Set both the Drop Shelf and expanded Edge Rail to 8 logical pixels of top collection clearance and 8 logical pixels of bottom collection clearance.
- Keep top clearance outside the actual scrollport as a fixed clipping boundary. Do not replace it with scrollable top padding, a sticky spacer, or an overlay.
- Keep bottom clearance inside the scroll range so the final Shelf Batch receives the full 8-pixel clearance at maximum scroll.
- Keep ordinary and virtualized collections top-aligned. Do not center short collections or distribute unused height between Shelf Batches.
- Preserve zero-based virtual coordinates, measured scroll offsets, bounded mounting, total-height calculation, focus retention, and keyboard reveal against the real inner scrollport.
- Continue deriving usable virtual collection width and horizontal origin from computed left and right scrollport padding. Do not add Drop Shelf- or Edge Rail-specific padding parameters to the shared virtualization interface.
- Preserve the Drop Shelf grid's 164-pixel minimum card width, responsive auto-fill columns, 4-pixel card gap, horizontal-overflow suppression, automatic growth, and resizing behavior.
- Match native automatic-growth capacity to the grid's symmetric inset budget: without a scrollbar, two columns start at 348 pixels and three at 516 pixels. Do not add width compensation or reduce minimum card width. Retain the approved 191-pixel minimum window width as a conservative floor.
- Preserve expanded Edge Rail width at 280 logical pixels, Shelf Batch row height at 48 logical pixels, row gap at 4 logical pixels, current header and footer dimensions, and existing control placement.
- Confirm that the reduced usable Rail row width still renders Shelf Batch content and action controls without overlap or unacceptable truncation.
- Change expanded Edge Rail target height to `max(280, min(642, 52 × batch count + 70))`.
- Preserve 280 logical pixels for zero through four batches in geometry calculations; at runtime, an empty Shelf continues to hide the Edge Rail.
- Use 330 logical pixels for five batches and 642 logical pixels for eleven batches without vertical overflow.
- Clamp twelve or more batches to 642 logical pixels and retain native vertical scrolling with the final Shelf Batch reachable.
- Preserve live growth when new content is added to an expanded Rail and preserve the existing policy that removal does not live-shrink the Rail until the next expansion.
- Preserve Edge Rail placement ownership, explicit and remembered monitor behavior, temporary primary-display fallback, Left and Right docking, fullscreen policy, no-activate behavior, expansion and collapse delays, and management/confirmation holds.
- Synchronize the living Product Specification in the same implementation change: update the maximum expanded height, target-height formula, five-through-eleven tier behavior, and twelve-or-more scrolling statement to the new contract.
- Preserve the completed 0.0.12 roadmap entry, resolved specification, tickets, recorded measurements, executable hashes, and acceptance statements as historical evidence. The new specification supersedes those values only for the follow-up implementation.
- Do not add a glossary entry or ADR. Lucide package identity, scrollport padding, viewport clearance, and Rail height constants are reversible implementation and product-presentation details, not new domain concepts or architectural ownership decisions.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- The highest acceptance seam is the actual running Windows Resident Instance using the real WebView2 Drop Shelf and expanded Edge Rail. This proves native scrollbar placement, clipping, transparent-window presentation, responsive layout, virtualization, native window sizing, and controls together.
- Supporting seam one is the existing frontend production dependency and build path. Verify a frozen lockfile installation with strict peer dependency enforcement, then run complete Vue single-file-component typechecking and production bundling.
- Supporting seam two is the existing native Rail geometry coverage. Update its externally observable height expectations and run the focused geometry checks plus the complete existing native regression suite.
- Build native code through cargo-xwin for `x86_64-pc-windows-msvc`; do not introduce a Visual Studio or commercial MSVC Build Tools dependency.
- Good tests assert user-visible geometry, overflow, reachability, icon behavior, and native height outcomes. They do not assert Tailwind class strings, source text, forwarding, or implementation-only DOM structure.
- Verify the deprecated package is absent from the resolved dependency graph, `@lucide/vue` resolves at the lockfile-selected 1.x version, and no unrelated direct dependency version changes.
- Exercise every current icon role through its real control or empty state. Confirm glyph geometry, filled Pin, mixed Pin/Minus, decorative SVG behavior, accessible button names and states, tooltips, focus, and disabled behavior.
- Exercise the Drop Shelf with fitting and overflowing content. Assert equal 7-pixel left-border and right-to-painted-thumb clearances; verify actual painted screenshot pixels as well as computed geometry. Retain the fixed 8-pixel top boundary and 8-pixel maximum-scroll bottom clearance.
- Exercise the Drop Shelf at minimum width and representative one-, two-, and three-column widths. Confirm the 164-pixel minimum card width, 4-pixel gaps, expected threshold movement, no horizontal overflow, and unchanged resizing.
- Exercise ordinary Drop Shelf collections with at most fifty Shelf Batches and virtualized collections with more than fifty Shelf Batches at the start, middle, and maximum-scroll positions.
- Confirm partially visible Shelf Batch cards cannot paint into the fixed top boundary and the final Shelf Batch remains fully reachable without extra blank scroll range.
- Exercise the expanded Edge Rail with one through four, five, eleven, twelve, and more than fifty Shelf Batches.
- Confirm exact Rail heights: 280 pixels for one through four batches, 330 pixels for five, and 642 pixels for eleven and twelve-or-more after clamping.
- Confirm eleven batches do not overflow, twelve batches do overflow, and the twelfth and final rows are fully reachable at maximum scroll.
- Exercise the Edge Rail with fitting and overflowing content. Assert equal 6-pixel left-border and right-to-painted-thumb clearances; verify actual painted screenshot pixels as well as computed geometry. Retain the fixed 8-pixel top boundary and 8-pixel maximum-scroll bottom clearance.
- Confirm the 280-pixel Rail width leaves Shelf Batch titles and action controls usable without overlap or unacceptable truncation with the symmetric 6-pixel insets and scrollbar channel.
- Exercise Left and Right docking, expansion and collapse, Open Shelf, Clear Temporary Items, Manage Items, pointer entry and leave, and the no-activate contract to ensure the geometry change does not alter interaction.
- Exercise keyboard focus reveal across ordinary and virtualized collections. Confirm focus remains visible within the bounded viewport and shared virtual calculations require no component-specific offset.
- Spot-check Dark, Light, and actual Windows Contrast presentation for unchanged scrollbar and control styling while measuring the new geometry. Color policy itself is not being redesigned.
- Preserve and rerun existing drag-in, Shelf Item drag-out, Shelf Batch drag-out, Pin, Remove, Manage Items, and Clear workflows at the running-application seam where they intersect the changed surfaces.
- Use disposable runtime measurement or inspection scaffolding if needed, then remove it before completion. Do not add a permanent screenshot, visual-regression, frontend component-test, or accessibility-test framework for this change.
- Treat the implementation as incomplete if source behavior and the living Product Specification disagree on the 642-pixel maximum, `+70` budget, or twelve-or-more scrolling threshold.

## Out of Scope

- Any icon redesign, icon-role remapping, new icon, changed glyph size, changed control target, or changed accessibility ownership.
- Any frontend dependency upgrade other than replacing `lucide-vue-next` with `@lucide/vue`.
- Vue, TypeScript, Vite, Tailwind CSS, VueUse, Tauri, Rust crate, or pnpm toolchain upgrades.
- A custom scrollbar component, changed scrollbar width, changed visible-thumb geometry, changed colors, permanently reserved scrollbar channel, or new scroll-position model.
- Spacing changes to Manage Items, Settings, confirmation surfaces, status surfaces, or the collapsed Rail Handle.
- Changes to Shelf Batch card internals, Shelf Item rows, header or footer controls, typography, radii, animation, motion, or general density.
- Changes to the shared virtualization threshold, entry-mounting algorithm, total-height formula, keyboard-navigation model, or component-specific virtual offsets.
- Changes to Shelf state, persistence, database schema, Temporary or Pinned lifecycle, duplicate handling, drag result semantics, source-file safety, or Shell Visual behavior.
- Changes to Edge Rail monitor ownership, docking preference, remembered placement, fullscreen policy, expansion/collapse delays, focus policy, or no-activate behavior.
- Increasing the Edge Rail minimum height, width, row height, or row gap beyond the explicitly approved geometry.
- Release version assignment, version metadata synchronization, NSIS packaging, installer hashes, signing, update behavior, or release publication.
- Rewriting the completed 0.0.12 specification, tickets, roadmap evidence, measurements, hashes, or acceptance record.
- New glossary terms, ADRs, compatibility shims, abstractions, configuration options, feature flags, or testing frameworks.
- macOS, Linux, x86, or ARM64 support.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- The official Lucide migration guidance identifies `@lucide/vue` as the supported replacement for the deprecated `lucide-vue-next` package. The package research linked above verifies Vue peer compatibility, named exports, props, tree-shaking metadata, and the nine icon components used by DropCove.
- The earlier 0.0.12 contract intentionally remains historical: it records the accepted 4-pixel boundary, `52 × batch count + 62` Rail budget, 640-pixel maximum, and exact release evidence that were true for that completed candidate.
- The follow-up geometry is internally constrained. With 48-pixel rows, 4-pixel gaps, and the approved 8-pixel top and bottom clearances, eleven batches require 642 pixels. Retaining a 640-pixel maximum would create a two-pixel overflow and violate the accepted five-through-eleven no-scroll tier.
- The living Product Specification currently records the earlier 640-pixel maximum and `+62` formula. Implementation must update that canonical contract atomically with the code rather than leaving mixed product definitions.
- The primary testing seam and all implementation decisions were confirmed by the user during the completed grill-with-docs workflow on 2026-10-15.
- Publishing this specification applies the `ready-for-agent` triage state only. Ticket decomposition, implementation approval, final acceptance, and Git commit authorization remain separate workflow decisions.

### Approved runtime amendment — 2026-10-15

Real Windows WebView2 measured a three-pixel width deficit at the former 180-pixel minimum even without a scrollbar; with the 8-pixel native scrollbar, the card-to-channel inset became -1 pixel instead of the required 10 pixels. The user approved increasing the Drop Shelf minimum, initial, and empty-reset width to 191 logical pixels while preserving the 180-pixel minimum height. The full minimum-width budget is 164-pixel card + 7-pixel left inset + 10-pixel right inset + 8-pixel scrollbar + 2-pixel border = 191 pixels.

This amendment includes native size validation, restored-width bounding, default/reset sizing, native resize constraints, initial window configuration, affected behavioral tests, and living Product Specification synchronization in ticket 02. Existing stored narrower widths are bounded when read; no database migration or compatibility layer is introduced. The automatic-growth column calculation must use the new horizontal spacing budget so its no-scroll thresholds follow the responsive grid. Historical 0.0.12 records remain unchanged.


### Approved gutter correction after visual acceptance — 2026-10-15

The user reported unequal left-border-to-card and card-to-scrollbar clearances in both actual surfaces, then explicitly chose equality measured to the scrollbar channel rather than the painted thumb. This supersedes the earlier asymmetric 7/10-pixel Drop Shelf and 6/10-pixel Edge Rail requirements: use 7/7 pixels for Drop Shelf and 6/6 pixels for expanded Edge Rail, with and without overflow. The existing 8-pixel scrollbar channel and 2-pixel transparent thumb border remain unchanged; neither should be subtracted from the left inset.

Keep the already-approved 191-pixel minimum width as a conservative floor (three pixels above the new 188-pixel minimum content budget with a scrollbar); do not change default/reset sizing again. Update the native automatic-growth budget to match the symmetric grid: two columns at 348 pixels and three at 516 pixels without overflow, with thresholds shifted by the actual scrollbar width when overflow is present. The 8-pixel vertical boundaries and native Rail height formula remain unchanged. Re-run the same real WebView2 equality assertion on ordinary and virtualized collections before calling this correction verified.


### Approved painted-thumb alignment — 2026-10-15

Further visual acceptance feedback showed that channel-only equality still left a 2-pixel visible imbalance: the centered thumb's transparent left border made the painted right gap 9 pixels versus 7 on Drop Shelf and 8 versus 6 on Edge Rail. The user approved a scrollbar-only correction: keep the 8-pixel channel and 4-pixel painted thumb, align that paint to the content-side channel edge, and retain rounded corners, existing colors/states, collection padding, virtualization, native dimensions, and column thresholds. This supersedes the earlier requirement to leave the thumb's horizontal border distribution unchanged. Manage Items and Settings keep their centered thumbs. No runtime overflow observer or new abstraction is needed.

