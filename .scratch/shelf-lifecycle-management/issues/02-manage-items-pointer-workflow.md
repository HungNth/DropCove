# 02: Manage Items pointer workflow and individual actions

Status: resolved
Type: task
Version: 0.0.3
Parent: [Specification](../spec.md)
Blocked by: 01
Implementation approval: approved by the user on 2026-10-13

## What to build

A user can open Manage Items for a multi-item batch and inspect, Pin/Unpin, remove, or drag one reference without transferring the entire batch. Deliver a real native management surface with its core mutations and native drag path together, not a backend-only or UI-only scaffold.

## Acceptance criteria

- [x] Only multi-item cards expose Manage Items; a single-item card remains directly manageable.
- [x] The native popup is 280 logical pixels wide, bounded to 480 logical pixels high, with a vertical scroller when necessary; it can extend beyond a 180-pixel Shelf.
- [x] Placement prefers right, left, below, then above the originating card, with work-area clamping and a preferred 16-pixel margin.
- [x] Pointer opening does not force keyboard focus into the popup.
- [x] Each row provides fallback file/folder presentation, name, shortened path, full-path tooltip/accessibility text, availability, Pin/Unpin, Remove Item, and native item drag-out.
- [x] Remove Item deletes only its reference, prunes an empty batch, and preserves other batches even when paths overlap.
- [x] Pinned item drag retains the reference; successful Temporary item drag consumes only that item. Rejection/cancellation/failure retains it.
- [x] Individual item drag keeps the popup open while its batch remains, even if the batch shrinks to one item.
- [x] Explicit Close and batch/owner disappearance release the management projection and close the surface; no detached popup remains.
- [x] Shelf and popup refresh from the same successful core mutation, including durable rollback on failure.
- [x] Only one pointer management view is open; opening another batch replaces it rather than creating independent durable state.

## Testing seam

Permanent checks cover the shared Core Shelf State item mutations, same-path occurrences in different batches, pruning, pin retention, and failure rollback. Native Windows smoke proves actual popup bounds/placement, no-forced-focus pointer opening, item actions, and OLE completion.

## Demo path

Drop a three-item batch into a minimum-width Shelf. Open Manage Items, inspect full paths and availability, Pin one row, remove another, and Copy the remaining Temporary row. Observe the retained Pinned row and the still-open management view; Close explicitly, then remove its batch and verify source safety.

## Non-goals

The Bulk Pinning header/card controls and exact keyboard/Escape traversal are completed in ticket 03. Do not expose a no-op bulk control in this intermediate pointer slice. Resize-start closure is integrated with native sizing in ticket 05.

## Comments

- Implemented one owned native `manage-items` surface with validated card anchors, logical/physical DPI conversion, complete-bounds placement priority and native outer-frame clamping. Popup projection is transient; durable ownership remains in ShelfState. The popup-specific composable never applies Shelf sizing to the management window.
- Added serialized Remove Item and management availability lookup. Explicit removal works for Pinned Items, rejects unknown IDs, preserves overlapping references in other batches, prunes empty batches, resets sizing only on final removal, and changes caches only after commit. Management lookup refreshes availability without Missing cleanup.
- Both surfaces receive invalidation notifications and read current core state; superseded responses cannot overwrite newer projections. Main inbound drop registration is not revoked when the popup closes. Management WebView external drop is disabled because it is not a storage intake surface.
- Final `pnpm build` passed; cargo-xwin compiled Windows-targeted tests with `--no-run`; direct native test executable reported **20 passed, 0 failed**. Final non-instrumented Windows x64 release build with `tauri/custom-protocol` passed. Existing external CRT PDB linker warnings remain non-fatal.
- Actual Windows app smoke proved: independent 280-pixel popup beside a 180-pixel Shelf; bounded 480-pixel scroller with 16 references; file/folder fallback, full-path tooltip/accessibility text, real Missing and Unavailable labels; pointer no-activation; per-row Pin/Unpin and Remove; synchronized card item counts; occurrence-specific removal retaining an identical path in another batch; matching copied/source bytes.
- Actual row OLE Copy retained a Pinned Item and consumed only the Temporary participant. Native Escape cancellation and rejected drop onto the Shelf retained Temporary references; invalid source-path drag reported its real error and retained the reference. The popup stayed open when its batch shrank to one Pinned Item. SQLite delete-failure injection displayed the error without changing either surface/core state; permanent checks cover rollback across reopen.
- Lifecycle smoke proved explicit Close releases the active projection, opening another batch reuses the same single native window with new rows, batch removal closes its popup without affecting unrelated batches, and native owner hide closes/releases the management surface. Final edge-placement smoke kept the complete native outer frame inside the work area with a 16-pixel boundary margin.
- Found and fixed hidden Tao window frame initialization: `SWP_FRAMECHANGED` is required before no-activation show. Native client/caption-offset checks and final screenshot proved no Windows title bar and the requested client size. An earlier row-drag attempt stalled; both owned processes were recovered and the owned Copy target restarted. Subsequent trace recorded `DoDragDrop -> Copy`; the final non-instrumented release also completed both Pinned and Temporary row Copies. The earlier stalled attempt is not counted as passing proof; its exact cause was not established.
- Removed all tagged drag instrumentation and the temporary Copy-target script. Closed managed CDP tabs; terminated/waited only known owned app/target processes; released input and SQLite/log handles; removed owned fixture/profile/WebView/copy/trace data. Final owned-directory existence check was false. Ambient user Shelf profiles/recovery data were not used.
- No Bulk Pinning or exact keyboard traversal/focus restoration was shipped here (ticket 03); resize-start closure remains ticket 05. Version metadata remains 0.0.2 until release ticket 06. No commit created.
