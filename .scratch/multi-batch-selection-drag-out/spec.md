# DropCove 0.0.25 — Multi-Batch Selection and Drag-Out

Status: ready-for-agent
Design confirmation: approved by the user through `grill-with-docs` on 2026-10-24
Testing seam: confirmed by the user through `to-spec` on 2026-10-24 — one exact installed Windows Resident Instance exercising real Drop Shelf pointer and keyboard selection, expanded Edge Rail pointer selection without activation, collections above the virtualization threshold, and native Copy/cancel/reject into real destinations, supported by the existing frontend interaction/accessibility and Core Shelf State SQLite/filesystem regression seams
Release assignment: 0.0.25
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md#0025--multi-batch-selection-and-drag-out)
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 8.1, 9.3, 11.3–11.4, 14, 15, 19, and 20
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 4, 6, 8, 9, 14, and 15
Relevant architectural decision: [ADR-0006 — Materialize non-filesystem drops as app-owned Managed Files](../../docs/adr/0006-managed-files-and-durable-cleanup.md)
Ticket decomposition: user approved five tracer-bullet tickets with DAG `01 → 02 → {03, 04} → 05` on 2026-10-24: [01 — Drop Shelf pointer Batch Selection Copy round trip](issues/01-drop-shelf-pointer-batch-selection-copy.md), [02 — Occurrence-safe mixed batch-set drag core](issues/02-occurrence-safe-mixed-batch-set-drag.md), [03 — Edge Rail pointer selection and lifecycle parity](issues/03-edge-rail-pointer-selection-lifecycle-parity.md), [04 — Drop Shelf accessible multiselectable grid](issues/04-drop-shelf-accessible-multiselectable-grid.md), and [05 — Release DropCove 0.0.25](issues/05-release-dropcove-0.0.25.md)
Ticket breakdown confirmation: approved by the user through `to-tickets` on 2026-10-24
Implementation authorization: granted by the user for all five approved tickets on 2026-10-24; candidate acceptance, Git commits, and deployment remain separate gates

## Problem Statement

DropCove currently drags one Shelf Item from Manage Items or one Shelf Batch from either Shelf surface. Users who need content from several drops must repeat the same native Copy gesture for every batch, even when those batches form one temporary working set.

The Shelf can now contain external filesystem references and DropCove-owned Managed Files created from text and website images. A multi-batch operation must compose both ownership models without deleting external sources, leaking Managed Files, losing Pinned occurrences, or consuming content after a canceled or rejected destination drag.

Selection also crosses difficult presentation boundaries. The Drop Shelf is a responsive, keyboard-accessible, virtualized grid; the Edge Rail is compact, pointer-only, no-activate, and can hide for fullscreen applications while an OLE drag remains active. The user needs one predictable Explorer-style selection model without persistent state, selection mode UI, checkboxes, focus theft, or a second lifecycle model.

The user needs to select Shelf Batches, see exactly what is selected, drag the selected batches as one deterministic Copy payload, handle Missing and Unavailable references once, and preserve every existing Temporary/Pinned, sizing, virtualization, cleanup, and installation contract.

## Solution

Add surface-local Batch Selection to the Drop Shelf and expanded Edge Rail. Plain click selects one batch, `Ctrl` toggles one batch, and `Shift` replaces the current selection with the contiguous displayed range from the selection anchor. Selection stores stable Shelf Batch identities rather than mounted card instances, so it remains correct above the virtualization threshold.

Dragging a selected card starts one batch-set drag for the complete selection. Dragging an unselected card starts the same batch-set workflow with only that card. A click on an already selected card collapses the selection only when the pointer is released without crossing the drag threshold, allowing the same gesture origin to support both ordinary click and complete-selection drag. Direct item drag remains available only inside Manage Items.

Prepare the ordered selected batch set as one serialized Core Shelf State operation. Reclassify every occurrence, durably remove Missing Items, retain Unavailable Items, flatten Available occurrences in displayed batch and source-item order, deduplicate outbound path strings ordinal case-insensitively, and retain every represented occurrence behind one preparation token. One aggregate confirmation handles mixed Available and Unavailable content. Accepted Copy consumes represented Temporary occurrences and records owned-file cleanup; Pinned occurrences remain.

Expose a compact selected treatment using the existing accent resources and a footer count on both surfaces. The Drop Shelf uses a multiselectable ARIA grid with roving batch focus, arrow/Home/End navigation, `Space` variants, `Ctrl+A`, deterministic focus recovery, and polite coalesced count announcements. The Edge Rail retains pointer-only/no-activate behavior and never adds keyboard activation or a background live region.

## User Stories

1. As a Drop Shelf user, I want to select several Shelf Batches, so that I can copy content from several drops with one drag.
2. As an Edge Rail user, I want the same pointer selection gestures, so that compact access does not require reopening the Drop Shelf.
3. As a user, I want plain click to select one batch, so that ordinary selection remains immediate and familiar.
4. As a user, I want `Ctrl+click` to toggle one batch, so that I can build or reduce a non-contiguous selection.
5. As a user, I want `Shift+click` to replace the selection with one contiguous displayed range, so that range selection is deterministic.
6. As a user, I want range selection to include off-screen virtualized batches, so that scrolling does not change selection meaning.
7. As a user, I want responsive multi-column range order to follow newest-first row-major display order, so that visible layout and selection agree.
8. As a user, I want `Ctrl+Shift` to use the same replacement-range behavior in this release, so that there is no hidden additive-range mode.
9. As a user, I want the most recently clicked, Space-selected, or Ctrl-toggled batch to become the selection anchor, so that later range selection is predictable.
10. As a user, I want toggling an anchor batch off to leave it as the anchor, so that a later Shift gesture still has a clear origin.
11. As a user, I want Shift selection to keep the existing anchor, so that I can adjust the other end of the range repeatedly.
12. As a user, I want Shift with no valid anchor to select only the target and establish a new anchor, so that deleted batches cannot leave broken range behavior.
13. As a user, I want dragging any selected card to drag the complete Batch Selection, so that I do not need a separate drag handle.
14. As a user, I want dragging an unselected card to drag only that batch, so that an old selection does not capture an unrelated gesture.
15. As a user, I want a one-item card to follow the same batch-selection drag rules as a multi-item card, so that card behavior is consistent.
16. As a Manage Items user, I want direct row drag to remain one-item-only, so that precise item transfer is preserved.
17. As a user, I want clicking an already selected card without dragging to collapse selection to that card, so that plain click still means select one.
18. As a user, I want the drag threshold to preserve the pre-press selection when a drag begins, so that selected drag does not collapse accidentally.
19. As a user, I want selected cards to use the existing accent border and subtle accent surface, so that selection is clear without new decoration.
20. As a user, I want both surfaces to show selected batch and item counts, so that I know the scope before dragging.
21. As a user, I want the item count to represent current Shelf Item occurrences, so that it matches what can be consumed or retained.
22. As a user, I want Pin, Unpin, Bulk Pinning, and Manage Items controls to leave selection unchanged, so that actions do not silently retarget my drag.
23. As a user, I want a newly accepted Shelf Batch to remain unselected without clearing existing selection, so that intake and selection are independent.
24. As a user, I want removing an item from a selected batch to update the count while retaining the batch selection, so that surviving work remains selected.
25. As a user, I want a removed or pruned batch to disappear from selection, so that selection never references invisible content.
26. As a user, I want surviving selected batches to remain selected after another selected batch disappears, so that one mutation does not discard the whole set.
27. As a keyboard user, I want batch drag regions on the Drop Shelf to be focusable, so that selection does not require a pointer.
28. As a keyboard user, I want arrow keys to navigate the responsive batch grid, so that focus movement matches the visual layout.
29. As a keyboard user, I want `Home` and `End` to move to the first and last batch, so that large collections remain navigable.
30. As a keyboard user, I want `Space`, `Ctrl+Space`, and `Shift+Space` to mirror pointer selection, so that selection has one model.
31. As a keyboard user, I want `Ctrl+A` within the batch grid to select every batch including virtualized batches without changing the current valid anchor, so that the command is complete without making later Shift behavior surprising.
32. As a screen-reader user, I want the Drop Shelf exposed as a multiselectable grid with per-batch selected state, so that the accessibility tree matches the interaction.
33. As a screen-reader user, I want coalesced polite announcements such as `3 batches, 8 items selected`, so that count changes are understandable without noise.
34. As a keyboard user, I want pointer selection on the Drop Shelf to move focus to the card drag region, so that pointer and keyboard continuation agree.
35. As a keyboard user, I want action-control clicks to retain focus on the activated control, so that selection focus does not override button behavior.
36. As a keyboard user, I want focus to move to the nearest surviving batch when the focused batch disappears, so that focus does not fall to the document body.
37. As a keyboard user, I want focus to move to More when the Shelf becomes empty, so that a usable control remains reachable.
38. As an Edge Rail user, I want selection to remain pointer-only and no-activate, so that my foreground application keeps focus.
39. As an Edge Rail user, I want visible selection counts without a live announcement channel, so that the Rail does not pretend to own keyboard focus.
40. As a user, I want clicking empty collection space to clear selection, so that I can dismiss the working set directly.
41. As a user, I want `Escape` to cancel an active availability confirmation and prepared token before clearing selection, so that prepared work is not left active.
42. As a user, I want Manage Items to close when batch-set preparation starts, so that a popup cannot conflict with the drag scope.
43. As a user, I want later `Escape` presses to close Manage Items, clear selection, then dismiss the Drop Shelf, so that each press performs one predictable step.
44. As a user, I want hiding a surface, switching surfaces, or restarting to clear selection, so that selection is never persistent or cross-surface.
45. As an Edge Rail user, I want fullscreen hiding to clear presentation selection without canceling an already-running native drag, so that fullscreen policy and data safety both hold.
46. As a user, I want rejected and canceled drag-out to retain selection when no clear trigger occurs, so that I can retry.
47. As a user, I want accepted Copy to clear selection, so that consumed content does not leave a stale working set.
48. As a user, I want selected batches flattened in displayed newest-first order and each batch kept in source-item order, so that destination order is deterministic.
49. As a user, I want repeated outbound paths emitted once case-insensitively, so that the destination does not receive duplicate path strings.
50. As a user, I want same filenames at different paths retained, so that filename equality does not discard distinct content.
51. As a user, I want every represented Temporary occurrence consumed after accepted Copy even when paths were deduplicated, so that Shelf lifecycle follows occurrences rather than payload string count.
52. As a user, I want every represented Pinned occurrence retained, so that reusable content survives the operation.
53. As a user, I want ordinary external files and folders never deleted by successful drag-out, so that source safety remains absolute.
54. As a user, I want consumed Temporary Managed Files permanently deleted or recorded for durable retry, so that owned storage does not leak.
55. As a user, I want Missing references removed before drag, so that dead references do not enter the destination payload.
56. As a user, I want Unavailable references retained and excluded, so that temporary storage or permission problems do not destroy references.
57. As a user, I want one aggregate confirmation when Available and Unavailable occurrences are mixed, so that I make one decision for the operation.
58. As a user, I want confirmation counts to use Shelf Item occurrences rather than unique outbound paths, so that counts match lifecycle effects.
59. As a user, I want no confirmation and clear status feedback when no Available occurrence remains, so that an empty drag never starts.
60. As a user, I want Missing cleanup to remain committed if I cancel the later confirmation or destination drag, so that confirmed dead references do not reappear.
61. As a user, I want a prepared token bound to the exact selected batch set and Shelf snapshot, so that the eventual drag cannot silently use stale content.
62. As a user, I want dragging the confirmation button to start the exact prepared operation immediately, so that mixed availability still supports one gesture.
63. As a user, I want changing selection or Shelf state after confirming to invalidate the old token and reprepare, so that visible selection and payload cannot diverge.
64. As a user, I want persistence failure during preparation or completion to leave visible and durable state unchanged apart from already committed Missing cleanup, so that failures do not partially consume content.
65. As a user, I want final-item removal to preserve the established automatic sizing reset and Hidden/EdgeDocked transitions, so that multi-batch drag composes with window behavior.
66. As a user, I want collections above 50 batches to keep stable selection, focus, counts, and range behavior while scrolling, so that virtualization remains invisible.
67. As a user, I want Dark, Light, and Windows Contrast presentation to show selected and focused states distinctly, so that appearance does not hide selection.
68. As a release reviewer, I want real installed pointer, keyboard, accessibility, Rail no-activate, and native destination evidence, so that automated tests are not mistaken for product acceptance.
69. As a release owner, I want version metadata to remain `0.0.24` during behavior work and become `0.0.25` only at the release gate, so that sequencing remains auditable.
70. As a project owner, I want specification, ticket approval, implementation authorization, candidate acceptance, and Git commit authorization to remain separate decisions, so that planning cannot imply deployment or commit approval.

## Implementation Decisions

- Scope this specification to roadmap version `0.0.25`: surface-local Batch Selection, deterministic selected batch-set drag-out, Drop Shelf keyboard/accessibility behavior, Edge Rail pointer parity, and one installed Windows release candidate.
- Reuse the canonical `Batch Selection` domain term. No new glossary term or ADR is introduced; the approved choices are presentation and drag-contract details within the existing architecture.
- Store selection outside mounted virtual rows as stable Shelf Batch identities plus an optional selection anchor and focused batch identity.
- Keep one independent selection owner per Shelf surface. Selection is session-only, is never persisted, and is never transferred between Drop Shelf and Edge Rail.
- Define range order as the complete displayed newest-first row-major batch order. Virtualization and viewport position do not limit the range.
- Plain click and `Space` select only the target and set the anchor. `Ctrl+click` and `Ctrl+Space` toggle the target and set the anchor even when toggling it off.
- `Shift+click` and `Shift+Space` replace selection with the inclusive anchor-to-target range and do not move a valid anchor. With no valid anchor they select the target and establish it as anchor.
- Treat `Ctrl+Shift` as ordinary Shift range replacement in `0.0.25`; do not add additive range selection.
- `Ctrl+A` selects the complete collection without changing a valid anchor. If no anchor exists, it remains absent, so the next Shift gesture selects only its target and establishes a new anchor.
- Delay plain-click collapse of an already selected card until pointer release below the established drag threshold. Crossing the threshold uses the pre-press selection.
- Route every card drag, including a one-item card, through the batch-set drag path. Preserve the distinct one-item drag path only for Manage Items rows.
- Keep action controls isolated from card selection. Pinning and management do not alter selection; removal and Missing pruning remove only identities that no longer exist.
- Preserve surviving selected identities and recalculate selected item occurrence counts after every Shelf snapshot change. Clear the anchor when its batch disappears; do not choose a replacement anchor automatically.
- Do not auto-select newly accepted batches and do not clear existing selection merely because drop-in, Pin state, or item counts changed.
- Use the established accent border plus accent-subtle surface for selected cards, with Windows system Highlight resources in Contrast Themes. Do not add checkboxes, badges, a selection toolbar, or a selection mode.
- Show `N batches · M items` in the existing footer budget on both surfaces. `M` counts current Shelf Item occurrences in selected batches before availability preparation.
- Expose the Drop Shelf batch collection as a multiselectable ARIA grid because cards contain independently operable controls. Expose per-batch selected state on the selectable row.
- Use roving focus for batch drag regions. Arrow keys follow responsive row-major geometry; `Home` and `End` move to collection boundaries; selection keys operate only when the batch grid owns focus.
- Keep card action controls as native tabbable controls with their established names and pressed/checked semantics. Selection focus never replaces button focus after activation.
- When a focused batch disappears, focus the batch now occupying the same displayed index, otherwise the previous batch, otherwise the More action when the Shelf is empty.
- Emit one coalesced polite Drop Shelf announcement after each selection change. The Edge Rail updates its visible footer count only and adds no live region or keyboard model.
- Starting batch-set preparation closes Manage Items. An active availability confirmation owns `Escape`; canceling it clears the prepared token but retains selection.
- Surface-level `Escape` ordering is availability confirmation, Manage Items, Batch Selection, then Dismiss Drop Shelf on a later press. Existing Utility Menu and Clear confirmation cancellation remain authoritative while open.
- Clear selection on empty-space click, accepted Copy, surface hide, surface switch, and restart. A hide or switch remains a clear trigger even if a native drag is active; the prepared participants continue independently and selection is not restored after rejection.
- Retain selection after rejected, canceled, or pre-acceptance-failed drag only when no clear trigger occurred.
- Replace the single-batch preparation contract with one batch-set preparation contract that accepts ordered selected batch identifiers. A singleton set supplies existing single-card behavior without a compatibility path.
- Serialize preparation through the existing Core Shelf State mutation gate. Reclassify every selected occurrence before creating the outbound payload.
- Durably remove Missing Items and prune empty batches in one transaction before drag begins. Queue Managed File cleanup through the existing ownership-derived durable deletion path.
- Retain Unavailable Items and exclude them from the current drag. If no Available occurrence remains, create no token and return status feedback.
- Record every Available occurrence behind the token, including repeated case-insensitive paths and both Temporary and Pinned occurrences.
- Build outbound paths in displayed selected-batch order and source-item order, deduplicating path strings with Windows ordinal case-insensitive comparison. Same names at different paths remain distinct.
- Report `available` and `unavailable` confirmation counts as Shelf Item occurrences, not deduplicated outbound path counts.
- Bind an armed prepared drag to the exact ordered selected batch identifiers and authoritative Shelf snapshot that produced it. Any selection or Shelf mutation invalidates the armed token and requires cancellation/repreparation.
- Preserve direct drag from the existing confirmation primary button. Clicking the primary button may arm a second card drag only while the binding remains valid.
- Completion consumes represented Temporary occurrences only after accepted Windows `Copy`. Current Pin state remains authoritative at completion; represented Pinned occurrences survive.
- Completion failure must not partially consume occurrences. Existing transaction rollback, pending Managed File deletion, retry, sizing, and warning behavior remain authoritative.
- Keep Missing cleanup committed when a later confirmation or destination drag is canceled or rejected.
- Preserve Copy-only OLE behavior, destination outcome interpretation, cross-integrity limitation, Manage Items item drag, application-owned drag rejection, and foreground-focus behavior.
- Preserve Drop Shelf sizing, Edge Rail expansion/collapse, fullscreen, Shake, Settings, About, clear confirmation, Shell Visual, appearance, and reduced-motion contracts.
- Keep Edge Rail pointer-only/no-activate. Do not add focusable selection controls, keyboard activation, or keyboard-only native OLE drag.
- Keep package, crate, lockfile, Tauri, PE, About, installer, and uninstall metadata at `0.0.24` during behavior implementation. Synchronize to `0.0.25` only in the approved release ticket.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Primary seam: one exact installed Windows x64 Resident Instance using embedded production assets, real Drop Shelf and expanded Edge Rail surfaces, real pointer modifiers and keyboard input, an owned test Profile Data set, and real native Copy/cancel/reject destinations.
- Supporting frontend coverage uses the existing Vue interaction and virtualization seams to exercise selection state, pointer threshold behavior, keyboard commands, focus recovery, ARIA state, footer counts, and polite announcements through observable DOM behavior.
- Supporting native coverage uses the existing Core Shelf State seam with real temporary SQLite databases and real temporary directories to exercise batch-set preparation, Missing cleanup, deduplication, occurrence completion, rollback, Managed File cleanup, restart, and sizing transitions.
- Prefer behavior tests over implementation details. Good tests fail for wrong selected identities, wrong range, stale payload, wrong path order, wrong occurrence consumption, lost focus, invalid accessibility state, external-source deletion, or leaked Managed Files.
- Do not test CSS class strings, private helper calls, SQL text, copied command forwarding, exact internal collection types, or mock echoes.
- Add frontend coverage for plain click, Ctrl toggle, Shift replacement range, no-valid-anchor fallback, anchor after toggle-off, and non-additive Ctrl+Shift behavior.
- Add pointer threshold coverage proving release below threshold collapses selection while movement beyond threshold drags the pre-press selection.
- Add one-item and multi-item card coverage proving selected cards use the complete batch set and unselected cards use a singleton batch set.
- Add action-isolation coverage for Pin, Bulk Pinning, Manage Items, Remove Item, Remove Batch, accepted drop-in, and Missing pruning.
- Add count coverage for selected batches and Shelf Item occurrences as items are removed, batches are pruned, or new batches are accepted.
- Add virtualization coverage above 50 batches for off-screen range selection, stable selected identities, focus pinning, scroll, and mounted-row churn.
- Add responsive-grid keyboard coverage for roving focus, Left/Right, Up/Down using current column count, Home/End, Space variants, Ctrl+A, and Escape ordering.
- Add accessibility-tree coverage for multiselectable grid state, per-batch `aria-selected`, native action-control semantics, visible focus, and coalesced polite count announcements.
- Add focus-recovery coverage for removal at the beginning, middle, and end of the collection, final-batch removal, and virtualization boundaries.
- Add visual-state checks in Dark, Light, and Windows Contrast for selected, focused, hovered, disabled, and ordinary cards without restoring the removed resize-hover border behavior.
- Add Edge Rail coverage for click/Ctrl/Shift selection, selected footer counts, action isolation, no focus acquisition, no live announcement, and no activation theft.
- Add surface lifecycle coverage for empty-space clear, selection Escape, hide/reopen, Rail-to-Shelf and Shelf-to-Rail switch, restart, accepted Copy clear, canceled/rejected retention, and hide during active drag.
- Add prepared-binding coverage proving selection or Shelf mutation after confirmation invalidates the old token and reprepares the visible set.
- Add confirmation coverage for direct drag, click-to-arm then valid second drag, Cancel, Escape, Available/Unavailable occurrence counts, and no-Available status.
- Add native batch-set ordering coverage for selected batches chosen out of order, displayed newest-first flattening, and source-item order within each batch.
- Add ordinal case-insensitive deduplication coverage for repeated exact paths, case variants, same filename at different paths, and duplicates across single- and multi-item batches.
- Add occurrence-completion coverage for duplicated paths with Temporary and Pinned occurrences. Accepted Copy consumes every represented Temporary occurrence and retains every Pinned occurrence.
- Add mixed ownership coverage for external references, text Managed Files, and website-image Managed Files in one selected drag. External sources remain untouched.
- Add availability coverage for Available, Missing, and Unavailable occurrences across several batches, including empty-batch pruning, all-Unavailable, all-Missing, and mixed aggregate confirmation.
- Add rollback coverage for persistence failure during Missing cleanup, token preparation, accepted-Copy consumption, and pending Managed File deletion recording. No partial visible or durable consumption is permitted.
- Reuse existing sharing-violation cleanup tests to prove consumed Managed Files enter durable pending deletion and are retried without restoring Shelf Items.
- Reuse existing Pin-race behavior so current Pin state at completion wins without consuming newly Pinned occurrences.
- Add final-item coverage for automatic sizing reset, manual-height reset, Hidden/EdgeDocked transitions, and footer/count disappearance.
- Exercise real installed collections below and above the virtualization threshold on both surfaces with mouse and precision touchpad pointer input where available.
- Exercise real Drop Shelf keyboard selection with a screen reader or Windows accessibility inspection tool. Record the multiselectable grid, selected state, focus order, and live announcements as observed evidence.
- Exercise real Edge Rail selection while another application owns foreground focus. Confirm click/Ctrl/Shift and native drag never activate DropCove.
- Exercise real native Copy into at least one filesystem destination and one accepting application where practical, plus Escape cancellation and destination rejection.
- Exercise repeated paths, same filenames at different paths, Missing local files, an unavailable volume/share, Temporary and Pinned external references, text Managed Files, and website-image Managed Files.
- Run frontend regressions, frontend typecheck/production build, native Rust regressions, cargo-xwin Windows release/test compilation, and the freshly produced Windows regression executable.
- After behavior and ticket acceptance, synchronize version `0.0.25`, build one exact current-user Windows x64 NSIS candidate through cargo-xwin, run installed embedded assets with the development server unavailable, and verify restart, reinstall/update, standard uninstall, and Profile Data preservation.
- Record exact installer/executable hashes, test counts, Windows/accessibility tooling, destination applications, observed focus owner, candidate version surfaces, and unobserved physical limitations. Never infer installed behavior from automated tests alone.

## Out of Scope

- Persistent Batch Selection, restoration after restart, or sharing selection between Drop Shelf and Edge Rail.
- Named selections, selection history, saved working sets, or a selection clipboard.
- Additive `Ctrl+Shift` ranges, subtractive ranges, lasso selection, marquee selection, or selection inversion.
- A selection toolbar, selection mode, always-visible checkboxes, count badges on every card, or new card geometry.
- Arbitrary Shelf Item selection across batches. Multi-selection remains Shelf Batch-based.
- Batch reordering, item reordering, drag-to-reorder, or changing newest-first display order.
- Keyboard activation of the Edge Rail, Edge Rail roving focus, or an Edge Rail live announcement channel.
- Keyboard-only native OLE destination drag. Keyboard support covers selection and ordinary controls only.
- Move, Link, cut, or destination negotiation beyond the existing Copy-only contract.
- Persistent availability state, filesystem watchers, retry loops, background polling, or automatic unavailable-volume reconnection.
- Content hashing, byte deduplication, shared Managed File backing, or destination-side duplicate handling.
- Changing Website Image Virtual File intake, plain-text intake, external-path intake, Attachment Services, Managed File ownership, or deletion authority.
- Changing Drop Shelf/Edge Rail geometry, the removed resize-hover border behavior, Clear confirmation dimensions, appearance modes, Shake thresholds, or monitor-placement policy.
- macOS, Linux, non-WebView Windows UI, or any non-Windows drag adapter.
- Beginning implementation, synchronizing release metadata, packaging a candidate, accepting a release, or creating a Git commit as part of this specification step.

## Further Notes

The central invariant is occurrence-safe deduplication: the Windows payload may contain one path string while the prepared token records several selected Shelf Item occurrences. Payload deduplication must never collapse lifecycle responsibility.

The selection model is deliberately presentation-only. It stores stable identities so virtualization is invisible, but it never becomes durable Shelf state and never changes Pin or ownership state by itself.

The ARIA grid choice follows the cards' existing interactive controls. A listbox option cannot safely contain independently operable Pin, Manage, and Remove controls; the grid model supplies valid multiselectable semantics without adding selection checkboxes.

This specification is published as `ready-for-agent` after explicit design, testing-seam, ticket-breakdown, and implementation authorization. The approved ticket DAG may now execute. Candidate acceptance, Git commits, and deployment still require separate explicit user authorization.
