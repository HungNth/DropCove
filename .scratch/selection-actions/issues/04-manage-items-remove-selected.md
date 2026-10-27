# 04: Manage Items ownership-safe Remove Selected

**Parent specification:** [DropCove 0.0.28 — Selection Actions](../spec.md)

**What to build:** Complete focus-capable Manage Items Selection Actions by adding immediate Remove Item for one selected row and confirmed Remove Selected for several rows, with exact item counts, authoritative ownership effects, atomic item removal, Managed File cleanup, stale-target safety, popup survival/closure, and deterministic focus recovery.

**Blocked by:** 02 — Drop Shelf ownership-safe Remove Selected; 03 — Manage Items Item Selection Pin/Unpin.

**Status:** ready-for-agent

**Dependencies:** Ticket 02's authoritative confirmation/removal lifecycle, Ticket 03's Item Selection menu and item-domain mutation contract, existing Manage Items focus recovery and popup lifetime, and Pending Managed File Deletion behavior.

**Testing seam:** The focus-capable Manage Items popup opened from Drop Shelf with ordinary and virtualized all-External, mixed, and all-Managed batches, supported by frontend modal/focus tests and Core Shelf State item-removal, rollback, source-safety, and sharing-violation regressions.

**Demo path:** Remove one selected item immediately, then select several rows, inspect `N items` plus ownership-specific copy, Cancel, Confirm while unselected items survive, remove the final selected set so the batch disappears, and repeat with stale and locked Managed File targets.

- [ ] One selected Manage Items row exposes Remove Item and retains immediate singleton behavior.
- [ ] Two or more selected rows expose Remove Selected and always require confirmation.
- [ ] Confirmation reports exact authoritative `N items` scope and ownership counts without trusting caller-supplied paths, labels, or counts.
- [ ] All-External, mixed-ownership, and all-Managed copy matches the canonical source-safety and permanent-deletion contract.
- [ ] The immutable confirmation target includes the active Shelf Batch identity and exact selected item identities; changed batch ownership or membership rejects the complete operation.
- [ ] Confirm rejects missing, duplicate, stale, wrong-domain, and cross-batch item identities with no partial removal.
- [ ] One item-selection removal transaction removes exactly the selected Shelf Items, records every required Pending Managed File Deletion, prunes the batch when empty, and publishes one result.
- [ ] Unselected Shelf Items survive unchanged and in source order; the popup remains open while its active batch exists, including one-item survival.
- [ ] Removing the final items closes Manage Items and applies established parent-surface focus fallback without leaving an empty popup.
- [ ] External source objects remain byte-exact; Managed Files delete permanently or enter durable pending retry without restoring removed items.
- [ ] Persistence and confirmation-preparation failure preserve visible/restored Shelf state, Managed bytes, selection, and focus ownership.
- [ ] Focus-capable modal confirmation initially focuses Cancel, blocks competing popup and Shelf mutations, and restores focus to the invoking row when it survives.
- [ ] Cancel, Escape, setup failure, stale-target rejection, and persistence failure retain Item Selection; success prunes only disappeared identities.
- [ ] Ordinary and virtualized focus recovery covers first, middle, last, unmounted, one-item-survivor, and final-batch-removal cases.
- [ ] Existing Item Selection drag, Bulk Pinning, independent row controls, Item Actions Menu, availability, privacy, and popup geometry regressions remain green.
- [ ] Rail-origin Manage Items remains unchanged until Ticket 05 adds the complete pointer-only/no-activate confirmation workflow.
