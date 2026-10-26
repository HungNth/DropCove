# 01: Occurrence-safe item-set drag foundation

**What to build:** Cut the existing Manage Items row drag over to the final item-set preparation and completion contract. A user dragging one row keeps the established direct Copy workflow, while the underlying contract accepts an ordered item identity set, validates its active-batch boundary, records exact occurrences, and completes Temporary/Pinned and Managed File lifecycle safely without treating item identities as batch identities.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.26 — Manage Items Item Selection and Drag-Out](../spec.md)

**Dependencies:** The completed Manage Items direct row drag, Core Shelf State serialization, availability classification, Windows Copy adapter, Managed File ownership, and Pending Managed File Deletion lifecycle.

**Testing seam:** The real existing Manage Items row drag into native Copy/cancel/reject destinations, supported by Core Shelf State tests using temporary SQLite databases and real temporary directories for identity validation, occurrence completion, rollback, and owned-file cleanup.

**Demo path:** Open Manage Items for a mixed batch and drag one Available Temporary external item, one Pinned plain-text Managed File, one Missing item, and one Unavailable item. Verify Copy/cancel/reject, singleton Unicode text, source safety, item retention or consumption, and cleanup state.

- [x] The item-set boundary accepts a non-empty ordered identity set, rejects duplicate or unknown identities, and requires every identity to belong to the active Manage Items batch.
- [x] Item identities never pass through the batch-set preparation API, and the old standalone item completion path is removed rather than retained as a compatibility route.
- [x] The existing one-row drag supplies a singleton item set and preserves its user-visible Copy-only behavior.
- [x] Preparation binds the exact source-ordered item identities and authoritative Shelf snapshot to one token.
- [x] A singleton Available external reference reaches a real destination without modifying or deleting its source object.
- [x] Accepted Copy consumes a Temporary singleton and retains a Pinned singleton; canceled, rejected, and pre-acceptance-failed outcomes retain both.
- [x] Current Pin state at completion remains authoritative.
- [x] A singleton plain-text Managed File retains the existing Unicode text-only OLE representation, without file-drop storage formats.
- [x] Confirmed Missing singleton cleanup commits durably before drag; an Unavailable or no-Available singleton starts no OLE drag and remains referenced.
- [x] Consumed Managed Files are permanently deleted after durable mutation or recorded for event-driven pending deletion retry.
- [x] Accepted-Copy item mutation and required pending cleanup authority commit atomically.
- [x] Persistence failure leaves visible occurrences and cleanup authority unchanged without partial consumption.
- [x] Permanent deletion failure after a committed consumption reports pending cleanup without restoring the consumed Shelf Item.
- [x] Existing popup synchronization, source-file safety, and final-batch removal behavior remain intact.

## Evidence — 2026-10-26

- Red: the new Core Shelf State test failed to compile because item-set preparation did not exist. Green: the exact source-order/batch-boundary test and the complete freshly compiled Windows native suite passed; final suite **185/185**.
- Item and batch tokens are domain-isolated; popup open/close cancels pending item tokens only. Matching stale tokens may be canceled but cannot be started, preserving concurrent-transfer completion safety.
- Exact installed candidate copied an unselected external row alone; singleton text delivered byte-exact `Xin chào DropCove\r\n第二行 🚀` with Unicode text and no FileDrop. Pinned text survived; Temporary text consumption and real delete-denying handle produced durable pending cleanup, then event-driven deletion after handle release.
- Singleton text-only behavior was explicitly reconfirmed by the user; the earlier dual-format baseline description was corrected rather than expanding the OLE adapter.
- Shared final release evidence: [Ticket 06](06-release-dropcove-0.0.26.md).
