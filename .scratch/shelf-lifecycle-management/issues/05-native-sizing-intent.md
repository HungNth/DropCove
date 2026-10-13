# 05: Native resizing and retained Manual Height Override

Status: ready-for-agent
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 02
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can resize the borderless Shelf naturally and have the core retain exactly the sizing intent they chose. Complete the existing automatic growth/width baseline without treating programmatic growth or monitor clamping as manual height input.

## Acceptance criteria

- [ ] All four edges and four corners resize natively with an approximately 8-logical-pixel hit region, appropriate cursor, and accent hover outline.
- [ ] Actual vertical/corner resizing that changes height creates Manual Height Override; width-only resizing preserves automatic-height eligibility when no override exists.
- [ ] Programmatic growth, restore/reflow, and work-area/DPI clamping do not create or overwrite manual intent.
- [ ] Preferred width and nullable manual logical height persist while content exists and restore after process restart.
- [ ] Width-only changes do not clear an existing Manual Height Override.
- [ ] Empty-shelf intent remains transient and is committed with the first accepted batch; unsupported/duplicate input does not persist or alter it.
- [ ] Automatic height follows the existing responsive-row tiers, can grow on accepted content without live shrinking, and recalculates on reopening/restart when no override exists.
- [ ] Actual bounds remain inside the selected monitor work area without rewriting preferred intent solely because of temporary clamping.
- [ ] Manage Items closes at the start of user resizing, before resize changes detach its origin; programmatic growth is not mistaken for user resize start.
- [ ] Removal of the final Shelf Item clears width/manual-height intent and resets the next opening to 180 × 180.
- [ ] Failed sizing persistence preserves the previous retained intent and reports the failure.

## Testing seam

Core checks cover width-only versus height-changing intent, pending empty intent, persisted manual override, non-persisted automatic growth, final-item reset, and rollback. Native smoke proves real edges/corners, cursor/outline, DPI conversion/clamping, programmatic-versus-user classification, and management closure at resize start.

## Demo path

Resize width only and add enough batches to change row count; observe automatic eligibility. Resize from bottom and a corner, restart, and observe manual height retention. Reopen with fewer references, remove the final item, then observe default next-opening dimensions. Repeat with Manage Items open and verify immediate resize-start closure.

## Non-goals

No generic sizing framework, new settings UI, polling loop, or fourth display state. Reuse the existing sizing columns and automatic formulas.

## Comments
