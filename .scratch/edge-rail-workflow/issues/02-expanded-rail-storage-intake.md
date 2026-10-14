# 02: Expanded Rail and Storage Intake

Status: resolved
Type: task
Version: 0.0.5
Parent: [Specification](../spec.md)
Blocked by: 01
Implementation approval: approved by user on 2026-10-13

## What to build

Turn the collapsed Rail Handle into a bounded expanded rail that opens and closes on the specified hover/storage-drag delays, presents batch summaries with stable sizing and accepts Windows storage drops through the existing Shelf intake semantics without activating DropCove.

## Acceptance criteria

- [x] Hover/valid storage entry expands after 200 ms; early leave cancels.
- [x] Leave collapses after 300 ms; re-entry cancels collapse.
- [x] One monotonic coordinator/native one-shot timer; no polling or sleeping permanent tests.
- [x] Expanded width280, canonical bounded height, actual rows64 and gaps4 verified.
- [x] 1–3 batches280, 4–8 bounded tiers, 9+640 with scrolling.
- [x] Accepted content grows; removal does not live-shrink; next expansion reclaims height.
- [x] Bounds reuse selected edge/work-area/DPI placement.
- [x] Core order and accessible summary labels retained without a second lifecycle implementation.
- [x] Copy/path-backed intake with own-drag rejection and no focus theft.
- [x] Shared real-store intake semantics preserve filtering, first occurrence/order, duplicate and skipped-count contracts.
- [x] Duplicate/unsupported intake does not mutate retained content/order/sizing.
- [x] Actual hidden Shelf DOM and rail DOM refresh from the same committed storage drop.
- [x] Final item removal hides surfaces and cancels deadlines/native timer.

## Testing seam

Coordinator tests use deterministic window/monitor adapters and a manual clock. Core Shelf State's existing real-store intake tests remain authoritative; rail tests prove routing, timer cancellation and presentation changes around committed outcomes. Actual OLE DragEnter/DragOver/DragLeave/Drop effects, no-activate focus and timed visual behavior require native Windows smoke.

## Demo path

Hover briefly and leave without expansion, then hover through 200 ms and observe expansion. Re-enter during the 300 ms collapse delay and observe cancellation. Drop supported, mixed, duplicate-only and unsupported payloads; confirm shared Shelf results. Populate nine or more batches and observe bounded height plus scrolling without foreground activation.

## Non-goals

Pin/remove/clear controls, Manage Items, item/batch drag-out, fullscreen hiding and release packaging are out of scope.

## Comments

## Answer

Ticket02 resolved. Native WM_TIMER dispatch uses a monotonic coordinator deadline; native hover/OLE input share one timer. Existing Shelf mutations remain authoritative. Version metadata stays0.0.4; no installer or commit created at this slice.

### Exercised verification

- TDD red/green: coordinator timer scenario initially failed for absent interfaces, then exposed final-item deadline/default-reset defects; focused freshly compiled test now passes. Real DragSession guard policy test went red before shared Enter/Over/Drop eligibility implementation, then passed (own Copy→None, external Copy→Copy, unsupported/Move→None). Existing first-occurrence/order and empty/mixed real-store checks also passed individually. Full integrated suite is reserved for the end of the requested multi-ticket delivery, not claimed here.
- vue-tsc/Vite and cargo-xwin embedded Windows release builds passed. Final layout binary: 13,087,744 bytes, SHA-256 `4821b354e3fba239eebfae221704265cad69b7f6ea8fb26a5762b286b9147d7e`.
- Native pointer80ms/leave prevented expansion;260ms hover expanded; re-entry during300ms delay preserved expansion;400ms leave collapsed. Owned foreground probe kept focus. Real OS-marshaled STA DoDragDrop used FileDrop: Copy accepted, Text returnedNone, Escape cancellation returnedNone. No foreign COM pointer dereference was used.
- Native rows/window tiers observed280,340,408,476,544,612,640 heights. Removal10→2 retained640 live, collapse/re-expand reclaimed280. Native storage drop refreshed exact hidden Shelf DOM and Rail DOM2→3 with the same newest item. Duplicate retained IDs/order and unsupported Text left counts unchanged. Final clear hid both surfaces.
- Actual screenshot exposed flex shrink despite CSS height64; fixed row flex basis/shrink and measured all10 rendered rows64 and all9 gaps4. Final viewport280×640, scrollHeight684 > clientHeight602; final geometry screenshot observed. Earlier runtime proof hashes055f/a7ca remain scoped to their versions, with final layout rechecked on4821.
- Owned native Exit returned0; profile/WebView removed; absent DropCove Run, five unrelated entries, ambient executable/shortcut hashes remained unchanged. Reusable test binary/scripts are deliberately retained outside the repository for tickets03–05 and will be removed after release proof.

### Coverage limits

- Own-drag rejection and mixed payload filtering have focused real-session/Core-store evidence; native surface smoke proves external FileDrop/Text and Copy/cancel, not a mixed virtual Shell payload or rail-owned outbound drag (ticket03 supplies outbound UI).
- Native measurements use available96-DPI hardware; mixed-DPI remains deterministic geometry coverage and an explicitly unobserved hardware limit.
