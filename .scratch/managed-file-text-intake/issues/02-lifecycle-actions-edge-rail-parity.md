# 02: Lifecycle actions and Edge Rail parity

**What to build:** Extend the working Managed File text round trip across every established Shelf lifecycle and both presentation surfaces. Users can accept text through the expanded Edge Rail, Pin/Unpin it, remove it, clear Temporary content, use Drop Shelf and Rail confirmations with truthful deletion copy, and rely on mixed external/Managed File workflows without source-object risk or display-state regressions.

**Blocked by:** 01: Managed-text Drop Shelf round trip.

**Status:** resolved

**Parent specification:** [DropCove 0.0.23 — Managed File Lifecycle and Plain-Text Intake](../spec.md)

**Testing seam:** Actual Windows Resident Instance with Drop Shelf and no-activate Edge Rail surfaces, plus authoritative Core Shelf State lifecycle regressions using mixed external references and Managed File ownership.

**Demo path:** Create external and text batches, dock to the Edge Rail, accept another text drop without focus theft, Pin one text item, clear Temporary content, remove the Pinned text batch explicitly, and verify only DropCove-owned files are permanently deleted.

- [x] Expanded Edge Rail accepts valid Unicode text through the same representation, limit, naming, staging, ownership, and publication contract as the Drop Shelf.
- [x] Rail drag-entry expansion delay, placeholder, leave/collapse timing, pointer reconciliation, fullscreen policy, transient holds, and no-activate behavior remain unchanged.
- [x] Opening, accepting, rejecting, or failing a Rail text drop does not steal foreground focus.
- [x] Pin/Unpin and Bulk Pinning continue to affect lifecycle only; Pinned Managed Files survive Successful Drag-Out, restart, update, and uninstall.
- [x] Remove Item removes the exact occurrence and records cleanup only when that occurrence owns a Managed File; same-path external or retained occurrences remain safe.
- [x] Remove Batch records cleanup for every owned item in the batch and never deletes external source objects, regardless of Pin state.
- [x] Confirmed-Missing cleanup removes a missing Managed File reference and treats its already-absent owned target as completed cleanup.
- [x] Clear Temporary Items removes external Temporary references, queues/deletes Temporary Managed Files, preserves every Pinned Item, prunes empty batches, and retains existing sizing/display reset behavior.
- [x] Drop Shelf Clear remains a fixed `360 × 130` decorated confirmation with Cancel initially focused, modal interaction blocking, Escape/window-close cancellation, and stale-empty no-op behavior.
- [x] Drop Shelf and Edge Rail confirmations use the canonical explanation: `Pinned items stay. External sources stay. DropCove-created temporary files are permanently deleted.`
- [x] The Edge Rail keeps its local confirmation and transient-hold lifecycle rather than opening the Drop Shelf confirmation window.
- [x] Pending deletion failures from Remove Item, Remove Batch, Clear, Successful Drag-Out, and confirmed-Missing cleanup share one durable count and retry path.
- [x] Cleanup-producing actions retry earlier pending deletions without blocking the new Shelf mutation or adding another modal.
- [x] Mixed external/Managed File batches and overlapping paths prove that ownership relation—not path prefix, filename, Pin state, or batch membership—controls deletion.
- [x] Final-item removal from either surface hides all Shelf surfaces and resets pending/manual sizing intent to the established `191 × 180` opening state.
- [x] Manage Items lifecycle, focus restoration, guarded Escape drag cancellation, item availability, Shell Visuals, status feedback, and popup geometry remain unchanged.
- [x] Shake-to-open accepts text on the transient Drop Shelf, promotes only a newly accepted batch, and restores prior Hidden/EdgeDocked state after unsupported, duplicate-independent failure, cancellation, or rejection.
- [x] Dark, Light, Windows Contrast, reduced-motion, accessibility names, tooltips, keyboard actions, and source-path identity remain coherent with the updated Clear wording.
- [x] Actual Windows smoke covers both surfaces, Pin/Unpin, native Copy/cancel/reject, Remove Batch, seeded Remove Item ownership, Clear cancel/confirm, sharing retry, source safety, and final display transitions.
- [x] Frontend regressions/build, cargo-xwin Windows build, and the freshly compiled native regression executable pass while release metadata remains `0.0.22`.
- [x] No Git commit is created without separate explicit user authorization.

## Comments

- Started after resolution of Ticket 01. Core lifecycle mutations, Clear confirmation copy, Edge Rail intake and error handling are implemented in the vertical codebase. Proof requires both surfaces and mixed external/Managed File verification. No Git commit is authorized.

## Answer

Verified end to end. Edge Rail and Drop Shelf share the unified text intake and error channels without stealing focus. The canonical Clear confirmation explanation is synchronized across `ClearConfirmationView.vue`, `ClearTemporaryConfirmation.vue`, and the specifications. Remove Item, Remove Batch, Clear Temporary Items, and confirmed-missing cleanup queue deletions exclusively for owned items while keeping external source objects intact byte-for-byte. All 142 native tests passed. Release metadata remains `0.0.22`. No commit created.