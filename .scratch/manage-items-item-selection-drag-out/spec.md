# DropCove 0.0.26 — Manage Items Item Selection and Drag-Out

Status: resolved — 0.0.26 candidate accepted by the user
Design confirmation: approved by the user through `grill-with-docs` on 2026-10-26
Testing seam: confirmed by the user through `to-spec` on 2026-10-26 — one exact installed Windows Resident Instance exercising Manage Items from both Drop Shelf and Edge Rail origins, ordinary and virtualized item collections, real pointer and keyboard selection, Windows accessibility inspection, and native Copy/cancel/reject into real destinations, supported by frontend interaction/accessibility tests and Core Shelf State SQLite/filesystem regression seams
Release assignment: 0.0.26
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md#0026--manage-items-item-selection-and-drag-out)
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 8.1, 10, 14, 15, 19, and 20
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 2.3, 4, 8, 9, and 14
Relevant architectural decision: [ADR-0006 — Materialize non-filesystem drops as app-owned Managed Files](../../docs/adr/0006-managed-files-and-durable-cleanup.md)
Ticket decomposition: user approved six tracer-bullet tickets with DAG `01 → 02 → {03, 04} → 05 → 06` through `to-tickets` on 2026-10-26: [01 — Occurrence-safe item-set drag foundation](issues/01-occurrence-safe-item-set-drag-foundation.md), [02 — Manage Items pointer Item Selection Copy round trip](issues/02-manage-items-pointer-item-selection-copy-round-trip.md), [03 — Mixed availability and ownership-safe Item Selection drag](issues/03-mixed-availability-ownership-safe-item-selection-drag.md), [04 — Accessible Manage Items multiselectable grid](issues/04-accessible-manage-items-multiselectable-grid.md), [05 — Cross-origin Item Selection lifecycle integration](issues/05-cross-origin-item-selection-lifecycle-integration.md), and [06 — Release DropCove 0.0.26](issues/06-release-dropcove-0.0.26.md)
Ticket breakdown confirmation: approved by the user through `to-tickets` on 2026-10-26
Implementation authorization: granted by the user through `implement` on 2026-10-26 for the approved ticket DAG; candidate acceptance, Git commits, and deployment remain separate gates
Singleton text clarification: on 2026-10-26 the user explicitly chose to preserve 0.0.25 Unicode text-only behavior rather than introduce a dual-format payload; the earlier storage-plus-text wording was corrected.
Implementation evidence: all six tickets resolved; [Ticket 06](issues/06-release-dropcove-0.0.26.md) records the exact packaged artifact, 43 frontend and 185 native passing tests, installed Windows/OLE proof, and observation limits.
Human acceptance and commit authorization: on 2026-10-26 the user confirmed “tôi đã test và tất cả đã đạt, hãy commit các thay đổi”, accepting the candidate and explicitly authorizing the commit. This supersedes the earlier request to keep changes uncommitted; publishing and deployment remain separate.

## Problem Statement

Manage Items currently exposes item-level Pin/Unpin, Remove, availability, and direct drag-out, but each drag can transfer only one Shelf Item. A user who needs several items from one Shelf Batch must repeat the native drag gesture for every item even though the popup already presents the items as one ordered working set.

DropCove 0.0.25 established Explorer-style Batch Selection and an occurrence-safe prepared-transfer contract. That contract cannot accept Shelf Item identities: batch preparation validates Shelf Batch identities and represents every eligible occurrence in those batches. Manage Items needs a distinct item-occurrence preparation boundary that selects only the requested Shelf Items while reusing lower-level availability, deduplication, completion, and Managed File cleanup rules.

The popup is also a keyboard-accessible, virtualized surface containing independent Pin and Remove buttons. Item Selection must remain stable across mounted-row churn, avoid turning action buttons into selection-wide commands, preserve Edge Rail no-activation behavior, recover focus after item or batch removal, and retain the existing 240-pixel popup geometry.

## Solution

Add **Item Selection** as popup-local presentation state for the one Shelf Batch currently shown by Manage Items. Store stable Shelf Item identities, an optional range anchor, and a focused identity outside mounted virtual rows. Selection is never persisted, transferred to another popup, or shared across batches.

Use the 0.0.25 selection model adapted to a vertical item grid. Plain click selects one item, `Ctrl` toggles one item, and `Shift` replaces selection with the complete source-order range. Dragging a selected row prepares the complete Item Selection; dragging an unselected row prepares a singleton without first replacing the visible selection. Pin/Unpin and Remove retain their current per-item scope, while Bulk Pinning retains complete-batch scope.

Add a distinct serialized item-set preparation contract. It accepts unique Shelf Item identities belonging to the active management batch, resolves them in source order, reclassifies availability, durably removes Missing occurrences, retains Unavailable occurrences, deduplicates outbound paths ordinal case-insensitively, and records every represented Available occurrence behind one token. Accepted Copy consumes represented Temporary occurrences, retains currently Pinned occurrences, and permanently cleans owned Managed Files or records durable retry.

Expose the item collection as a multiselectable ARIA grid. One row drag region participates in roving focus; Pin/Unpin and Remove remain independently named buttons. Pointer opening does not activate or focus the popup. Keyboard opening from the Drop Shelf focuses the first item drag region. Selection changes use a coalesced polite announcement, while a pointer-opened Edge Rail origin remains no-activate and receives no focus restoration.

## User Stories

1. As a Manage Items user, I want to select several Shelf Items from the current batch, so that I can copy them with one native drag.
2. As a user, I want plain click to select one item, `Ctrl+click` to toggle, and `Shift+click` to select a contiguous source-order range, so that selection matches familiar Windows behavior.
3. As a user, I want off-screen virtualized items included in range selection, so that viewport position does not change selection meaning.
4. As a user, I want dragging a selected row to drag the complete selection and dragging an unselected row to drag only that row, so that an older selection does not capture an unrelated drag.
5. As a user, I want Pin/Unpin and Remove to keep their existing item scope, so that selection never silently retargets destructive or lifecycle actions.
6. As a keyboard user, I want a multiselectable grid with roving item focus and Space variants, so that selection does not require a pointer.
7. As a screen-reader user, I want explicit selected state and coalesced count announcements, so that the popup communicates the same selection shown visually.
8. As an Edge Rail user, I want pointer selection and drag-out without foreground activation, so that the application I am working in keeps focus.
9. As a user, I want Missing, Unavailable, Temporary, Pinned, external, and Managed content handled with the same safety guarantees as 0.0.25, so that multi-item drag cannot lose or leak data.
10. As a user, I want the popup to remain open when one item survives and close only when its batch disappears, so that a partial Copy or Remove does not interrupt the current task.

## Functional Requirements

### Selection ownership and ordering

- Item Selection belongs to one open Manage Items popup and therefore one active Shelf Batch.
- Store selected identities, the optional anchor, and the focused identity outside mounted virtual rows.
- Selection is session-only and is cleared rather than persisted or transferred when the popup closes or another batch replaces it.
- Source-item order, not click order or current viewport order, defines Shift ranges and drag payload order.
- Plain click selects only the target and sets the anchor.
- `Ctrl+click` toggles only the target and sets the anchor even when toggling it off.
- `Shift+click` replaces selection with the inclusive anchor-to-target range and preserves a valid anchor.
- With no valid anchor, Shift selects only the target and establishes it as anchor.
- `Ctrl+Shift` follows the same replacement-range behavior; additive ranges are not added.
- `Ctrl+A` selects all items without changing a valid anchor or creating an absent anchor.
- Newly added items are not selected. Pin changes do not change selection.
- Remove and confirmed-Missing cleanup prune only disappeared identities. Surviving identities remain selected; a removed anchor becomes absent without automatic replacement.

### Pointer and drag arbitration

- Selection gestures apply only to the row drag region. Pin/Unpin and Remove button activation does not select the row or change the selection anchor.
- A plain pointer press on an already selected row preserves the pre-press selection until release below the established drag threshold.
- Release below threshold collapses selection to that row; crossing the threshold drags the pre-press Item Selection.
- Crossing the threshold on an unselected row drags a singleton item set without first replacing visible selection.
- Selected rows use the established accent border and accent-subtle surface, with Windows Highlight resources in Contrast Themes.
- The header summary is `S selected · X/N pinned` while selection is non-empty and remains `X/N pinned` otherwise.
- No toolbar, footer, checkbox, selection mode, per-row badge, row-size change, or popup-size change is introduced.

### Action scope

- Per-item Pin/Unpin affects only the button's Shelf Item.
- Per-item Remove affects only the button's Shelf Item and retains existing source-file safety.
- Bulk Pinning affects the complete Shelf Batch, not Item Selection.
- Pin, Remove, and Bulk Pinning invalidate any armed prepared item transfer when Shelf state changes, but they do not otherwise retarget themselves to selection.

### Keyboard and accessibility

- The collection uses `role="grid"` and `aria-multiselectable="true"`; each item row exposes explicit `aria-selected="true"` or `"false"`.
- The row drag region is the roving grid focus target. Pin/Unpin and Remove remain independent native buttons with their existing accessible names and pressed state.
- A newly keyboard-opened popup from the Drop Shelf focuses the first item drag region. A pointer-opened popup does not force focus or foreground activation.
- Up/Down moves by one source item. Home/End moves to the first/last item and mounts virtualized targets before focus moves.
- `Space` selects only the focused item, `Ctrl+Space` toggles it, `Shift+Space` selects the anchor range, and `Ctrl+Shift+Space` uses replacement-range behavior.
- `Ctrl+A` operates only while the item grid owns focus and selects the complete batch.
- The cyclic Tab order is Bulk Pinning, Close, the grid's primary drag-region stop, then item Pin/Unpin and Remove controls in source order. Virtualized action targets are mounted and scrolled into view before focus moves.
- Selection changes announce `1 item selected` or `N items selected`; clearing announces `Selection cleared`. Synchronous changes coalesce to the latest message.
- If the focused item disappears while the batch survives, focus moves to the item now at the same source index, otherwise the previous item, otherwise Bulk Pinning.
- If explicit close occurs after the batch becomes a one-item card, focus returns to that card's drag region because its Manage Items control no longer exists.
- If the batch disappears, the Drop Shelf recovers to the batch now at the same displayed index, otherwise the previous batch, otherwise More.
- A pointer-opened Edge Rail origin receives no keyboard focus or live announcement ownership and remains no-activate throughout selection and drag-out.
- Keyboard selection does not add keyboard-only native OLE destination drag.

### Selection lifecycle and Escape

- Rejected, canceled, or pre-acceptance-failed drag retains Item Selection when no separate clear trigger occurred.
- Successful Copy, empty collection-space click, selection-level `Escape`, explicit popup close, opening another batch, owner-driven popup closure, or restart clears Item Selection.
- `Escape` cancels an active item availability confirmation and prepared token first.
- During an active native item drag, `Escape` cancels the native drag before a later press can clear selection.
- With no active confirmation or drag, `Escape` clears non-empty Item Selection before a later press closes Manage Items.
- The popup remains open while its Shelf Batch exists, including when exactly one Shelf Item remains.
- The popup closes when the batch disappears, the shelf hides, shelf resizing starts, or whole-batch/Batch Selection drag starts.
- A close or hide request during an active native item drag becomes pending. The recorded transfer completes independently; the popup then closes and discarded selection is not restored after rejection or cancellation.

### Item-set preparation and completion

- Item-set preparation is distinct from batch-set preparation. Item identities are never passed through the batch preparation API.
- The item-set boundary validates a non-empty unique identity set and requires every identity to belong to the active Manage Items batch.
- Preparation serializes through the existing Core Shelf State mutation gate and binds its token to the exact source-ordered item identities plus the authoritative Shelf snapshot.
- Every represented occurrence is reclassified immediately before preparation as Available, Missing, or Unavailable.
- Confirmed Missing occurrences are durably removed before drag, empty batches are pruned, and owned cleanup responsibility commits atomically with that mutation.
- Unavailable occurrences remain in the Shelf and are excluded from the current payload.
- If no Available occurrence remains, no token or OLE drag is created and Manage Items reports a clear aggregate status.
- When Available and Unavailable occurrences are mixed, one aggregate confirmation reports occurrence counts, not unique path counts.
- The confirmation is rendered inside Manage Items without resizing the popup. Its primary action supports direct drag or click-to-arm followed by dragging the same selected row set again.
- Cancel or `Escape` releases the token and retains selection and every non-Missing occurrence.
- Any Item Selection or Shelf mutation invalidates an armed token. The next drag prepares the currently visible state rather than retargeting the stale token.
- Missing cleanup remains committed when the later confirmation or destination drag is canceled or rejected.
- Outbound paths preserve source-item order and use Windows ordinal case-insensitive deduplication. Same filenames at different paths remain distinct.
- The prepared transfer records every represented Available occurrence even when repeated paths emit only once.
- Accepted Copy consumes every represented Temporary occurrence. Current Pin state at completion is authoritative; represented items pinned after preparation remain.
- Rejected, canceled, and pre-acceptance-failed outcomes retain all represented occurrences when no separate popup-lifetime clear trigger occurred.
- Consuming external references never deletes or modifies their source filesystem objects.
- Consumed Managed Files are permanently deleted after durable item mutation or recorded as Pending Managed File Deletions for event-driven retry.
- A real sharing/deletion failure leaves consumption committed, reports the pending cleanup count, and never restores the consumed Shelf Item.

### Payload representations

- Exactly one represented Available plain-text Managed File retains the existing Unicode text-only OLE representation from 0.0.25; it does not advertise file-drop storage formats. Multi-item payloads use storage paths instead.
- A multi-item payload exposes only file/folder storage paths. DropCove does not concatenate, merge, or otherwise synthesize Unicode text from several items.
- Drag-out remains Copy-only and retains the existing cross-integrity limitation.

### Transaction and failure invariants

- Persistence failure before Missing cleanup commits leaves the pre-preparation Shelf and cleanup authority unchanged.
- Failure after a preparation token is allocated must cancel or release that token without changing visible selection.
- Accepted-Copy occurrence consumption and required pending Managed File deletion records commit atomically.
- Persistence failure during accepted-Copy completion leaves represented occurrences and cleanup authority unchanged; no partial consumption is visible or durable.
- Permanent deletion failure after a successful durable mutation records retry responsibility and does not roll back the consumed Shelf Item.
- A publish, projection refresh, or popup-close failure must report its error without replaying consumption, restoring discarded selection, or losing cleanup responsibility.

## Testing Decisions

- Primary seam: one exact installed Windows x64 Resident Instance using embedded production assets, owned Profile Data, the real shared Manage Items popup opened from both Drop Shelf and Edge Rail, ordinary and more-than-50-item batches, real pointer modifiers and keyboard input, Windows accessibility inspection or a screen reader, and real native Copy/cancel/reject destinations.
- Supporting frontend coverage uses the existing Vue interaction, virtualization, and accessibility seams to exercise identity selection, anchors, pointer threshold arbitration, keyboard commands, Tab order, focus recovery, ARIA selected state, header counts, and coalesced announcements through observable DOM behavior.
- Supporting native coverage uses real temporary SQLite databases and temporary directories to exercise item-set validation, source ordering, availability preparation, Missing cleanup, path deduplication, occurrence-safe completion, rollback, Managed File cleanup, pending deletion retry, and restart.
- Prefer behavior tests that fail for wrong selected identities, wrong source range, stale payload, invalid action scope, wrong occurrence consumption, lost focus, invalid accessibility state, external-source deletion, or leaked Managed Files.
- Do not test private helper calls, exact collection types, SQL text, CSS class strings, command forwarding, mock echoes, or source-code wording.
- Real installed acceptance must observe pointer no-activation from the Edge Rail rather than infer it from unit tests.
- Real installed accessibility acceptance must record the grid role, multiselectable state, selected rows, focus movement, Tab sequence, announcement text, and focus owner.
- Real OLE acceptance must include successful Copy, Escape cancellation, and destination rejection with Temporary/Pinned, external/Managed, Missing/Unavailable, repeated-path, and singleton-text cases.
- Version metadata remains `0.0.25` during behavior implementation and changes to `0.0.26` only in an approved release ticket.

## Acceptance Criteria

### Selection and presentation

- Plain, Ctrl, Shift, Ctrl+Shift, no-anchor, toggle-off-anchor, and Ctrl+A behavior match this specification below and above the virtualization threshold.
- Drag threshold arbitration proves selected-row drag uses the pre-press set and unselected-row drag uses a singleton.
- Selected visuals, `S selected · X/N pinned`, action isolation, empty-space clear, and one-item popup survival work in Dark, Light, and Windows Contrast presentation.

### Keyboard, accessibility, and focus

- The real popup exposes a multiselectable grid with explicit selected rows and one roving item drag-region focus target.
- Up/Down/Home/End, Space variants, Ctrl+A, cyclic Tab order, and the three-stage Escape ordering remain correct with ordinary and virtualized item collections.
- Focus recovery covers removal at the beginning, middle, and end, one-item survival, final-batch disappearance, explicit close with no Manage Items control, and Edge Rail no-focus behavior.
- Announcements coalesce to the latest `N items selected` or `Selection cleared` message.

### Transfer correctness and safety

- Selected item identities are validated against the active batch and resolved in source order independent of click order.
- Repeated paths and case variants emit once while every represented occurrence remains recorded; same filenames at different paths remain distinct.
- Mixed Available/Unavailable content shows one occurrence-count confirmation; all-Unavailable or all-Missing content starts no empty drag.
- Missing cleanup, stale-token invalidation, direct confirmation drag, armed retry, Cancel, and Escape match the approved lifecycle.
- Accepted Copy consumes represented Temporary occurrences, retains current Pinned and Unavailable occurrences, leaves external sources byte-exact, and permanently cleans or durably queues owned files.
- Canceled, rejected, and failed operations retain occurrences and selection unless a separate close/hide/restart clear trigger occurred.
- Singleton text exposes Unicode text; multi-item content exposes storage paths only.
- Persistence and deletion failures satisfy the transaction invariants without partial consumption or lost cleanup authority.

### Release proof

- Frontend tests, frontend typecheck/production build, native Rust regressions, and the freshly cargo-xwin-built Windows regression executable pass.
- One exact installed 0.0.26 candidate runs embedded assets with the development server unavailable and passes the primary seam.
- Restart, same-candidate reinstall/update, standard uninstall, and Profile Data preservation pass before human candidate acceptance.
- Exact installer/executable hashes, test counts, Windows/accessibility tooling, destination applications, observed focus owner, and unobserved physical limitations are recorded.

## Demo Path

1. Open a multi-item batch from the Drop Shelf by keyboard and verify focus starts on the first item drag region.
2. Build a non-contiguous Item Selection with Ctrl, replace it with an off-screen Shift range in a batch above 50 items, and inspect selected rows plus the coalesced announcement.
3. Tab through Bulk Pinning, Close, the grid stop, and virtualized Pin/Remove actions; remove the focused item and observe same-index focus recovery.
4. Drag a selected row to a real destination and verify source-order payload, repeated-path deduplication, Temporary consumption, Pinned retention, and exact external-source bytes.
5. Repeat with an unselected row while another selection is visible and verify only the row participates.
6. Exercise mixed Available/Unavailable content through direct confirmation drag and click-to-arm/re-drag; mutate selection after arming and verify stale-token cancellation.
7. Exercise singleton plain-text Managed File drag and verify Unicode text; select it with another item and verify the multi-item payload is storage-only.
8. Cancel and reject native drags, then request close during an active drag and verify pending close completes without restoring discarded selection.
9. Reduce the batch to one surviving item and verify the popup remains open; close it and verify focus returns to the one-item card drag region.
10. Open Manage Items from the Edge Rail while another application owns foreground focus; select and drag with the pointer and verify DropCove never takes foreground focus.
11. Force persistence and Managed File deletion failures and verify rollback or durable pending cleanup according to the transaction stage.
12. Restart, reinstall/update, uninstall, and verify retained Profile Data and source safety with the exact candidate.

## Dependencies and Blockers

- Depends on the completed 0.0.25 Batch Selection prepared-transfer and occurrence-completion primitives, but does not reuse the batch identity API for item identities.
- Depends on the existing Manage Items popup, virtual collection, announcement channel, Shell Visual, item Pin/Remove, Bulk Pinning, and single-item drag behavior.
- Depends on existing Core Shelf State serialization, availability classification, SQLite transaction, Managed File ownership, Pending Managed File Deletion, and Windows OLE adapters.
- No new dependency, persistence schema, setting, platform, popup geometry, or ADR is required by the approved design.
- Implementation is authorized for the approved six-ticket DAG. Human candidate acceptance, Git commits, and deployment remain separate explicit gates.

## Out of Scope

- Persistent, named, cross-popup, or cross-batch Item Selection.
- Item selection outside the active Manage Items batch.
- Selection-scoped Pin/Unpin, Remove, Bulk Pinning, bulk delete, or partial-failure bulk-action UI.
- Checkboxes, selection mode, a new toolbar/footer, item count badges, popup resizing, row-density changes, or per-origin geometry.
- Item or batch reordering, lasso/marquee selection, additive Ctrl+Shift ranges, inversion, or selection history.
- Concatenating multiple text items or inventing a new combined text representation.
- Keyboard-only native OLE destination drag, Move/Link semantics, cross-integrity guarantees, or non-Windows behavior.
- Changing drag-in formats, Attachment Services, Managed File ownership, deletion authority, appearance modes, Shake, fullscreen, or monitor-placement policy.
- Beginning implementation, synchronizing release metadata, accepting a candidate, creating a Git commit, or deploying without the corresponding separate authorization.

## Further Notes

The central invariant is that payload strings and lifecycle occurrences are different things. A deduplicated Windows path list may contain one string while the prepared transfer records several Shelf Item occurrences. Completion follows recorded occurrences and current Pin state, never payload string count.

The Item Selection owner is deliberately disposable. Stable identities make virtualization invisible, but selection never becomes durable Shelf state and never changes Pin scope or ownership authority by itself.

The grid role is required because rows contain independent Pin and Remove buttons. A listbox option cannot safely contain those interactive descendants without creating a second, conflicting keyboard model.

This specification and its six-ticket DAG are resolved. The user accepted the 0.0.26 candidate and authorized the Git commit. Publishing and deployment remain separate explicit gates.
