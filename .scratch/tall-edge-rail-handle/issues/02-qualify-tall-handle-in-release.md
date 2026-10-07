# 02: Qualify Tall Edge Rail Handle in Installed Release

Status: ready-for-agent
Blocked by: 01: Increase Edge Rail Handle Height and Apply Minimum Expanded Floor

## Description

Verify the modified Edge Rail handle height and expanded tiers in a clean Release build, update documentation, package the installer, and verify live execution.

## Required Behavior

1. Clean Release build succeeds with 0 errors and 0 warnings.
2. Full test suite passes in Release mode.
3. Documentation (`docs/DropCove.md`) updated with `16 × 280` handle dimensions and `280px` minimum expanded floor.
4. Repackage NSIS installer and silently install to local system.
5. Verify updated binary is running and responsive.

## Acceptance Criteria

- [x] Clean Release build succeeds with 0 errors and 0 warnings.
- [x] 226 tests pass in Release mode.
- [x] `docs/DropCove.md` reflects `16 × 280` handle and `280px` minimum expanded height.
- [x] Installer packaged, installed, and verified on machine.
- [x] No git commit created without explicit user authorization.

## Testing Seam

- `dotnet build -c Release`
- `dotnet test -c Release`
