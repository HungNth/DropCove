---
status: proposed
---

# Gate native drag-out on acceptance evidence

DropCove will not use WinUI `DropCompleted=Copy` as sufficient evidence of a Successful Drag-Out because a destination that stops during `DragEnter` can still produce that result. A real throwaway OLE probe must first show a reliable distinction between destination acceptance, rejection, cancellation, and a disappearing destination. Only then will DropCove replace the WinUI source-drag path with one managed P/Invoke implementation for item and whole-Shelf-Batch drag-out from both the Drop Shelf and Edge Rail. If the probe remains ambiguous, release stays blocked; DropCove will not use a heuristic or weaken the Path Reference lifecycle contract.

## Probe verdict — 2026-10-05

The [Ticket 01 probe](../../.scratch/reliable-drag-acceptance/probe-verdict.md) did not establish a reliable acceptance predicate. Accepted Copy and destination exit during `DragEnter` both returned `DRAGDROP_S_DROP` with `DROPEFFECT_COPY` in five of five correlated attempts; the exiting destination never received `Drop`. The native cutover remains unapproved by evidence. This ADR stays proposed, production drag-out is unchanged, and release remains blocked pending a new design decision.
