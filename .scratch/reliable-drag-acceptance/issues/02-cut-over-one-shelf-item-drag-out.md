# 02: Cut over one-Shelf-Item drag-out

**Parent specification:** [Reliable Native Drag Acceptance Specification](../spec.md)

**What to build:** Replace WinUI source drag for one Shelf Item with the probe-validated managed native OLE path on both the Drop Shelf and Edge Rail. Preserve file/folder interoperability, Copy-only source safety, Temporary/Pinned lifecycle rules, and equivalent Windows drag feedback.

**Blocked by:** 01: Prove a reliable native OLE acceptance signal.

**Status:** blocked — Ticket 01 failed its acceptance gate; native source results do not distinguish Copy from destination exit during DragEnter. See [probe verdict](../probe-verdict.md). No production cutover started.

**Testing seam:** Reuse the manager lifecycle and persistence seam for accepted versus non-successful outcomes. Test only decision-rich native result classification and resource lifetime below it. Use installed Release interaction for pointer threshold, drag image, cursor feedback, `Esc`, and real destination behavior.

**Demo path:** Drag one Temporary Item and one Pinned Item from both the Drop Shelf and Edge Rail to an accepting target; then repeat cancellation, rejection, disappearing target, native startup failure, and folder drag-out.

- [ ] A managed native adapter invokes OLE without a C++ project, unsafe code, or a new third-party package.
- [ ] Drag starts only after the pointer exceeds the Windows drag threshold; a normal click keeps its existing action.
- [ ] The drag shows equivalent item feedback and standard Copy/Not-allowed cursors; `Esc` cancels it.
- [ ] One file or folder is offered with Copy-only semantics and DropCove never mutates the source filesystem object.
- [ ] The probe-validated predicate is the only path to `AcceptedCopy`; every unclassified, canceled, rejected, disappearing-target, COM, or data-object result retains the valid Path Reference.
- [ ] Accepted Copy removes a Temporary Item and retains a Pinned Item on both the Drop Shelf and Edge Rail.
- [ ] Existing Missing and Unavailable preparation behavior remains unchanged.
- [ ] Individual popup drag keeps the popup open through cancellation, rejection, and failure; ordinary popup dismissal still works after completion.
- [ ] A native startup or execution error uses the existing warning surface and never falls back to WinUI source drag.
- [ ] The old one-item WinUI source-drag properties, handlers, result mapping, and dead code are removed from both surfaces.
- [ ] Existing manager and persistence tests remain green, focused native classification tests pass, and installed Release smoke proves accepted and non-successful item outcomes.
