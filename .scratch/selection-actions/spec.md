# DropCove 0.0.28 — Selection Actions

Status: ready-for-agent
Design confirmation: approved by the user through `grill-with-docs` on 2026-10-27
Testing seam: confirmed by the user through `to-spec` on 2026-10-27 — one exact installed Windows Resident Instance exercising Selection Actions on Drop Shelf, expanded Edge Rail, and Manage Items from both origins with real secondary-click, keyboard menu invocation, focus/foreground observation, confirmation, and Managed File cleanup, supported by the existing mounted Vue interaction/accessibility seams and Core Shelf State temporary SQLite/filesystem regression seam
Release assignment: 0.0.28
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md#0028--selection-actions)
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 8, 9, 10, 13, 19, and 20
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 2.2, 4, 6, 14, and 16
Relevant architectural decision: [ADR-0006 — Materialize non-filesystem drops as app-owned Managed Files](../../docs/adr/0006-managed-files-and-durable-cleanup.md)
Ticket decomposition: user approved six tracer-bullet tickets with DAG `01 → {02, 03} → 04 → 05 → 06` through `to-tickets` on 2026-10-27: [01 — Drop Shelf Batch Selection Pin/Unpin round trip](issues/01-drop-shelf-batch-selection-pin-unpin.md), [02 — Drop Shelf ownership-safe Remove Selected](issues/02-drop-shelf-remove-selected.md), [03 — Manage Items Item Selection Pin/Unpin](issues/03-manage-items-item-selection-pin-unpin.md), [04 — Manage Items ownership-safe Remove Selected](issues/04-manage-items-remove-selected.md), [05 — Edge Rail and Rail-origin Selection Actions parity](issues/05-edge-rail-selection-actions-parity.md), and [06 — Release DropCove 0.0.28](issues/06-release-dropcove-0.0.28.md).
Ticket breakdown confirmation: approved by the user through `to-tickets` on 2026-10-27.
External delivery gate: Ticket 01 has no numbered Selection Actions blocker, but implementation remains blocked by `0.0.27` final acceptance as recorded in the canonical Roadmap.
Implementation authorization: separate gate after `to-tickets`; 0.0.27 final acceptance is also required before 0.0.28 implementation begins

## Problem Statement

DropCove already lets users build a **Batch Selection** on the Drop Shelf or expanded Edge Rail and an **Item Selection** inside Manage Items. Those selections can be dragged as one Copy operation, but lifecycle actions still operate through one card, one row, or one complete batch at a time. Pinning, unpinning, or removing several selected targets therefore requires repeated actions and makes the visible selection less useful than users expect from a Windows-style multiselectable surface.

The current secondary-click behavior is also incomplete. Manage Items consumes a row secondary-click only to suppress the WebView browser menu, while Drop Shelf and Edge Rail cards have no selection-aware secondary-click action. Without an explicit targeting rule, adding lifecycle actions could apply to a stale selection when the user secondary-clicks an unselected card or row.

Removal has two ownership outcomes that must remain clear and safe. Removing an **External Reference** removes only the Shelf reference and must never modify its source object. Removing a Shelf Item backed by a **Managed File** ends DropCove's ownership lifecycle and may permanently delete app-owned bytes or record durable retry when Windows still holds the file. A multi-selection operation must preserve this boundary, reject stale targets, avoid partial transactions, and explain the actual ownership effects before destructive work begins.

The feature spans three presentation models. Drop Shelf and Drop-Shelf-origin Manage Items can own keyboard focus; Edge Rail and Rail-origin Manage Items are pointer-only/no-activate and must preserve the foreground application. Selection Actions must add consistent menu, confirmation, focus, and Escape behavior without changing the existing Item Actions Menu, drag-out workflows, virtualization, Managed File privacy, or selection ownership.

## Solution

Add a distinct **Selection Actions Menu** to Batch Selection and Item Selection surfaces. Secondary-clicking a selected member preserves the current selection and targets the complete selection. Secondary-clicking an unselected member replaces the selection with that member before the menu opens. Secondary-button modifiers do not extend or toggle selection, and independent action controls never retarget selection.

On Drop Shelf and expanded Edge Rail, the menu operates on Batch Selection. One selected single-item batch exposes **Pin Item** or **Unpin Item** plus **Remove Batch**. One selected multi-item batch exposes **Pin All Items** or **Unpin All Items** plus **Remove Batch**. Two or more selected batches expose **Pin Selected** or **Unpin Selected** plus **Remove Selected**. Pinning applies to every Shelf Item in the selected batches; removal removes the selected Shelf Batches.

In Manage Items, the menu operates on Item Selection within the active Shelf Batch. One selected item exposes **Pin Item** or **Unpin Item** plus **Remove Item**. Two or more selected items expose **Pin Selected** or **Unpin Selected** plus **Remove Selected**. No cross-batch Item Selection is introduced.

Pin and Unpin use the established tri-state rule: an all-Pinned target offers Unpin; a None or Mixed target offers Pin. Multi-target mutation is atomic and retains selection on success. Singleton Remove remains immediate. Remove Selected always requires ownership-aware confirmation, validates an immutable authoritative target snapshot, blocks competing mutations, and commits reference removal plus any Pending Managed File Deletion responsibility atomically.

Focus-capable surfaces support secondary-click, the Context Menu key, and `Shift+F10`, with standard menu navigation and focus restoration. Edge Rail and Rail-origin Manage Items remain pointer-only/no-activate, hold their transient presentation open while the menu or confirmation is visible, and never steal foreground focus.

## User Stories

1. As a Drop Shelf user, I want to pin several selected Shelf Batches at once, so that I can retain a working set without repeating the same action on every card.
2. As a Drop Shelf user, I want to unpin several selected Shelf Batches at once, so that I can make the complete working set consumable again.
3. As a Drop Shelf user, I want to remove several selected Shelf Batches at once, so that I can clear completed work efficiently.
4. As an Edge Rail user, I want the same selection-scoped Pin, Unpin, and Remove actions as the Drop Shelf, so that compact access does not require reopening the Drop Shelf.
5. As a Manage Items user, I want to pin several selected Shelf Items at once, so that I can retain only the chosen items from one batch.
6. As a Manage Items user, I want to unpin several selected Shelf Items at once, so that I can make only the chosen items Temporary.
7. As a Manage Items user, I want to remove several selected Shelf Items at once, so that I can prune one batch without repeating per-row removal.
8. As a user, I want Batch Selection to remain the action domain on Drop Shelf and Edge Rail, so that a selected multi-item card is never reinterpreted as an implicit cross-batch Item Selection.
9. As a user, I want Item Selection to remain scoped to one active Manage Items batch, so that item actions never cross popup or batch boundaries.
10. As a user, I want secondary-clicking a selected member to preserve the complete selection, so that the menu applies to the working set I already built.
11. As a user, I want secondary-clicking an unselected member to replace the old selection with that member, so that an unrelated stale selection cannot receive the action.
12. As a user, I want `Ctrl` or `Shift` held during secondary-click to be ignored for selection changes, so that primary-button selection gestures remain the only modifier-selection model.
13. As a user, I want secondary-click on Pin, More, Manage, or Remove controls to avoid opening the Selection Actions Menu, so that independent controls remain isolated.
14. As a user, I want the Selection Actions Menu to remain distinct from the Item Actions Menu, so that lifecycle actions cannot be confused with Show in File Explorer, Copy Path, or Copy Text.
15. As a user, I want one selected Shelf Batch to remain one selection unit even when it contains many Shelf Items, so that labels and confirmation rules match the visible selection domain.
16. As a user of a selected one-item batch, I want the menu to say Pin Item or Unpin Item, so that the lifecycle target is explicit.
17. As a user of a selected multi-item batch, I want the menu to say Pin All Items or Unpin All Items, so that I know every item in that batch will change.
18. As a user of one selected batch, I want removal labeled Remove Batch, so that the action is not confused with removing one item from the batch.
19. As a user of several selected batches, I want the menu to say Pin Selected, Unpin Selected, and Remove Selected, so that the menu communicates selection-wide scope.
20. As a Manage Items user with one selected row, I want Pin Item, Unpin Item, and Remove Item labels, so that singular scope is unambiguous.
21. As a Manage Items user with several selected rows, I want Pin Selected, Unpin Selected, and Remove Selected labels, so that plural scope is unambiguous.
22. As a user with a None-Pinned target, I want activation to Pin every target item, so that the result is deterministic.
23. As a user with a Mixed target, I want activation to Pin every target item, so that the action matches existing Bulk Pinning behavior.
24. As a user with an all-Pinned target, I want activation to Unpin every target item, so that one action returns the set to Temporary.
25. As a user, I want selection-scoped Pinning to include Available, Missing, and Unavailable retained references, so that availability does not silently change lifecycle scope.
26. As a user, I want Pin or Unpin success to retain my selection, so that I can inspect or perform another action on the same working set.
27. As a user, I want Pin or Unpin failure to preserve both selection and prior Pin states, so that no partial lifecycle change is visible.
28. As a user, I want the complete selected Pin or Unpin operation committed atomically, so that several cards or rows cannot end in a partially changed state.
29. As a user, I want singleton Remove to remain immediate, so that the existing single-target workflow does not gain an unnecessary confirmation.
30. As a user, I want Remove Selected to require confirmation whenever several selection units are targeted, so that accidental bulk removal has a clear safety boundary.
31. As a Drop Shelf user, I want bulk-removal confirmation to show selected batch and affected item counts, so that I understand the complete scope.
32. As an Edge Rail user, I want the same batch and item counts in confirmation, so that compact presentation does not hide destructive scope.
33. As a Manage Items user, I want confirmation to show the selected item count, so that the number matches the rows I chose.
34. As a user removing only External References, I want confirmation to state that source files and folders will not be deleted, so that reference removal is not mistaken for filesystem deletion.
35. As a user removing mixed External References and Managed Files, I want confirmation to distinguish safe external sources from permanently deleted DropCove-created files, so that both ownership outcomes are clear.
36. As a user removing only Managed Files, I want confirmation to state how many DropCove-created files will be permanently deleted, so that irreversible cleanup is explicit.
37. As a user, I want removal of External References to leave source bytes, names, locations, and metadata unchanged, so that DropCove never exceeds reference authority.
38. As a user, I want selected Managed Files permanently deleted when possible, so that removed DropCove-owned content does not leak storage.
39. As a user, I want a locked Managed File recorded for durable pending deletion, so that successful Shelf removal does not lose cleanup responsibility.
40. As a user, I want Cancel or `Escape` in removal confirmation to retain selection and Shelf state, so that reviewing the warning has no side effect.
41. As a user, I want a failed multi-remove to retain selection and every target, so that persistence failure cannot produce partial removal.
42. As a user, I want successful removal to prune only identities that disappeared, so that any surviving selection outside the target remains valid.
43. As a Manage Items user, I want the popup to remain open when its batch still contains items, so that partial item removal does not interrupt management.
44. As a Manage Items user, I want the popup to close when removal empties its batch, so that an invalid empty management surface does not remain.
45. As a user, I want confirmation bound to the exact selected identities and authoritative Shelf state, so that stale counts or targets cannot be confirmed later.
46. As a user, I want stale or missing targets to reject the complete operation, so that bulk actions never silently shrink their scope.
47. As a user, I want selection, drag, Pin, Remove, Clear, and competing mutations blocked while removal confirmation is open, so that the reviewed target cannot change underneath me.
48. As a Drop Shelf keyboard user, I want `Shift+F10` and the Context Menu key to open Selection Actions for the focused batch, so that the menu does not require a pointer.
49. As a Manage Items keyboard user, I want the same keyboard invocation for the focused row, so that selection lifecycle actions remain accessible.
50. As a keyboard user, I want keyboard invocation on an unselected focused member to select only that member before opening, so that keyboard and pointer targeting agree.
51. As a keyboard user, I want opening the menu to focus its first item and support Arrow, Home, End, Enter, and Space, so that it follows the established menu model.
52. As a keyboard user, I want `Escape` to close an open Selection Actions Menu and restore focus to its invoking card or row, so that dismissal is predictable.
53. As a keyboard user, I want `Escape` during removal confirmation to cancel confirmation and retain selection, so that destructive intent requires explicit confirmation.
54. As a keyboard user, I want later `Escape` presses to resume the existing selection and surface lifecycle, so that the new transient states compose with current ordering.
55. As a keyboard user, I want Tab to close the menu and resume the established Tab order, so that the menu does not create a second focus trap.
56. As an Edge Rail user, I want Selection Actions to remain pointer-only and no-activate, so that the application I am working in keeps foreground focus.
57. As a Manage Items user opened from Edge Rail, I want its menu and confirmation to remain pointer-only/no-activate, so that item management does not steal focus.
58. As an Edge Rail user, I want the Rail and Rail-origin popup held open while a Selection Actions transient is visible, so that the target does not disappear during interaction.
59. As a user with a virtualized collection, I want off-screen selected identities to remain in the action target, so that viewport mounting does not change scope.
60. As a user, I want projection changes or target disappearance to close stale menus, so that a visible menu never points at obsolete state.
61. As a user, I want selection-scoped mutation to invalidate any armed drag prepared against older Shelf state, so that later Copy cannot use a stale snapshot.
62. As a user, I want existing Batch Selection and Item Selection drag-out behavior unchanged, so that adding lifecycle actions does not regress Copy workflows.
63. As a user, I want the existing Item Actions Menu capability matrix unchanged, so that selection actions do not expose Managed File paths or unsupported actions.
64. As a Managed File user, I want Profile Data paths absent from menu, confirmation, status, and accessibility text, so that internal storage remains private.
65. As a screen-reader user, I want menu items, confirmations, selected states, counts, and destructive consequences named explicitly, so that the interaction is understandable without visual context.
66. As a Windows Contrast user, I want selected, focused, menu, confirmation, and destructive states to use system colors and visible borders, so that the feature remains operable in Contrast Themes.
67. As a user, I want Dark and Light presentation to reuse existing semantic menu and danger resources, so that Selection Actions look native to DropCove.
68. As a user, I want no new toolbar, checkbox, persistent selection mode, or footer action strip, so that the existing compact surfaces remain unchanged until I invoke the menu.
69. As a user, I want Batch Selection and Item Selection to remain transient and non-persistent after restart or surface replacement, so that lifecycle actions do not turn selection into durable Shelf state.
70. As a user, I want no cross-surface Batch Selection or cross-batch Item Selection, so that action ownership remains local and predictable.
71. As a release reviewer, I want one exact installed Windows candidate to prove right-click, keyboard, confirmation, cleanup, and no-activate behavior, so that unit tests are not mistaken for product acceptance.
72. As a project owner, I want `to-spec`, `to-tickets`, implementation authorization, candidate acceptance, Git commit authorization, and deployment to remain separate gates, so that planning cannot imply delivery approval.

## Implementation Decisions

### Release and domain boundary

- The complete feature ships as DropCove `0.0.28` after `0.0.27` receives final acceptance. No one-surface intermediate release is permitted.
- **Selection Actions Menu**, **Batch Selection**, and **Item Selection** are the canonical domain terms.
- Drop Shelf and expanded Edge Rail operate on Batch Selection. Manage Items operates on Item Selection within one active Shelf Batch.
- Selection remains disposable presentation state. It is never persisted, shared between Shelf surfaces, transferred between Manage Items popups, or made authoritative for ownership.
- No persistence schema change, new setting, new dependency, or new ADR is required.

### Menu ownership and targeting

- Add one shared Selection Actions Menu behavior that surfaces can host without merging it with the Item Actions Menu.
- The Item Actions Menu remains available only through More and retains its ownership-aware Show in File Explorer, Copy Path, and Copy Text matrix.
- Secondary-click on a selected card or row preserves the current selection. Secondary-click on an unselected card or row replaces selection with that member before opening the menu.
- `Ctrl`, `Shift`, and `Ctrl+Shift` do not extend, toggle, or range-select during secondary-button invocation.
- Only the card or row selection region opens Selection Actions. Secondary-clicks on Pin, More, Manage, Remove, confirmation controls, inputs, or textareas do not retarget selection or open either action menu.
- A consumed secondary-click never permits the embedded WebView browser menu to appear on Selection Actions boundaries.
- Pointer invocation anchors and clamps the menu at the invocation point. Keyboard invocation anchors and clamps it to the focused card or row.
- At most one Selection Actions Menu is open on a surface. Opening it closes an Item Actions Menu, and opening an Item Actions Menu closes it.
- Projection replacement, target disappearance, virtualization unmount, scroll, drag, resize, movement, confirmation ownership, busy mutation, surface hide/close, Rail collapse, fullscreen transition, or display transition closes the menu.
- The menu contains only the applicable Pin or Unpin action followed by the applicable Remove action. Remove uses the established danger semantics.

### Labels and selection cardinality

- Cardinality is measured in the owning selection domain, not by affected Shelf Item count.
- One selected one-item Shelf Batch exposes **Pin Item** or **Unpin Item**, followed by **Remove Batch**.
- One selected multi-item Shelf Batch exposes **Pin All Items** or **Unpin All Items**, followed by **Remove Batch**.
- Two or more selected Shelf Batches expose **Pin Selected** or **Unpin Selected**, followed by **Remove Selected**.
- One selected Manage Items row exposes **Pin Item** or **Unpin Item**, followed by **Remove Item**.
- Two or more selected Manage Items rows expose **Pin Selected** or **Unpin Selected**, followed by **Remove Selected**.
- All labels and accessible names identify the current selection unit and never expose a Managed File internal path.

### Pin and Unpin semantics

- Derive selected Pin state from the current authoritative projection: All Pinned, None Pinned, or Mixed.
- All Pinned offers Unpin. None or Mixed offers Pin, matching existing Bulk Pinning semantics.
- Batch-selection Pin or Unpin changes every Shelf Item in every selected Shelf Batch, including retained Missing and Unavailable references.
- Item-selection Pin or Unpin changes exactly the selected Shelf Item identities from the active Manage Items batch.
- Multi-target Pin and Unpin validate the complete non-empty unique identity set before changing any item and commit one durable transaction.
- Unknown, duplicate, stale, or wrong-domain identities reject the complete operation.
- Successful Pin or Unpin publishes one replacement Shelf snapshot and retains presentation selection.
- Persistence failure or validation failure publishes no partial state, preserves prior Pin states, and retains selection.
- Any successful lifecycle mutation invalidates prepared drag state bound to an older Shelf snapshot.

### Remove semantics and ownership safety

- One selected batch continues through existing immediate Remove Batch behavior. One selected Manage Items row continues through existing immediate Remove Item behavior.
- Two or more selection units always require confirmation, including all-External selections.
- Batch confirmation identifies `N batches · M items`. Manage Items confirmation identifies `N items`.
- An all-External confirmation states that source files and folders will not be deleted.
- A mixed-ownership confirmation states that External sources stay and reports how many DropCove-created files will be permanently deleted.
- An all-Managed confirmation reports how many DropCove-created files will be permanently deleted.
- Focus-capable confirmation actions are **Cancel** and **Remove**, with Cancel initially focused. Pointer-only confirmations expose the same actions without acquiring focus.
- Confirmation request resolves authoritative selection-unit count, item count, ownership count, and exact identities. The caller does not supply trusted counts or ownership classification.
- Confirmation is bound to an immutable authoritative target snapshot. Confirm accepts only that prepared target; changed, missing, reordered-domain, or otherwise stale membership rejects the complete operation.
- Batch-selection removal deletes the selected Shelf Batches and every contained Shelf Item. Item-selection removal deletes exactly the selected Shelf Items and prunes the active Shelf Batch if empty.
- External Reference removal never deletes, moves, renames, overwrites, or modifies its source object.
- Managed File-backed removal records Pending Managed File Deletion responsibility atomically with reference removal, then attempts permanent deletion.
- Sharing, antivirus, or filesystem deletion failure leaves the Shelf mutation committed, reports pending cleanup, and uses existing retry behavior without restoring removed Shelf Items.
- Persistence failure before commit leaves visible and durable Shelf state, cleanup authority, and owned bytes unchanged.
- Cancel, confirmation setup failure, stale-target failure, and persistence failure retain selection.
- Successful removal prunes disappeared selected identities. Manage Items remains open while its active Shelf Batch exists and closes when the batch disappears.

### Authoritative Core Shelf State contracts

- Preserve separate identity domains rather than routing Shelf Item identities through a batch API or Shelf Batch identities through an item API.
- Add one batch-selection mutation contract accepting a finite action and a non-empty unique set of Shelf Batch identities.
- Add one item-selection mutation contract accepting the active Shelf Batch identity, a finite action, and a non-empty unique set of Shelf Item identities all belonging to that batch.
- The finite mutation action is Pin, Unpin, or Remove. Presentation labels are not authoritative action values.
- Singleton presentation paths may continue to use existing one-item or one-batch mutations; multi-target behavior must use the atomic selection contracts rather than loop over singleton commands.
- Mutations serialize through the existing Core Shelf State gate, validate all targets before writes, and publish exactly one resulting snapshot after success.
- Selection state is not sent to persistence. Only exact target identities and requested lifecycle action cross the presentation-to-core boundary.
- The removal-confirmation preparation and confirmation lifecycle is distinct from drag preparation and Item Actions. It cannot reuse a prepared drag token as removal authority.
- Existing Pending Managed File Deletion, final-item sizing reset, warning, restart, and cleanup-retry invariants remain authoritative.

### Confirmation and competing ownership

- Opening Remove Selected closes the Selection Actions Menu before confirmation appears.
- While confirmation is open, its originating surface blocks selection changes, native drag start, Pin, Unpin, Remove, Clear Temporary Items, accepted drop-in mutation, window movement, resize, and other competing Shelf mutations.
- Drop Shelf and Drop-Shelf-origin Manage Items present focus-capable modal confirmation within the owning surface and restore focus to the invoking card or row after Cancel or non-foreground failure.
- Edge Rail and Rail-origin Manage Items present inline pointer confirmation, acquire the required transient hold, and never activate DropCove or claim keyboard focus.
- Rail pointer leave does not collapse an owned Selection Actions Menu or confirmation; dismissal releases the hold and resumes ordinary delayed collapse.
- Fullscreen, owner destruction, cooperative shutdown, and Exit retain higher teardown authority. A forced surface transition cancels the transient without mutation and does not restore discarded surface state later.
- Repeated removal requests while confirmation is open do not replace or extend the immutable target.

### Keyboard, focus, and accessibility

- On focus-capable Drop Shelf and Manage Items origins, the Context Menu key and `Shift+F10` open Selection Actions for the focused card or row.
- Keyboard invocation preserves selection when the focused member is selected and replaces selection with the focused member when it is not.
- Opening the menu focuses the first menu item and follows the established menu contract: Arrow Up/Down wrapping, Home/End, Enter/Space activation, `Escape` dismissal, and normal Tab exit.
- `Escape` closes an open Selection Actions Menu and retains selection. If removal confirmation is open, `Escape` cancels it and retains selection. Otherwise existing prepared-drag, Manage Items, selection-clear, and surface-dismissal ordering resumes.
- Tab closes the menu and resumes the existing surface Tab order from the logical invoking control.
- Dismissal restores focus to the invoking card or row when it still exists. If removal changes the target, existing same-index/previous/control fallback rules apply.
- Drop Shelf menu and confirmation changes use the existing polite announcement channel. Manage Items uses its existing item-selection announcement ownership.
- Edge Rail and Rail-origin Manage Items remain pointer-only/no-activate and add no keyboard focus target or live-region ownership.
- Menu and confirmation expose semantic roles, specific accessible names, selected scope, ownership-safe consequences, visible focus, and Windows system colors in Contrast Themes.

### Lifecycle integration

- Pin and Unpin retain selection. Remove prunes only disappeared identities. Cancel and failure retain selection.
- New drop-in does not join selection and does not close an unrelated menu unless projection ownership changes its target.
- Existing accepted Copy, hide, surface switch, popup replacement, explicit close, and restart selection-clear rules remain unchanged.
- Selection Actions do not change Batch Selection or Item Selection ordering, anchors, virtualization storage, drag payload order, availability classification, or occurrence identity.
- Item Actions remain read-only and target only the More button's Shelf Item regardless of current selection.
- Selection Actions never expose or derive authority from External Reference paths, Managed File paths, filenames, extensions, or presentation labels.
- No Windows toast is added. Success and failure feedback remain local to the originating surface.

### Documentation and delivery gates

- Amend the canonical Product Specification to permit selection-scoped Pin, Unpin, and Remove and to define menu, confirmation, lifecycle, accessibility, and no-activate behavior.
- Amend architecture documentation where the current Manage Items secondary-click boundary and Core mutation table would otherwise contradict the approved contract.
- The specification is published as `ready-for-agent` for ticket decomposition. It does not authorize implementation.
- `to-tickets` and explicit user approval are required before implementation. Final `0.0.27` acceptance remains a separate prerequisite.
- Candidate acceptance, Git commit authorization, publishing, and deployment remain separate explicit gates.

## Testing Decisions

- Good tests assert consumer-visible selection scope, lifecycle state, source safety, cleanup responsibility, focus, foreground ownership, accessibility semantics, and rollback. They do not assert private helper calls, command forwarding, exact SQL, internal collection types, CSS class strings, mock echoes, or source wording.
- Primary seam: one exact installed Windows x64 `0.0.28` Resident Instance using embedded production assets, owned Profile Data, and the real Drop Shelf, expanded Edge Rail, and shared Manage Items popup opened from both origins.
- The installed seam exercises ordinary and more-than-50-member virtualized selections, real primary modifier selection, real secondary-click, the Context Menu key and `Shift+F10` on focus-capable surfaces, menu navigation, confirmation, local feedback, focus restoration, and foreground ownership.
- The installed seam observes another application retaining foreground focus during Edge Rail and Rail-origin Manage Items menus, Pin/Unpin, Cancel, Remove confirmation, and removal. Unit tests cannot substitute for this observation.
- The installed seam uses External files/folders, plain-text Managed Files, image Managed Files, Temporary and Pinned states, Available/Missing/practically reproducible Unavailable states, and locked Managed Files that force durable pending deletion.
- The installed seam verifies all-External, mixed-ownership, and all-Managed confirmation text and counts, immediate singleton removal, multi-target confirmation, source-byte safety, permanent cleanup, pending retry, restart, and final popup/surface transitions.
- Supporting frontend coverage extends the existing mounted Vue interaction/accessibility seams used for Batch Selection, Manage Items Selection, Item Actions Menu, and selection Escape lifecycle.
- Frontend tests exercise selected versus unselected secondary-click targeting, ignored secondary-button modifiers, action-control isolation, pointer and keyboard menu anchoring, singular/plural labels, None/Mixed/All Pin labels, one-menu ownership, close triggers, confirmation ownership, busy blocking, selection retention/pruning, Tab, Escape, focus fallback, ARIA roles/names/states, and Contrast resources through observable behavior.
- Frontend virtualization coverage proves that selection scope and labels use stable identities rather than mounted rows and that unmount/scroll closes only stale transients without shrinking the authoritative target.
- Supporting native coverage extends the existing Core Shelf State seam with real temporary SQLite databases and temporary directories.
- Native tests use one table-driven selection-mutation matrix across the separate Batch Selection and Item Selection contracts. They cover non-empty unique identity validation, wrong-domain identities, cross-batch item identities, stale identities, duplicate identities, None/Mixed/All Pin transitions, exact selected removal, empty-batch pruning, final-item sizing reset, and one published snapshot.
- Native rollback tests inject persistence failure before Pin/Unpin and removal commit and prove complete visible and durable state remains unchanged.
- Native ownership tests compare exact external source bytes before and after removal and verify Managed File deletion authority comes only from authoritative ownership records.
- Native cleanup tests hold Managed Files open to force sharing failures, prove reference removal and pending-deletion responsibility commit together, release the handle, and prove later event/startup retry completes cleanup without restoring the Shelf Item.
- Native stale-confirmation tests prepare authoritative removal counts/targets, mutate the Shelf through a competing owner, and prove confirmation rejects the complete stale target with no partial removal.
- Regression coverage preserves existing single-item Pin/Unpin, Bulk Pinning, Remove Item, Remove Batch, Clear Temporary Items, Item Actions Menu, Batch Selection drag, Item Selection drag, virtualization, privacy, and no-activate behavior.
- Verification includes frontend tests, Vue typecheck/production build, freshly cargo-xwin-compiled native Windows regressions, cargo-xwin Windows build, exact NSIS packaging, installed restart/reinstall/uninstall, and Profile Data preservation during the approved release ticket.
- Exact test counts, artifact sizes and hashes, installed executable identity, foreground observer, input method, surfaces exercised, cleanup evidence, and unobserved physical limitations are recorded for candidate review.

## Out of Scope

- Persistent, named, or cross-surface Batch Selection.
- Cross-popup, persistent, or cross-batch Item Selection.
- Arbitrary item selection outside the active Manage Items batch.
- Additive `Ctrl+Shift` ranges, lasso/marquee selection, selection inversion, or selection history.
- Selection mode UI, a selection toolbar, always-visible checkboxes, new footer actions, or per-card/per-row count badges.
- Item or batch reordering.
- Undo, Recycle Bin removal, recoverable trash, or delayed user-controlled deletion.
- Partial-success Pin, Unpin, or Remove; skip-invalid behavior; per-target failure lists; or retry-selected UI.
- Merging Item Actions into Selection Actions, right-click Show in File Explorer, right-click Copy Path, right-click Copy Text, or new Managed Image actions.
- Changing ownership classification, Managed File storage layout, availability policy, drag representation, drag completion, or path privacy.
- Keyboard activation of Edge Rail or Rail-origin Manage Items, keyboard-only native OLE destination drag, or a new Rail live region.
- New settings, telemetry, dependencies, persistence schema, non-Windows behavior, or a new ADR.
- Beginning implementation, creating tickets without `to-tickets`, synchronizing release metadata, accepting a candidate, creating a Git commit, publishing, or deploying without the corresponding explicit authorization.

## Further Notes

The central boundary is selection domain, not affected item count. One selected Shelf Batch is one Batch Selection member even when it contains many Shelf Items; one selected Manage Items row is one Item Selection member. Labels, immediate-versus-confirmed removal, and target validation follow that boundary.

Selection remains presentation-owned and disposable. Core Shelf State receives exact identities and an explicit lifecycle action, validates the complete domain-specific target, and commits one transaction. It never persists selection or trusts presentation-provided ownership counts.

Selection Actions Menu is intentionally separate from Item Actions Menu. Item Actions remain read-only and ownership-aware through More. Selection Actions mutate lifecycle state through the visible selection and never expose Managed File paths.

The smallest complete implementation reuses the existing selection owners, menu interaction model, Core Shelf State serialization, managed cleanup queue, confirmation presentation patterns, transient Rail holds, and current frontend/native test harnesses. No generalized action framework or second lifecycle model is justified.
