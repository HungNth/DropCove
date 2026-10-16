# 02: Native Drop Shelf movement

**What to build:** Make the visible Drop Shelf movable through the non-action area of the control strip using native Windows movement. Movement must close Manage Items first, preserve every existing content interaction, remain bounded by the current display/DPI rules, and last only for the current visible session.

**Blocked by:** 01 — Dismiss Drop Shelf control strip.

**Status:** resolved

**Parent specification:** [Drop Shelf Window Controls](../spec.md)

**Testing seam:** Actual Windows Resident Instance, WebView2 Drop Shelf, and native window movement, supported by the existing native regression suite and directly executed cargo-xwin-produced Windows test binary.

**Demo path:** Open a populated Drop Shelf, open Manage Items, then drag from the control strip. Observe Manage Items close and move the Shelf within and across available displays. Exercise card Copy drag-out, scrolling, actions, confirmations, and every resize direction; dismiss and reopen to observe canonical placement rather than the last dragged coordinate.

- [x] Only the main Drop Shelf window receives `core:window:allow-start-dragging`; Settings, Manage Items, Edge Rail, and unrelated callers cannot start native movement.
- [x] A primary-pointer action on the control strip's non-action region starts platform-native window movement without document-level coordinate tracking, polling, custom snapping, or a second native movement loop.
- [x] The close action, Shelf Batches, item/batch drag regions, action controls, collection content, scrollbars, dialogs, status UI, footer, and all resize handles are excluded from the movement region.
- [x] Manage Items closes through its established lifecycle before movement starts. If closure fails, movement does not begin and the last coherent presentation remains available.
- [x] Movement is blocked while native resize is active, while Shelf mutation state is busy, while batch-drag or Clear Temporary Items confirmation is open, or while another modal/transient interaction owns the surface; those interactions are not silently canceled.
- [x] Empty, populated, scrollable, automatically grown, manually resized, and status-bearing Drop Shelves can be moved from the dedicated region without card overlap or loss of the `28`-pixel control-strip contract.
- [x] Item and whole-batch drag-out remain Windows Copy-only with unchanged preparation, acceptance, cancellation, Temporary consumption, Pinned retention, and source-file safety.
- [x] Pin/Bulk Pinning, Manage Items, Remove, Clear Temporary Items, status dismissal, collection scrolling, scrollbar dragging, and all eight resize directions remain independently operable and never move the window.
- [x] A manually selected position survives ordinary Shelf refresh, content mutation, automatic growth, and resize while the same visible session continues, except when work-area, display, or DPI constraints require event-driven reflow.
- [x] Movement within and across available monitors respects Per-Monitor V2 DPI and keeps the complete Drop Shelf within the applicable work area without adding continuous polling or custom magnetic-edge behavior.
- [x] Hotkey, tray/normal activation, Open Shelf, restart, and Shake-to-open continue to use their canonical cursor-monitor or Edge Rail monitor placement rules; no coordinate, setting, schema field, or new display state is persisted.
- [x] The decorative movement affordance is theme-aware and absent from the accessibility tree; the movement feature adds no animation and remains functional when Windows client-area animations are disabled.
- [x] Frontend production build, directly executed Windows native regression, and actual Resident/WebView2 movement smoke pass across every available monitor/theme configuration; unavailable DPI, display, or theme cases are recorded as unobserved.
- [x] Canonical product documentation records the delivered movement boundaries and session-only placement while ADR-0002 placement ownership remains unchanged.

## Answer — implementation and observed evidence, 2026-10-16

- Added `src-tauri/capabilities/main-movement.json` granting `core:window:allow-start-dragging` exclusively to `main`. Attempted IPC from `edge-rail` proved `plugin:window|start_dragging` is denied by ACL and `prepare_shelf_move` is rejected for non-main windows.
- Added backend `prepare_shelf_move` rejecting active resize/content drag sessions and closing Manage Items before native drag begins.
- Added frontend `startMove` in `src/composables/useShelf.ts` with primary pointer guard, modal/busy/resizing guards, and native `window.startDragging()`. Two-stage Escape parity closed Manage Items on stage 1 and dismissed Shelf on stage 2 without accidental movement.
- Real Windows pointer input dragged the Drop Shelf from the control strip within monitor 1 (`dx=100, dy=40`), across monitors (`x=2373` on `DISPLAY2`), and proved post-drag clamp inside the active work area. Click-only interaction on the drag region cleanly released control states.
- Dragging while the Clear Temporary Items modal was active was blocked without canceling the dialog; Escape canceled the modal and preserved all batches.
- An owned Windows Forms OLE Copy target proved single-item Copy consumed only the Temporary participant (`item-03.txt`), whole-batch Copy copied all fixture bytes while retaining Pinned references (`item-04.txt`, `item-05.txt`), and Escape cancellation preserved all references. All 70 fixture files remained identical to their source bytes.
- Content mutation/refresh preserved the manually selected position (`650, 150`), while dismiss + secondary launch repositioned the Shelf to the current cursor monitor (`2505, 26`), proving session-only placement.
- Unit test `shelf_resize::tests::work_area_clamping_keeps_rect_inside_boundaries` passed. All background review findings (moving reset, Escape parity, caller boundary, and capability scope) were addressed and verified.
