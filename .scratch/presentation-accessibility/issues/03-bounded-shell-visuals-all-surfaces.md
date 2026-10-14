# 03: Bounded Shell Visuals Across All Surfaces

**What to build:** Extend the proven Shell Visual tracer to every place a Shelf Item is represented. Multi-item cards show at most three bounded previews, Manage Items rows show item visuals, and the expanded Edge Rail uses the same visual language. Every surface owns cancellation and release for its current generation, so hidden, collapsed, closed, removed, or replaced presentation cannot retain or display stale resources.

**Blocked by:** 02 — Single-Item Shell Visual Tracer.

**Status:** resolved

**Parent specification:** [DropCove 0.0.7 — Presentation and Accessibility Conformance](../spec.md)

## Acceptance criteria

- [ ] A multi-item Shelf Batch requests and displays Shell Visuals for at most its first three Shelf Items.
- [ ] Each stacked preview uses a `24 × 24` logical-pixel box; Manage Items rows use a `32 × 32` logical-pixel box; target-monitor DPI and aspect ratio are respected.
- [ ] Manage Items and expanded Edge Rail show immediate generic fallbacks and asynchronously replace only current matching results.
- [ ] Identical active requests within one surface generation are deduplicated without retaining an application-level cache after that generation ends.
- [ ] Hiding the Drop Shelf cancels queued Shelf work and releases its browser resources.
- [ ] Collapsing the Edge Rail cancels expanded-Rail work and releases previews not needed by the collapsed handle.
- [ ] Closing or replacing Manage Items, removing its owning Shelf Batch or Shelf Item, or disposing its window clears the full-item presentation projection and releases its visual resources.
- [ ] Delayed results from an older batch, surface generation, or removed item are discarded and release native/browser ownership without reaching the active UI.
- [ ] Visual fallback, cancellation, and release remain silent and do not change Core Shelf State, availability, ordering, pinning, drag, or dismissal behavior.

## Testing seam

Drive actual surface lifecycle through the Resident Instance while a deterministic extractor adapter holds and releases selected requests. Assert current visible outcomes and resource accounting at the Shell Visual interface. Use the real Windows adapter for representative image, file, folder, and unavailable references.

## Demo path

Open a mixed multi-item Shelf Batch, its Manage Items popup, and the expanded Edge Rail. Rapidly replace the managed batch, close/reopen the popup, collapse/re-expand the Rail, and hide/reopen the Shelf while visual requests are delayed. Only the current surface generation receives results.

## Non-goals

Large-collection virtualization, accessibility announcements, custom visual caching, non-image content thumbnails, or new Shelf state.

## Answer

Delivered in 0.0.7:
- Multi-item cards in `ShelfBatchCard.vue` display at most the first 3 items as 24px stacked previews.
- Manage Items rows render 32px Shell Visuals with surface-specific generation invalidation.
- Generational invalidation wired into `resident::show_shelf`, `resident::dismiss_shelf`, `manage_items::open`, `manage_items::close`, and `RailHandle.vue` collapse.
- Verified live: multi-item batch of 4 items mounts exactly 3 stacked visual components.
