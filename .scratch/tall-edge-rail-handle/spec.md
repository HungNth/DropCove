# Tall Edge Rail Handle & Minimum Expanded Height

Status: ready-for-agent

## Problem Statement

DropCove's resting Rail Handle currently measures `16 × 96` logical pixels. On modern high-resolution displays (1080p, 1440p, 4K), a 96-pixel vertical target requires precise cursor positioning to trigger hover expansion (Fitts's Law penalty). Reference application Edge-Drop dedicates 25%–60% of the screen height (typically 260px–416px on 1080p) to its edge hot zone, making edge-based interaction effortless.

Additionally, with a taller resting handle (e.g. 280px), expanding horizontally when holding 1 to 3 Shelf Batches (which previously targeted 136px, 204px, or 272px) must not vertically collapse or shrink the window.

## Required Behavior

1. **Resting Rail Handle Height**:
   - `EdgeRailSizingPolicy.HandleHeight` increases from `96` to `280` logical pixels.
   - The resting Rail Handle bounds are `16 × 280` logical pixels, vertically centered on the active monitor work area.
   - Desktop-facing corners retain their selective 8px radius and 1px `CardStrokeColorDefaultBrush` outline; screen-edge corners remain square with 0px outline.

2. **Adaptive Expanded Height Floor**:
   - The expanded rail target height is floored at `HandleHeight`:
     `TargetExpandedHeight(n) = Math.Max(HandleHeight, 68 * n + 68)` for $n \ge 1$.
   - Explicit tiers:
     - 1 to 3 batches: `280px`
     - 4 batches: `340px`
     - 5 batches: `408px`
     - 6 batches: `476px`
     - 7 batches: `544px`
     - 8 batches: `612px`
     - 9+ batches: `640px` (MaxHeight, scrollable)
     - 0 batches: `0px` (hidden)
   - Expanding on hover or drag entry never vertically shrinks the window.

3. **Invariants & Preserved Behavior**:
   - Grow-only while expanded: `HeightAfterMutation` continues to preserve the taller height when batches are removed while expanded.
   - Removing all items hides the rail immediately.
   - Selective GDI window clipping dynamically clips to the physical dimensions of the 280px handle or expanded window.
   - Timing (200ms expand, 300ms collapse) and button placements (`Open Shelf` top, `Clear Temporary Items` bottom-left) remain exact.

## Acceptance Criteria

- [ ] `EdgeRailSizingPolicy.HandleHeight` is `280`.
- [ ] `EdgeRailSizingPolicy.TargetExpandedHeight` returns `280` for 1, 2, and 3 batches.
- [ ] `EdgeRailSizingPolicyTests` updated and passing for new tiers and invariants.
- [ ] Full regression test suite passes (`226/226`).
- [ ] Documentation (`docs/DropCove.md`) updated to reflect `16 × 280` Handle and minimum expanded floor.
- [ ] All changes remain uncommitted until explicit user authorization.
