# Stacked Shelf Batch Preview Refinement

Status: ready-for-agent
Design confirmation: approved by user on 2026-10-21
Testing seam: user-approved Windows Resident/WebView2 presentation surfaces
Ticket decomposition: pending
Implementation approval: separate gate after ticket decomposition
Release assignment: unassigned; no new roadmap version approved
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md#92-multi-item-batch)
Canonical vocabulary: [DropCove Domain Glossary](../../GLOSSARY.md)

## Problem Statement

When a Shelf Batch contains multiple Shelf Items, its first previews currently extend horizontally to the right, with each later preview covering part of the previous preview. This consumes horizontal space and does not present the batch as the compact layered stack the user expects.

The user wants a compact layered stack: the first Shelf Item remains clearly in front at the lower-left, while the next two previews sit behind it toward the upper-right. Opaque square backing must not fill transparent artwork or aspect-fit margins; later layers may remain visible through those regions. The refinement must not change what the Shelf Batch contains or how the card is operated.

## Solution

Render a compact diagonal stack for every multi-item Shelf Batch on both the Drop Shelf and expanded Edge Rail. Continue to use at most the first three Shelf Items in source order and their real Shell Visuals.

Each preview uses a `24 × 24` logical-pixel visual box. The first Shelf Item is the front layer at the lower-left. Each following preview is placed three logical pixels farther right and three logical pixels farther up, behind the previous layer. A three-preview stack therefore remains bounded to approximately `30 × 30` logical pixels.

The stack remains decorative and non-interactive. The item-count title, whole-batch drag region, Bulk Pinning, Manage Items, Remove Batch, accessibility behavior, and card dimensions remain unchanged. Batches with more than three Shelf Items still render only three previews and communicate the total through the existing item-count title.

## User Stories

1. As a user viewing a multi-item Shelf Batch, I want its previews arranged as a compact layered stack, so that the card communicates that the items belong together.
2. As a user, I want the first Shelf Item shown as the front preview, so that the displayed order matches the source order of the batch.
3. As a user, I want later previews placed behind and toward the upper-right while preserving natural transparency, so that the stack remains compact without dark square backing around nonsquare artwork.
4. As a user viewing a two-item Shelf Batch, I want exactly two visible layers, so that the preview accurately represents the available bounded sample.
5. As a user viewing a Shelf Batch with three or more Shelf Items, I want three visible layers, so that the preview remains informative without becoming crowded.
6. As a user viewing a large Shelf Batch, I want the preview count capped at three, so that card width and rendering work remain bounded.
7. As a user viewing a large Shelf Batch, I want the existing `N items` title to continue showing the total count, so that hidden previews do not hide the batch size.
8. As a user moving between the Drop Shelf and expanded Edge Rail, I want the same stacked-preview geometry on both surfaces, so that Shelf Batches remain visually consistent.
9. As a user viewing files, folders, and image thumbnails together, I want each layer to retain its real Shell Visual, so that the stack remains recognizable.
10. As a user whose Shell Visual is unavailable or still loading, I want the existing fallback visual to occupy the same layer, so that the stack does not jump or collapse.
11. As a user waiting for asynchronous Shell Visual loading, I want replacement visuals to retain their original layer and order, so that the stack does not reorder itself.
12. As a pointer user, I want dragging the card to continue dragging the entire multi-item Shelf Batch, so that the visual change does not alter Copy behavior.
13. As a pointer user, I want the preview itself to remain part of the existing batch drag hit area rather than becoming a separate control, so that the card remains predictable.
14. As a user, I want Bulk Pinning, Manage Items, and Remove Batch controls to retain their existing hit areas, so that the stacked preview does not cover or intercept actions.
15. As a keyboard user, I want the existing control order and activation behavior preserved, so that the decorative stack does not add an unexpected focus target.
16. As a screen-reader user, I want the existing batch name, item count, and drag instructions preserved without individual decorative preview announcements, so that the refinement adds no accessibility noise.
17. As a user in Windows Dark or Light presentation, I want the stack to use the existing surface and Shell Visual resources, so that it remains visually integrated with DropCove.
18. As a user in a Windows Contrast Theme, I want the batch title, controls, focus indicators, and card boundary to remain distinguishable, so that decorative layering does not weaken operability.
19. As a user at the minimum Drop Shelf width or in the narrow expanded Edge Rail, I want the preview stack to fit within the existing card height and leave the title and controls usable, so that the refinement does not introduce clipping or horizontal scrolling.
20. As a user, I want Shelf state, ordering, pinning, persistence, availability, drag completion, and source-file safety to remain unchanged, so that this remains a presentation-only refinement.
21. As a user, I do not want an extra `+N` badge or additional preview layers, so that the existing item-count title remains the single count indicator.
22. As a user, I do not want preview hover expansion, clicking, or item-level drag behavior added to the card, so that Manage Items remains the established item-level workflow.

## Implementation Decisions

- Refine the shared multi-item Shelf Batch presentation rather than creating separate Drop Shelf and Edge Rail variants.
- Preserve the existing distinction between single-item and multi-item cards. Single-item Shell Visual size and direct item drag behavior do not change.
- Continue to select at most the first three Shelf Items in source order.
- Use `24 × 24` logical-pixel visual boxes for every multi-item preview layer.
- Place the first Shelf Item at the lower-left as the front layer. Place the second layer three logical pixels right and three logical pixels up; place the third layer another three logical pixels right and three logical pixels up.
- Bound a three-layer stack to approximately `30 × 30` logical pixels and keep it inside the existing 48-logical-pixel card geometry.
- Define layer ownership explicitly so that the first Shelf Item remains in front regardless of DOM sibling paint order. Later source items must not accidentally paint over the front item.
- Preserve each Shell Visual's natural transparency without opaque per-layer backing. Later layers or the card surface may show through transparent artwork and aspect-fit margins; the first Shelf Item still paints in front wherever its artwork is opaque.
- Use existing Shell Visual content, fallback behavior, sizing, aspect-ratio handling, and asynchronous replacement. Do not generate illustrative colors or synthetic thumbnails.
- Add no decorative border, shadow, count badge, hover expansion, animation, or item-level interaction to the preview stack.
- Keep the preview region non-interactive. The established batch drag region continues to own pointer drag initiation, while Bulk Pinning, Manage Items, and Remove Batch remain separate controls.
- Preserve whole-batch drag semantics, the existing pointer movement threshold, busy-state blocking, tooltips, accessible drag instructions, and action behavior.
- Preserve the existing item-count title as the sole total-count presentation for multi-item Shelf Batches.
- Preserve current card dimensions, collection sizing, virtualization measurements, ordering, and scrolling behavior on both presentation surfaces.
- Add no Core Shelf State, persistence, schema, IPC, settings, native-window, or Windows Shell extraction changes.
- Clarify the canonical multi-item Shelf Batch presentation contract with the approved front-layer order, diagonal offset, and bounded geometry when implementation is delivered.
- Add no compatibility path, dependency, ADR, or glossary term. Existing Shelf Batch, Shelf Item, and Shell Visual vocabulary remains canonical.
- Do not add a roadmap version for this presentation-only refinement. Release assignment remains separate from this specification.

## Testing Decisions

- The primary and only new behavioral seam is the running Windows Resident Instance with its actual WebView2 presentation surfaces. Verify the same Shelf Batch on the Drop Shelf and expanded Edge Rail rather than testing private layout helpers.
- Exercise two-item, three-item, and more-than-three-item Shelf Batches. Observe two layers, three layers, and a three-layer cap respectively, while the existing item-count title reports the full total.
- Use visually distinct files, folders, image thumbnails, and fallback visuals to verify source order, first-item front ownership, three-pixel upper-right offsets, natural transparency through artwork and aspect-fit margins, and stable layering after asynchronous visual replacement.
- Measure the rendered preview region and card. A three-layer stack remains approximately `30 × 30` logical pixels inside the existing 48-logical-pixel card; the title and trailing actions remain visible and usable at minimum supported widths.
- Exercise pointer drag from the preview area and adjacent card area. Both continue to initiate the established whole-batch drag after the existing movement threshold; clicking or hovering a preview introduces no separate behavior.
- Exercise Bulk Pinning, Manage Items, and Remove Batch with pointer and keyboard after the layout change. Their hit areas, disabled states, focus behavior, Enter/Space activation, tooltips, and accessible names/states remain unchanged.
- Inspect accessibility through the actual surface: the preview layers remain decorative, the card retains its batch drag instruction and item-count title, and no additional focus targets or announcements appear.
- Inspect Dark, Light, and actual Windows Contrast presentation where available. Layers retain their natural transparency without opaque per-layer backing, a new border, or a shadow treatment; unavailable environment checks must be reported as unobserved.
- Recheck narrow Drop Shelf and expanded Edge Rail layouts, ordinary and virtualized collections, and batches whose Shell Visuals load after initial fallback. No clipping, action overlap, horizontal scrolling, row-height change, or virtual positioning regression is accepted.
- Run the existing frontend typecheck/production build after implementation. Add no frontend framework, source-text assertion, or test that pins private class names, DOM order, or a specific CSS technique.
- Good verification observes user-visible geometry and preserved behavior. It does not assert whether explicit layering is implemented through `z-index`, reversed rendering, grid placement, or another equivalent internal technique.
- Prior art is the established actual-surface verification used for bounded first-three Shell Visuals and compact Shelf Batch controls. Reuse that seam rather than introducing a second presentation harness.

## Out of Scope

- Single-item Shelf Batch redesign or size changes.
- Rendering more than the first three Shelf Items.
- Adding a `+N` badge, per-layer labels, filenames, or another count indicator.
- Preview hover expansion, fan-out, animation, clicking, selection, item-level drag-out, or direct item actions.
- Changes to Bulk Pinning, Manage Items, Remove Batch, card drag thresholds, Copy completion, Temporary Item consumption, or Pinned Item retention.
- Changes to Shelf Item source ordering, Shelf Batch ordering, duplicate suppression, availability classification, persistence, or recovery.
- Changes to Drop Shelf or Edge Rail window dimensions, collection gaps, card height, virtualization thresholds, or scrollbar behavior.
- New Shell thumbnail extraction, caching, fallback, resource-lifetime, or IPC behavior.
- New settings, dependencies, design-system primitives, ADRs, glossary entries, compatibility layers, or platform support.
- A new roadmap version, installer release, packaging work, version synchronization, or Git commit.

## Further Notes

- The canonical product behavior already requires a stacked preview using at most the first three Shelf Items. This specification narrows the previously unspecified stack geometry and corrects the current horizontal-overlap presentation.
- The current presentation allows later preview siblings to cover earlier previews. Delivery must explicitly preserve the approved first-item-front ordering rather than relying on incidental DOM paint order.
- The visual example supplied during design discovery described geometry only. Production uses real Shell Visuals and semantic surface resources, not the example's solid colors.
- The user explicitly approved applying the refinement to both the Drop Shelf and expanded Edge Rail, retaining the three-preview cap, using 24-pixel visuals with three-pixel offsets, and leaving the roadmap unchanged.
- No glossary update is required because Shelf Batch, Shelf Item, and Shell Visual already have precise canonical definitions.
- No ADR is required because this is a reversible presentation refinement with no new architectural ownership or material trade-off.
- Ticket decomposition and implementation approval remain separate workflow gates. This `ready-for-agent` specification does not authorize implementation, release packaging, a version change, or a Git commit.
