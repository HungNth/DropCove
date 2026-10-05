---
status: rejected
---

# Gate native drag-out on acceptance evidence

DropCove will not use WinUI `DropCompleted=Copy` as sufficient evidence of a Successful Drag-Out because a destination that stops during `DragEnter` can still produce that result. A real throwaway OLE probe must first show a reliable distinction between destination acceptance, rejection, cancellation, and a disappearing destination. Only then will DropCove replace the WinUI source-drag path with one managed P/Invoke implementation for item and whole-Shelf-Batch drag-out from both the Drop Shelf and Edge Rail. If the probe remains ambiguous, release stays blocked; DropCove will not use a heuristic or weaken the Path Reference lifecycle contract.

## Probe verdict — 2026-10-05

The [Ticket 01 probe](../../.scratch/reliable-drag-acceptance/probe-verdict.md) did not establish a reliable acceptance predicate. Accepted Copy and destination exit during `DragEnter` both returned `DRAGDROP_S_DROP` with `DROPEFFECT_COPY` in five of five correlated attempts; the exiting destination never received `Drop`. The native cutover remains unapproved by evidence. This ADR stays proposed, production drag-out is unchanged, and release remains blocked pending a new design decision.

## Controlled oracle correction — 2026-10-05

The [controlled follow-up](../../.scratch/reliable-drag-acceptance/controlled-probe-verdict.md) found that the initial inference did not exclude acceptance by a destination exposed underneath the disappearing target. A directly registered native target and an equivalent WinUI source both pass their complete 5×4 matrices over Reject backing. Over Copy backing, both report Copy with real acceptance by that backing target. This does not establish what handled the historical installed attempt. ADR remains proposed; an isolated installed reproduction and a planning decision are required before production cutover. No native advantage has been established by this comparison.

## Real DropCove verification — 2026-10-05

The user-approved test-profile seam enabled [real popup verification](../../.scratch/reliable-drag-acceptance/installed-profile-verdict.md) without replacing or mutating live user data. Actual DropCove WinUI passed the controlled 5×4 matrix over Reject backing; a further five attempts confirmed that acceptance by Copy backing legitimately consumes the participating Temporary reference after primary exit. Native replacement has no demonstrated advantage in these measured scenarios; see decision below.

## Decision — 2026-10-05

Rejected. Following controlled destination testing on native OLE, isolated WinUI, and real DropCove Release popup drags, the existing WinUI drag engine was proven to consistently retain Temporary references on destination rejection, cancellation, and unaccepted exits, while consuming references only when a destination genuinely receives and accepts the drop. Replacing the WinUI drag path with managed native OLE provided no measured reliability advantage and introduced unnecessary complexity. DropCove retains its existing WinUI drag implementation.
