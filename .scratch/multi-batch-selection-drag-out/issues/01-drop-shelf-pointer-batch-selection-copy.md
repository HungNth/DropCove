# 01: Drop Shelf pointer Batch Selection Copy round trip

**What to build:** Make the first narrow but complete multi-batch workflow work through the Drop Shelf: select several Shelf Batches with pointer gestures, see stable selected visuals and counts across ordinary and virtualized collections, drag the selected cards as one ordered native Copy payload, retain the set after cancellation or rejection, clear it after accepted Copy, and preserve external source safety. This slice establishes the selection owner and singleton-or-selected batch-set path that later tickets harden for mixed availability, duplicate paths, Managed Files, Edge Rail, and full keyboard accessibility.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Parent specification:** [DropCove 0.0.25 — Multi-Batch Selection and Drag-Out](../spec.md)

**Testing seam:** Actual Windows Resident Instance with the real Drop Shelf/WebView2 surface, real pointer modifiers, distinct Available external paths, ordinary and >50-batch collections, and a real native Copy/cancel/reject destination, supported by frontend interaction/virtualization and Core Shelf State SQLite regressions.

**Demo path:** Create at least three distinct external-reference batches, use plain click, `Ctrl+click`, and `Shift+click` to form a selection, drag one selected card into a real destination, repeat with cancellation and rejection, then drag an unselected card and verify it transfers alone.

- [x] Batch Selection is presentation state owned by the Drop Shelf and stores stable Shelf Batch identities outside mounted virtual rows.
- [x] Plain click on a card drag region selects only that Shelf Batch and sets the selection anchor.
- [x] `Ctrl+click` toggles only the target batch and sets the anchor even when the target is toggled off.
- [x] `Shift+click` replaces selection with the inclusive anchor-to-target range in displayed newest-first row-major order.
- [x] With no valid anchor, `Shift+click` selects only the target and establishes it as anchor.
- [x] `Ctrl+Shift+click` uses the same replacement-range behavior and does not add an additive range mode.
- [x] Range selection includes batches outside the current viewport and remains correct above the 50-batch virtualization threshold.
- [x] A plain pointer press on an already selected card does not collapse selection until release below the established drag threshold.
- [x] Crossing the drag threshold on a selected card uses the pre-press Batch Selection; crossing it on an unselected card uses a singleton set.
- [x] One-item and multi-item cards use the same batch-set drag rule. Direct one-item drag remains available only from Manage Items.
- [x] Selected cards use the existing accent border and accent-subtle surface, with a distinct visible focus treatment reserved for Ticket 04.
- [x] The Drop Shelf footer shows `N batches · M items`, where `M` is the current Shelf Item occurrence count across selected batches.
- [x] Pin/Unpin, Bulk Pinning, and opening or closing Manage Items do not alter selected identities.
- [x] Accepted drop-in creates an unselected newest batch without clearing existing selection.
- [x] Remove Item updates selected item counts while the batch survives; Remove Batch or empty-batch pruning removes only the vanished identity.
- [x] A removed anchor becomes absent without auto-selecting a replacement.
- [x] Clicking empty collection space clears selection without invoking a card action or incoming-drop behavior.
- [x] Surface hide and Resident restart clear Drop Shelf selection. Edge Rail surface-switch behavior remains owned by Ticket 03.
- [x] Starting a batch-set preparation closes Manage Items before the native drag begins.
- [x] The Core Shelf State accepts an ordered non-empty batch identifier set; a singleton set is the clean replacement for the old whole-card single-batch path.
- [x] For distinct Available external paths, preparation flattens selected batches in displayed newest-first order and preserves source-item order inside each batch.
- [x] The Windows OLE adapter receives one read-only Copy payload containing all prepared paths in deterministic order.
- [x] Accepted Copy consumes represented Temporary external references and retains represented Pinned references without deleting, moving, renaming, or overwriting external sources.
- [x] Rejected, canceled, and pre-acceptance-failed native drags retain represented Shelf Items and the current Batch Selection.
- [x] Accepted Copy clears Batch Selection and refreshes counts, sizing, and empty-Shelf presentation from the authoritative result.
- [x] Dragging an unselected card while another selection exists transfers only the singleton batch and does not consume the unrelated selected batches.
- [x] Existing one-item Manage Items drag, Copy-only cursor/effect, application-owned drag rejection, Clear confirmation blocking, movement/resize blocking, and status behavior remain unchanged.
- [x] Permanent frontend tests cover pointer gestures, anchor behavior, threshold arbitration, action isolation, counts, selection clearing/retention, and virtualization through observable interaction rather than source strings.
- [x] Permanent native tests cover ordered distinct-path batch-set preparation and accepted/canceled completion through real temporary SQLite state.
- [x] A real Drop Shelf pointer smoke proves multi-batch Copy, cancellation/rejection retry, singleton drag, external source safety, and >50-batch selection while version metadata remains `0.0.24`.
- [x] No Git commit is created without separate explicit user authorization.

## Comments

- User authorized implementation of all five approved tickets on 2026-10-24. Ticket 01 is active; version metadata remains `0.0.24` until Ticket 05 and no Git commit is authorized.

## Verification evidence — 2026-10-24

- Real pre-release Windows Resident/WebView2 smoke at version `0.0.24` created 60 batches, selected non-adjacent cards with plain/Ctrl click, selected a 57-batch range across virtualized rows, and verified accurate counts with only six rows mounted.
- Actual Windows pointer input drove native OLE Copy into an owned `IDropTarget`: destination received `file-59.txt` then `file-57.txt` with exact bytes; Shelf count changed `60 → 58`; all 62 owned external source files stayed unchanged.
- Native Escape cancellation and a rejecting destination both retained the two-batch selection and 58 references. Dragging an unselected singleton consumed only that card (`58 → 57`) and cleared selection after accepted Copy.
- Pinning and later intake retained the selected identities and occurrence count. Restart cleared presentation selection and retained durable Shelf content.
- Frontend selection/confirmation regressions, Vue typecheck/production build, and native Windows regressions passed. Later integrated totals are recorded in Ticket 05; the final installed `0.0.25` repetition remains owned by Ticket 05, not a blocker for this behavior slice.
- Initial destination harness failure decoded `CF_HDROP` bytes as integer paths; after a DROPFILES self-check and decoding correction, the same real app drag succeeded. Initial shared-WebView startup failed with `0x8007139F`; an owned `WEBVIEW2_USER_DATA_FOLDER` isolated the smoke process and started successfully. Neither failure is counted as product acceptance.
