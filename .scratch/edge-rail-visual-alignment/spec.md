# Edge Rail Visual Alignment

Status: ready-for-agent

## Problem Statement

The Edge Rail currently uses a `16px` corner radius on its desktop-facing corners (`CornerRadius="16,0,0,16"` or `"0,16,16,0"` and native GDI clipping with `Radius = ScaleLogicalPixels(16)`). This radius is noticeably larger than the Drop Shelf shell, which uses the standard Windows 11 `8px` corner radius (`OverlayCornerRadius`).

In addition, while the Edge Rail root has a `BorderBrush` resource set, its `BorderThickness` is not explicitly structured across edges:
- It lacks an edge-docked stroke definition: the physical screen-edge side should have a `0px` border while the 3 free sides facing the desktop have a subtle `1px` border matching the Drop Shelf's `CardStrokeColorDefaultBrush`.

This causes a visual mismatch between the Drop Shelf and Edge Rail. The Edge Rail needs to align with the Drop Shelf's visual boundary: an `8px` desktop-facing corner radius and a consistent `1px` theme-aware stroke on its free edges.

## Solution

1. **Corner Radius Alignment**:
   - Reduce the desktop-facing corner radius of the Edge Rail from `16px` to `8px` in both resting (Rail Handle `16 × 96`) and expanded (`280 × H`) states.
   - Retain selective corner rounding: the two desktop-facing corners are rounded to `8px`, while the two corners abutting the monitor edge remain square (`0px`).
   - Update native GDI clipping region (`SetEdgeRailCorners`) to scale `8` logical pixels by the window DPI.

2. **Border Stroke & Thickness Alignment**:
   - Apply a `1px` border on the 3 free outer edges and `0px` on the monitor-docked edge:
     - Right rail (docked right): `BorderThickness="1,1,0,1"` (Left: 1, Top: 1, Right: 0, Bottom: 1).
     - Left rail (docked left): `BorderThickness="0,1,1,1"` (Left: 0, Top: 1, Right: 1, Bottom: 1).
   - Dynamically update `BorderThickness` and `CornerRadius` in `ApplyPlacement` / `ApplyVisibility` when the rail is positioned on the Left or Right edge.
   - Use `{ThemeResource CardStrokeColorDefaultBrush}` for `BorderBrush`, guaranteeing theme adaptation across Light, Dark, and High Contrast.

3. **Behavioral Invariants**:
   - Preserve all existing geometry contracts: Rail Handle `16 × 96`, expanded width `280px`, adaptive height tiers `136..640px`, and vertical work-area centering.
   - Preserve hover timing (`200ms` expand, `300ms` collapse), control placement (`Open Shelf`, `Clear Temporary Items`), multi-item flyout lifecycle, and drag-and-drop operations.
   - Do not add animations, timers, or background polling.

## Implementation Decisions

- Corner radius is `8` logical pixels, matching Drop Shelf `OverlayCornerRadius`.
- Native GDI region clipping in `WindowInterop.SetEdgeRailCorners` receives `WindowInterop.ScaleLogicalPixels(8, dpi)`.
- `BorderThickness` is set dynamically in `EdgeRailWindow.xaml.cs` to match docking orientation (`1,1,0,1` for right edge, `0,1,1,1` for left edge).
- XAML initial definition in `EdgeRailWindow.xaml` uses `CornerRadius="8,0,0,8"` and `BorderThickness="1,1,0,1"`.
- All repository changes remain uncommitted until explicit user authorization.
