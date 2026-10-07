# 01: Align Edge Rail Corner Radius and Border Stroke

Status: ready-for-agent

## Description

Align the Edge Rail's corner radius and border outline with the Drop Shelf visual language.

## Required Behavior

1. **Corner Radius**:
   - The desktop-facing corners of the Edge Rail use an `8px` corner radius in both the resting Rail Handle (`16 × 96`) and Expanded states (`280 × H`).
   - The two corners touching the monitor edge remain square (`0px`).
   - When docked on the right monitor edge, `CornerRadius` is `(8, 0, 0, 8)`.
   - When docked on the left monitor edge, `CornerRadius` is `(0, 8, 8, 0)`.
   - Native GDI region clipping via `WindowInterop.SetEdgeRailCorners` applies a scaled `8px` radius (`WindowInterop.ScaleLogicalPixels(8, dpi)`).

2. **Border Stroke & Docking Orientation**:
   - The outer border uses `{ThemeResource CardStrokeColorDefaultBrush}`.
   - When docked on the right monitor edge, `BorderThickness` is `(1, 1, 0, 1)`.
   - When docked on the left monitor edge, `BorderThickness` is `(0, 1, 1, 1)`.
   - The border updates dynamically whenever rail placement changes.

3. **Invariants**:
   - Rail Handle `16 × 96` and expanded bounds `280 × 136..640` remain exact.
## Acceptance Criteria

- [x] `EdgeRailWindow.xaml` declares `CornerRadius="8,0,0,8"` and `BorderThickness="1,1,0,1"`.
- [x] `EdgeRailWindow.xaml.cs` dynamically updates `CornerRadius` and `BorderThickness` based on `placement.Edge`.
- [x] `ApplyVisibility` passes a scaled `8px` radius to `WindowInterop.SetEdgeRailCorners`.
- [x] Full automated test suite passes (`226/226`).

## Testing Seam

- Unit and regression test suite (`dotnet test`).
- Native visual inspection of window corner and border rendering.

## Comments
