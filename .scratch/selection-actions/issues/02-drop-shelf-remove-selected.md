# 02: Drop Shelf ownership-safe Remove Selected

**Parent specification:** [DropCove 0.0.28 — Selection Actions](../spec.md)

**What to build:** Complete Drop Shelf Selection Actions by adding immediate singleton Remove Batch and confirmed Remove Selected for several Shelf Batches, with authoritative counts, ownership-safe copy, atomic reference cleanup, durable Managed File deletion responsibility, stale-target rejection, and focus-capable modal behavior.

**Blocked by:** 01 — Drop Shelf Batch Selection Pin/Unpin round trip.

**Status:** ready-for-agent

**Dependencies:** Ticket 01's shared Selection Actions Menu and batch-selection mutation boundary, existing Remove Batch and Pending Managed File Deletion behavior, sizing reset, local status/announcement channels, and modal-ownership guards.

**Testing seam:** The real Drop Shelf with External-only, mixed-ownership, and Managed-only selections, supported by frontend confirmation/focus tests and Core Shelf State temporary SQLite/filesystem failure-injection and sharing-violation regressions.

**Demo path:** Remove one selected batch immediately; then select several batches containing External References and Managed Files, inspect `N batches · M items` plus ownership-specific warning, Cancel once, Confirm once, and repeat with a locked Managed File and a stale target.

- [ ] One selected Shelf Batch exposes Remove Batch and retains the established immediate behavior regardless of affected item count.
- [ ] Two or more selected Shelf Batches expose Remove Selected and always require confirmation, including all-External selections.
- [ ] Confirmation reports exact authoritative `N batches · M items` scope and never trusts presentation-provided ownership or count values.
- [ ] All-External copy states that source files and folders will not be deleted; mixed copy distinguishes safe External sources and reports the exact Managed File count; all-Managed copy reports the exact permanent-deletion count.
- [ ] Opening Remove Selected closes the Selection Actions Menu before confirmation appears.
- [ ] The confirmation target is immutable and bound to the exact authoritative batch identities, contained item identities, ownership counts, and relevant Shelf state.
- [ ] Confirm rejects stale, missing, duplicate, or wrong-domain targets without shrinking scope or partially mutating the Shelf.
- [ ] One batch-selection removal transaction removes every selected Shelf Batch, records all required Pending Managed File Deletions, prunes final content, and publishes one resulting snapshot.
- [ ] External source objects remain byte-exact and are never deleted, moved, renamed, overwritten, or modified.
- [ ] Managed Files are permanently deleted after durable mutation or remain represented by durable pending-deletion records when Windows sharing, antivirus, or filesystem failure prevents immediate deletion.
- [ ] Persistence failure leaves visible and restored Shelf state, cleanup authority, owned bytes, sizing, and selection unchanged.
- [ ] Confirmation blocks selection changes, Pin/Unpin, Remove, Clear Temporary Items, accepted drop-in mutation, drag start, window movement, resize, and competing Shelf mutations on the Drop Shelf.
- [ ] Cancel, Escape, setup failure, stale-target failure, and persistence failure retain Batch Selection and restore focus to the invoking card when it survives.
- [ ] Success prunes disappeared selected identities, applies final-item sizing/display transitions, and uses existing deterministic focus fallback when the invoking card disappears.
- [ ] Focus-capable modal semantics expose exact heading/body/actions, initially focus Cancel, trap destructive intent behind explicit Remove, and use established Dark/Light/Contrast danger resources.
- [ ] Prepared drag state bound to the prior Shelf snapshot is invalidated by successful removal; existing batch drag and Item Actions regressions remain green.
