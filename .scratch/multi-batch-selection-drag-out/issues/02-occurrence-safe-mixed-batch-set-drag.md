# 02: Occurrence-safe mixed batch-set drag core

**What to build:** Make selected drag-out correct for the full Shelf domain: external references and Managed Files, Temporary and Pinned occurrences, repeated case-insensitive paths, Missing and Unavailable items, aggregate confirmation, stale prepared state, persistence failures, and owned-file cleanup. The destination receives one deterministic deduplicated path payload while DropCove retains every selected occurrence needed for exact lifecycle completion.

**Blocked by:** 01: Drop Shelf pointer Batch Selection Copy round trip.

**Status:** resolved

**Parent specification:** [DropCove 0.0.25 — Multi-Batch Selection and Drag-Out](../spec.md)

**Testing seam:** The real Drop Shelf mixed-content workflow with external files, text/image Managed Files, unavailable storage, and native Copy/cancel/reject, supported by the existing Core Shelf State temporary-profile SQLite/filesystem seam and the current aggregate confirmation component.

**Demo path:** Select batches containing duplicate paths, same-name different paths, Temporary and Pinned external references, text/image Managed Files, one Missing item, and one Unavailable item; confirm the available subset, copy it to a real destination, then verify exact emitted order, occurrence-specific retention/consumption, permanent owned-file cleanup or durable retry, and rollback on a forced persistence failure.

- [x] Batch-set preparation reclassifies every selected Shelf Item occurrence through the established Available/Missing/Unavailable rules.
- [x] Confirmed Missing Items are durably removed in one transaction, empty batches are pruned, sizing clears when the Shelf becomes empty, and required Managed File cleanup responsibility commits atomically.
- [x] Missing cleanup remains committed when the later confirmation or destination drag is canceled or rejected.
- [x] Unavailable occurrences remain in the Shelf and are excluded from the current outbound payload.
- [x] If no Available occurrence remains, preparation creates no token, starts no OLE drag, and reports one clear aggregate status.
- [x] Mixed Available and Unavailable selections show one aggregate confirmation with counts measured as Shelf Item occurrences.
- [x] The confirmation supports direct drag from the primary action, click-to-arm followed by a later card drag, Cancel, and `Escape`.
- [x] Cancel and `Escape` clear the prepared token while retaining Batch Selection and all non-Missing occurrences.
- [x] An armed prepared drag is bound to the exact ordered selected batch identifiers and authoritative Shelf snapshot that produced it.
- [x] Any Batch Selection change or Shelf mutation after arming invalidates and cancels the old token; the next drag prepares the currently visible state instead of reusing stale participants.
- [x] Preparation records every Available occurrence in displayed selected-batch order and source-item order.
- [x] Outbound path strings are deduplicated with Windows ordinal case-insensitive comparison while preserving the first occurrence's deterministic position.
- [x] Exact repeated paths and case variants emit one path string but retain every represented occurrence behind the token.
- [x] Same filenames at different paths remain distinct outbound paths and distinct participants.
- [x] No content hashing, byte comparison, shared Managed File backing, or destination-side deduplication is introduced.
- [x] Accepted Copy consumes every represented Temporary occurrence even when several occurrences map to one emitted path.
- [x] Accepted Copy retains every represented Pinned occurrence, including a Pinned occurrence sharing an emitted path with Temporary occurrences.
- [x] Current Pin state at completion remains authoritative; an occurrence pinned after preparation is retained.
- [x] Rejected, canceled, and pre-acceptance-failed completion retains all represented occurrences when no separate selection-clear trigger occurred.
- [x] Consumed external references disappear from the Shelf without modifying any source filesystem object.
- [x] Consumed Temporary Managed Files are permanently deleted after the durable item mutation or recorded as Pending Managed File Deletions for event-driven retry.
- [x] Pinned Managed Files remain byte-exact and owned after successful Copy.
- [x] Sharing, antivirus, or filesystem deletion failure leaves the Shelf mutation committed, reports the pending count through the established warning path, and retries without restoring the consumed item.
- [x] Persistence failure during Missing cleanup leaves the pre-preparation Shelf and cleanup authority unchanged.
- [x] Persistence failure during accepted-Copy consumption leaves all represented occurrences visible and durable and creates no unowned Managed File.
- [x] No prepared participant is consumed after an invalid token, stale token, failed native drag start, or failure before destination acceptance.
- [x] Preparation and completion serialize through the existing mutation gate and cannot overlap into two active authoritative prepared operations.
- [x] Whole-card single-batch drag uses the same batch-set core through a singleton identifier set; no compatibility command or duplicate implementation remains.
- [x] Manage Items direct item drag retains its established separate path resolution and completion behavior.
- [x] Final-item removal preserves automatic sizing reset, manual-height reset, Hidden/EdgeDocked content transitions, and footer/count disappearance.
- [x] Permanent native coverage uses real temporary SQLite/filesystem state for ordering, deduplication, mixed availability, all-Missing, all-Unavailable, empty-batch pruning, occurrence completion, Pin races, rollback, cleanup retry, restart, and Managed File source safety.
- [x] Permanent frontend coverage proves confirmation counts use occurrences, stale arming is invalidated by selection/Shelf changes, and Cancel/Escape retain selection.
- [x] A real mixed-content Drop Shelf smoke proves aggregate confirmation, exact native payload order, duplicate-path lifecycle, Temporary/Pinned behavior, external source safety, and Managed File cleanup while version metadata remains `0.0.24`.
- [x] No Git commit is created without separate explicit user authorization.

## Verification evidence — 2026-10-24

- Native Windows suite passed **179/179**, including selected occurrence deduplication/consumption, mixed Missing/Unavailable cancellation, empty/duplicate/unknown identifier rejection, preparation rollback preserving availability, and stale-start rejection after mutation. Existing Pin-at-completion and accepted-Copy rollback tests remained green.
- Real Drop Shelf mixed-availability drag showed one aggregate confirmation with `1 available` and `1 unavailable` occurrence, removed the Missing reference durably, retained selection on Escape, then accepted the armed retry and consumed only the Available occurrence.
- A controlled three-batch fixture contained six occurrences: external paths with case variants and same filenames at different paths, UTF-8 text Managed File, PNG Managed File, and Temporary/Pinned states. Actual native OLE Copy produced exactly five paths in deterministic order; every destination byte matched its source; only the two Pinned occurrences remained.
- A read-sharing lock on the Temporary text allowed destination Copy but prevented deletion. Consumption committed one durable Pending Managed File Deletion and displayed its warning. Releasing the lock and removing another reference retried cleanup, deleted the owned text, and removed the pending row while preserving the Pinned PNG and all 62 external source files.
- Frontend regressions cover direct confirmation drag, armed card retry exactly once, selection-change invalidation, late preparation cancellation after hide, and Escape ordering through the actual Shelf composable. Integrated final counts and installed-candidate repetition are owned by Ticket 05.
