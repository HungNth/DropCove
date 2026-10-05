# 03: Cut over whole-Shelf-Batch drag-out

**Parent specification:** [Reliable Native Drag Acceptance Specification](../spec.md)

**What to build:** Extend the validated native OLE path to whole-Shelf-Batch drag-out from both the Drop Shelf and Edge Rail. Preserve multi-path payloads, mixed Temporary/Pinned lifecycle, Missing cleanup, Unavailable partial-drag rules, and popup/flyout behavior.

**Blocked by:** 02: Cut over one-Shelf-Item drag-out.

**Status:** retired — Effort closed. DropCove retains WinUI whole-batch drag-out.

**Testing seam:** Reuse the existing prepared-batch identity set and manager completion seam. Test multi-path payload/result decisions without duplicating item-level lifecycle tests. Use installed Release interaction for actual multi-file/folder payload delivery and transient UI behavior.

**Demo path:** Drag a ten-item Temporary Shelf Batch, a mixed Temporary/Pinned Shelf Batch, a batch with Missing Items, and a partially Unavailable batch from both surfaces. Exercise accepted Copy, Reject, Cancel, disappearing target, and native failure.

- [ ] Whole-batch drag uses the same native engine and probe-validated acceptance predicate as one-item drag.
- [ ] The payload contains every prepared available path exactly once and remains Copy-only.
- [ ] Accepted Copy consumes only participating Temporary Items, retains participating Pinned Items, and removes an empty Shelf Batch.
- [ ] Cancellation, rejection, disappearing destination, and native failure retain every valid participating Path Reference.
- [ ] Confirmed-Missing references are removed before native drag and are not restored after a non-successful operation.
- [ ] Unavailable references and partial-batch confirmation retain the existing behavior.
- [ ] Starting whole-batch drag closes its popup or flyout; completion does not leave orphaned transient UI.
- [ ] Equivalent batch drag feedback communicates that multiple paths participate.
- [ ] Drop Shelf and Edge Rail use the same batch adapter and lifecycle semantics.
- [ ] The old whole-batch WinUI source-drag properties, handlers, result mapping, and dead code are removed from both surfaces.
- [ ] Manager and persistence tests prove prepared identity consumption and retention; installed Release smoke proves the target receives all paths for accepted file and folder batches.
