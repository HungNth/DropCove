# 03: Mixed availability and ownership-safe Item Selection drag

**What to build:** Extend pointer Item Selection to the complete Shelf domain. A selected set containing Available, Missing, Unavailable, repeated paths, external references, and text or image Managed Files prepares once, presents one aggregate decision, transfers a deterministic deduplicated payload, and completes every represented occurrence without losing Pinned content, deleting external sources, leaking owned files, or reusing stale state.

**Blocked by:** 02: Manage Items pointer Item Selection Copy round trip.

**Status:** resolved

**Parent specification:** [DropCove 0.0.26 — Manage Items Item Selection and Drag-Out](../spec.md)

**Dependencies:** Ticket 01's item-set contract, Ticket 02's authoritative pointer-selected identities, existing availability classification, confirmation presentation, Managed File cleanup/retry, and serialized Core Shelf State mutation gate.

**Testing seam:** The real mixed-content Manage Items workflow and native Copy/cancel/reject destinations, supported by temporary SQLite/filesystem tests for Missing cleanup, Unavailable retention, path deduplication, occurrence completion, stale-token rejection, rollback, and Pending Managed File Deletion retry.

**Demo path:** Select external references, text and website-image Managed Files, repeated case-variant paths, same-name different-path items, one Missing item, and one Unavailable item. Exercise direct confirmation drag, click-to-arm/re-drag, stale selection mutation, cancel, reject, accepted Copy, and forced persistence/deletion failures.

- [x] Every represented occurrence is reclassified immediately before item-set preparation.
- [x] Confirmed Missing occurrences are removed durably, owned cleanup authority commits atomically, and empty batches are pruned before drag.
- [x] Missing cleanup remains committed when the later confirmation or destination drag is canceled or rejected.
- [x] Unavailable occurrences remain referenced and are excluded from the current payload.
- [x] If no Available occurrence remains, no token or OLE drag is created and Manage Items reports clear aggregate status.
- [x] Mixed Available and Unavailable occurrences show one in-popup aggregate confirmation using occurrence counts rather than unique path counts.
- [x] The confirmation preserves popup dimensions and supports direct primary-action drag, click-to-arm/re-drag, Cancel, and `Escape`.
- [x] The token is bound to exact source-ordered item identities and the authoritative Shelf snapshot.
- [x] Item Selection or Shelf mutation after arming invalidates and cancels the old token; the next drag prepares current visible state.
- [x] Outbound paths preserve source order and deduplicate exact paths and case variants with Windows ordinal case-insensitive comparison.
- [x] Same filenames at different paths remain distinct outbound entries.
- [x] Every represented Available occurrence remains recorded even when several occurrences emit one path string.
- [x] Accepted Copy consumes every represented Temporary occurrence and retains every occurrence currently Pinned at completion.
- [x] Canceled, rejected, and pre-acceptance-failed outcomes retain all non-Missing represented occurrences when no separate clear trigger occurred.
- [x] Consumed external references disappear only from Shelf state and never modify source filesystem objects.
- [x] Consumed Managed Files are permanently deleted or recorded for durable retry without restoring consumed Shelf Items.
- [x] A singleton plain-text item retains Unicode text-only; any multi-item payload exposes storage paths only and never concatenates text.
- [x] Persistence failure during preparation or accepted-Copy completion produces no partial visible or durable consumption and loses no cleanup authority.
- [x] Publish, refresh, or popup synchronization failure reports the error without replaying consumption or reviving stale selection.

## Evidence — 2026-10-26

- Native mixed-availability/deduplication and domain-isolation regressions passed with the complete **185/185** native suite. Existing rollback, Pin-at-completion, source safety, and Managed File tests now run through prepared item transfers where applicable.
- Windows popup with Available, Missing, and Unavailable references removed the Missing occurrence before showing one confirmation; direct primary-action Copy left the Unavailable occurrence and kept the popup open at one remaining item.
- Exact installed candidate exercised armed retry, then a separate armed selection mutation that canceled ready state. Rail-origin direct confirmation Copy preserved the external foreground application. An Unavailable-only survivor returned `No available items remain in this batch.` without consumption.
- Exact installed singleton text and mixed text/external storage payloads were captured by a real Forms OLE destination. Text bytes matched the Managed File; multi-item FileDrop paths had no UnicodeText format and copied exact bytes.
- A real Managed File sharing violation committed consumption and one durable pending deletion, showed the existing Shelf warning, and completed permanent deletion on a later cleanup-producing mutation after the handle was released.
- Shared final release evidence: [Ticket 06](06-release-dropcove-0.0.26.md).
