# 01: Increase Edge Rail Handle Height and Apply Minimum Expanded Floor

Status: ready-for-agent

## Description

Update `EdgeRailSizingPolicy` to set `HandleHeight` to `280` logical pixels and floor `TargetExpandedHeight` at `HandleHeight`, ensuring the Edge Rail never vertically shrinks upon expansion.

## Required Behavior

1. **Policy Constants**:
   - `EdgeRailSizingPolicy.HandleHeight` = `280`.

2. **Expanded Height Floor**:
   - `TargetExpandedHeight(int batchCount)` returns `Math.Max(HandleHeight, 68 * batchCount + 68)` for positive batch counts.
   - For 1, 2, and 3 batches, target height is `280`.
   - For 4 through 8 batches, target height is `340`, `408`, `476`, `544`, `612`.
   - For 9 or more batches, target height is capped at `640`.
   - For non-positive batch count, target height is `0`.

3. **Window Construction & Positioning**:
   - `EdgeRailWindow` initial window sizing and `PositionEdgeRail` calls respect the new `280` collapsed height.

4. **Unit Tests**:
   - Update `EdgeRailSizingPolicyTests` to cover the new tiers, floor behavior, and mutation preservation.
   - All tests pass.

## Acceptance Criteria

- [x] `EdgeRailSizingPolicy.HandleHeight` is `280`.
- [x] `TargetExpandedHeight` returns 280 for 1, 2, and 3 batches.
- [x] `EdgeRailSizingPolicyTests` covers 280px floor and passes.
- [x] Full regression test suite passes.

## Testing Seam

- `dotnet test tests/DropCove.Tests/DropCove.Tests.csproj`
