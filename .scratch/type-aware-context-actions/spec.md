# DropCove 0.0.27 — Type-aware Item Actions

Status: resolved — amendment implemented 2026-10-27; frontend regressions, production build, isolated NSIS installation, installed smoke, and interactive Windows clipboard operations verified by user
Original design confirmation: approved by the user through `grill-with-docs` on 2026-10-26
Testing seam: confirmed by the user through `to-spec` on 2026-10-26 — one exact installed Windows Resident Instance exercising Item Actions Menu behavior on Drop Shelf, Edge Rail, and Manage Items surfaces with real clipboard and File Explorer effects, supported by frontend interaction/accessibility tests and Core Shelf State temporary SQLite/Profile Data regressions
Original ticket breakdown confirmation: approved by the user through `to-tickets` on 2026-10-26
Original approved tickets: [01 — Authoritative Item Classification and Managed File Path Privacy](issues/01-authoritative-item-classification-path-privacy.md), [02 — Drop Shelf Item Actions Round Trip](issues/02-drop-shelf-item-actions-round-trip.md), [03 — Manage Items Row Actions and Virtualized Lifecycle](issues/03-manage-items-row-actions-virtualized-lifecycle.md), [04 — Edge Rail No-activate Item Actions](issues/04-edge-rail-no-activate-item-actions.md), and [05 — Release DropCove 0.0.27](issues/05-release-dropcove-0.0.27.md)
Original approved DAG: `01 → 02 → {03, 04} → 05`
Amended release DAG: `01 → 02 → {03, 04}`, `03 → 06`, and `{04, 06} → 05`. Ticket 05 preserves the pre-amendment candidate record but its final human-acceptance gate is reopened until Ticket 06 is implemented and verified.
Original implementation authorization: user invoked `implement` and explicitly selected the complete original 0.0.27 specification on 2026-10-26. Tickets 01–05 were authorized and resolved; their candidate evidence remains unchanged.
Amendment confirmation: approved by the user through `grill-with-docs` on 2026-10-27
Amendment ticket: [06 — Suppress Default WebView Menu on Manage Items Rows](issues/06-suppress-manage-items-webview-menu.md)
Amendment implementation authorization: granted by the user's `implement` invocation on 2026-10-27. Human candidate acceptance, Git commits, publishing, and deployment remain separate gates. No commit is authorized.
Release assignment: 0.0.27
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 4, 8, 9, 10, 11, 13, 15, 19, and 20
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 2.3, 2.5, 2.7, 4, 6, 9, 14, 15, and 16
Relevant architectural decision: [ADR-0006 — Materialize non-filesystem drops as app-owned Managed Files](../../docs/adr/0006-managed-files-and-durable-cleanup.md)

## Contract Amendment — 2026-10-27

The user rejected replacing the Item Actions Menu with a right-click menu. The existing **More** entry point and authoritative action matrix remain unchanged on every surface:

| Shelf Item classification | Item Actions Menu |
|---|---|
| External Reference, file or folder | **Show in File Explorer**, then **Copy Path** |
| Plain-text Managed File | **Copy Text** |
| Image Managed File | No More button or Item Actions Menu |

More remains visible on every eligible Manage Items row under the existing action-control styling. Menus contain only valid actions, target only the row whose More button opened the menu, and do not alter Item Selection.

Manage Items gains one narrow right-click rule. The complete secondary-button sequence on a Shelf Item row, including `pointerdown` before `contextmenu`, remains selection-neutral and action-neutral; the `contextmenu` event is consumed so the embedded WebView does not show its default browser menu. The sequence does not open the Item Actions Menu, change Item Selection or its anchor, start drag, or dispatch an Item Action. More remains the sole Item Actions Menu entry point.

The suppression applies only at the Shelf Item row boundary in Manage Items. It must not interfere with existing primary-button activation or keyboard operation of the row's drag region, Pin/Unpin, More, or Remove controls. It does not change input or textarea editing menus, the Manage Items background, the Drop Shelf, or the Edge Rail. No native Windows context menu, keyboard context-menu-key handling, batch action, or additional pointer shortcut is added.

This amendment is the sole exception to the original statements that right-click/context-menu handling is not added and is out of scope. Those exclusions remain authoritative everywhere else.

### Amendment acceptance criteria

- The complete secondary-button sequence on any ordinary or virtualized Manage Items row, including `pointerdown`, `contextmenu`, and release, prevents the WebView default menu without triggering primary selection or drag behavior.
- Right-click suppression never opens an Item Actions Menu, changes Item Selection or its anchor, targets an action, or dispatches Pin/Unpin, More, Remove, or drag behavior.
- Existing primary-button row/control behavior and More keyboard opening, navigation, activation, and dismissal remain unchanged.
- External Reference, Managed Text, and Managed Image More/action behavior remains exactly as specified and previously verified, including Managed Text **Copy Text**.
- Right-click behavior outside Manage Items Shelf Item rows remains unchanged, including input/textarea editing behavior and the Edge Rail no-activate contract.
- Frontend behavior tests exercise the complete secondary-button event sequence and prove WebView-menu prevention, unchanged selection/anchor state, absence of menu/action/drag dispatch, preserved primary/keyboard behavior, and unaffected non-row boundaries.
- A newly packaged exact 0.0.27 candidate is smoke-tested on Windows for ordinary and virtualized Manage Items collections, existing More actions, Managed Text Copy Text, and the unchanged Edge Rail boundary.
- The 2026-10-26 candidate evidence remains historical and must not be represented as proof of this amendment. New artifact hashes and amendment evidence are required before human candidate acceptance.

No glossary change or ADR is required. **Item Actions Menu** remains the canonical term, **Context menu** remains avoided terminology for that product concept, and the amendment is a reversible presentation rule rather than an architectural choice.

## Problem Statement

DropCove exposes Shelf Item paths in its presentation projection, but the same path field represents two materially different ownership models.

An **External Reference** points to a filesystem object DropCove does not own. Its path is user-facing information that can safely be copied or located in File Explorer without granting DropCove authority to modify the object.

A **Managed File** is stored under DropCove-owned Profile Data and exists only to represent accepted non-filesystem content. Its internal path is an implementation detail governed by DropCove lifecycle and cleanup rules. Revealing or copying that path would expose private storage, encourage users or other applications to depend on a temporary location, and blur the boundary between external sources and DropCove-owned data.

The current UI has no direct action for locating an External Reference, copying its path, or copying the original content of a plain-text Managed File. Users must drag content out, manually transcribe a tooltip, or navigate independently. The frontend also infers Managed Text by searching for `/managed/` and `.txt` in path strings, while Managed Image tooltips and accessibility text can expose the internal Profile Data path. That makes the WebView depend on storage layout rather than the authoritative ownership model.

## Solution

Add an **Item Actions Menu** opened by a per-item **More** button.

The menu appears on:

- a single-item Shelf Batch card in the Drop Shelf;
- a single-item Shelf Batch card in the expanded Edge Rail;
- each Shelf Item row in Manage Items.

A multi-item Shelf Batch card does not expose item actions because it has no unambiguous Shelf Item target. Its items remain reachable through Manage Items.

Available actions are determined by authoritative Core Shelf State classification:

| Shelf Item classification | Item Actions Menu |
|---|---|
| External Reference, file or folder | **Show in File Explorer**, **Copy Path** |
| Plain-text Managed File | **Copy Text** |
| Image Managed File | No Item Actions Menu |

The frontend sends only the target Shelf Item identity and requested action. Core Shelf State resolves the current Shelf Item, validates ownership and content kind, and supplies the exact path or text required by the Windows integration adapter. The WebView never sends an authoritative path or text payload.

Copy actions write exact Unicode text to the real Windows clipboard without quotes, truncation, or lifecycle changes. Show in File Explorer resolves the current external path at invocation time and asks Windows Explorer to select that object. All Item Actions are read-only: they do not consume Temporary Items, alter Pin state, change selection, mutate availability, or refresh durable Shelf state.

Managed File internal paths are removed from user-facing tooltip and accessibility text. External References retain their complete path. Managed Text exposes its user content/name, while Managed Image exposes its user-visible name and the description `DropCove-created image`.

## User Stories

1. As a Drop Shelf user, I want to open an Item Actions Menu for a one-item batch, so that I can act on that exact item without dragging it.
2. As an Edge Rail user, I want the same item actions available on a one-item card, so that I do not need to open the Drop Shelf first.
3. As a Manage Items user, I want an Item Actions Menu on each row, so that I can act on one item in a multi-item batch.
4. As a user, I want multi-item cards to continue using Manage Items, so that a card-level action never silently chooses an arbitrary item.
5. As a user of an External Reference, I want to reveal the referenced file in File Explorer, so that I can navigate to its real filesystem location.
6. As a user of an External Reference folder, I want File Explorer to select the folder itself in its parent, so that Show in File Explorer has the same meaning for files and folders.
7. As a user of a drive or filesystem root reference, I want File Explorer to open that root, so that the action still has a meaningful result without a selectable parent.
8. As a user, I want Show in File Explorer to check the path when I activate it, so that a previously unavailable drive or network location can work after it reconnects.
9. As a user, I want a failed reveal to leave my Shelf Item untouched, so that a missing or inaccessible path cannot cause data loss.
10. As a user of an External Reference, I want to copy its exact absolute path, so that I can paste it into another application.
11. As a command-line user, I want Copy Path to omit automatic quotes, so that DropCove copies the path itself rather than guessing the syntax of the destination.
12. As a user, I want Copy Path to omit a trailing newline, so that pasting produces only the path I requested.
13. As a user, I want Copy Path to remain available for Missing and Unavailable External References, so that the retained reference remains useful even when the object cannot currently be reached.
14. As a user of a plain-text Managed File, I want to copy its full original text, so that I can paste the content without performing a native drag.
15. As a user of a long Managed Text item, I want Copy Text to use the complete content rather than the 500-character hover preview, so that no content is silently truncated.
16. As a user, I want clipboard actions to preserve Unicode and line breaks exactly, so that non-ASCII text and structured text remain intact.
17. As a user, I want clipboard actions to replace the current Windows clipboard contents in the normal way, so that paste behavior matches other Windows applications.
18. As a user, I want Copy Path and Copy Text to retain Temporary Items, so that clipboard use is not mistaken for Successful Drag-Out.
19. As a user, I want item actions to leave Pinned state unchanged, so that a read-only convenience action cannot change lifecycle intent.
20. As a user with an existing Item Selection, I want an Item Action to affect only the clicked row and leave selection unchanged, so that an unrelated selection does not retarget the action.
21. As a user with an existing Batch Selection, I want a one-item card action to affect only that card's item and leave Batch Selection unchanged, so that context actions never become implicit bulk actions.
22. As a user, I want Managed Images to omit an Item Actions Menu in this release, so that DropCove does not expose an internal path or imply unsupported Copy Image/Save As behavior.
23. As a user, I want Managed File tooltips to avoid Profile Data paths, so that temporary internal storage is not presented as a durable source location.
24. As a screen-reader user, I want Managed Files described by user-facing name/content and ownership-appropriate labels, so that internal storage paths are not announced.
25. As a user of External References, I want full paths to remain available in tooltip and accessibility text, so that removing Managed File leakage does not reduce useful source information.
26. As a keyboard user, I want the More button and Item Actions Menu to support standard menu navigation, so that the feature does not require a pointer.
27. As a keyboard user, I want `Escape` to close the Item Actions Menu and return focus to More, so that menu dismissal is predictable.
28. As a keyboard user, I want Arrow keys, Home, End, Enter, and Space to work consistently with the existing Drop Shelf Utility Menu, so that DropCove has one menu interaction model.
29. As a user, I want clicking outside the menu to dismiss it, so that transient item actions do not remain in the way.
30. As a user, I want only one Item Actions Menu open on a surface, so that there is always one unambiguous target.
31. As a user, I want a menu to close when its item disappears or the surface projection changes, so that an action cannot target stale state.
32. As a user, I want the menu to close when a drag, resize, modal confirmation, surface hide, or Manage Items close takes ownership, so that transient controls do not conflict.
33. As an Edge Rail user, I want Copy Path and Copy Text to preserve the foreground application, so that the no-activate Rail contract remains intact.
34. As an Edge Rail user, I accept File Explorer becoming foreground after explicitly choosing Show in File Explorer, so that the requested destination is visible.
35. As a user, I want a local confirmation after text is copied, so that an invisible clipboard operation has observable success feedback.
36. As a user, I want a local error if clipboard or Explorer integration fails, so that I can retry without guessing.
37. As a user, I do not want Windows toast notifications for routine item actions, so that frequent actions do not create system-level noise.
38. As a user, I want a successful Explorer reveal to rely on the visible Explorer result rather than an extra DropCove banner, so that feedback is not duplicated.
39. As a user, I want an action request validated against the current Shelf Item identity, so that stale UI cannot expose or copy a different item's data.
40. As a user, I want invalid ownership/action combinations rejected, so that a Managed File can never be revealed or copied as an External Reference path.
41. As a user, I want restart to preserve Shelf state exactly as before, so that adding read-only actions does not introduce new persistence.
42. As a user, I want update and uninstall behavior unchanged, so that 0.0.27 remains compatible with the established installer lifecycle and Profile Data retention rules.

## Implementation Decisions

### Domain and projection contract

- **External Reference** and **Item Actions Menu** are canonical terms recorded in the project glossary.
- Shelf Item presentation data gains an authoritative ownership field with values `External` or `Managed`.
- A Managed Shelf Item also gains an authoritative content kind with values `Text` or `Image`; External References have no Managed content kind.
- Ownership and Managed content kind come from Core Shelf State and persistence knowledge. The frontend must not infer either value from path substrings, filename extensions, Profile Data roots, or presentation labels.
- The ownership/content-kind combination is internally valid by construction: External plus no Managed content kind, or Managed plus exactly one supported content kind.
- No persistence schema change is required. External versus Managed ownership already exists through Managed File records, and current accepted Managed content is already limited to plain text and validated images.
- The projection remains serializable through the existing Shelf snapshot and management projection channels so every current surface observes the same classification.

### Authoritative action boundary

- Add one read-only item-action contract accepting a Shelf Item identity and a finite action enum: Show in File Explorer, Copy Path, or Copy Text.
- The WebView sends only the identity and action. It never supplies the path, text, ownership, content kind, or clipboard payload as authority.
- Core Shelf State resolves the current retained item and validates the requested action against authoritative classification:
  - Show in File Explorer and Copy Path require an External Reference.
  - Copy Text is offered for every retained plain-text Managed File regardless of cached `Available`, `Missing`, or `Unavailable` classification.
  - On invocation, Copy Text requires a committed Managed File record in persistence state `ready` and attempts a live read of its current bytes. Here `ready` describes the Managed File record, not cached item availability.
  - Cached Missing or Unavailable state never hides the menu or pre-rejects Copy Text. A restored/readable file can succeed before any projection refresh; a currently unreadable or absent file reports a local error without cleanup or availability mutation.
  - Managed Images support none of these actions.
- A missing item identity, staged/unready Managed File record, invalid action, wrong ownership, wrong content kind, unreadable Managed Text file, clipboard failure, or Shell failure returns a user-facing error without mutation.
- The action resolver returns owned data needed by the Windows adapter and does not hold persistence or state locks while waiting on Explorer or clipboard APIs.
- Item actions do not enter the serialized mutation gate because they do not change Shelf state. They must still resolve against one coherent current state snapshot before the native side effect begins.
- No item action invokes drag preparation, drag completion, availability cleanup, Managed File deletion, Pending Managed File Deletion, Pin mutation, or selection mutation.

### Windows clipboard behavior

- Clipboard writes use the native Windows Unicode text representation.
- Copy Path writes exactly the retained absolute External Reference path, without quotes, newline, normalization, canonicalization, or existence requirement.
- Copy Text reads and writes the complete Managed Text content, preserving Unicode and line endings. It never uses the bounded presentation preview.
- Copy Text capability is independent of cached availability. Invocation always uses a live Managed File read after authoritative identity, ownership, content-kind, and committed-record validation.
- A successful clipboard write replaces current clipboard contents according to normal Windows behavior.
- Clipboard contention or API failure reports a local error. It does not mutate the Shelf Item or silently fall back to a WebView-only clipboard path.
- Existing Windows dependencies and capabilities are sufficient; no new package is required solely for text clipboard access.

### File Explorer behavior

- Show in File Explorer is offered for every External Reference regardless of its cached availability classification.
- Invocation evaluates the current retained path rather than trusting cached Available/Missing/Unavailable presentation state.
- For a file or non-root folder, File Explorer opens the parent and selects the referenced object.
- For a filesystem root that has no selectable parent, File Explorer opens the root.
- Missing, inaccessible, malformed, or rejected paths produce a local error and leave cached availability and durable Shelf state unchanged.
- Successful invocation needs no DropCove success banner because the visible Explorer result is the direct outcome.
- Explicit Show in File Explorer may activate File Explorer. Copy actions must not activate DropCove when invoked from the Edge Rail.

### Item Actions Menu presentation

- Use one shared Item Actions Menu behavior across Shelf Batch cards and Manage Items rows.
- The trigger is a named 24-by-24 **More** action matching established DropCove icon-action styling, focus visibility, High Contrast behavior, and disabled treatment.
- On eligible cards and rows, action-control order is Pin/Unpin, More, then Remove. Managed Images retain Pin/Unpin then Remove because they have no Item Actions Menu.
- A single-item External Reference card or row renders menu items in this order: Show in File Explorer, then Copy Path.
- A single-item Managed Text card or row renders Copy Text.
- A Managed Image renders no More button because it has no actions in 0.0.27.
- Multi-item cards render no Item Actions Menu. Their Item Actions are reached through Manage Items rows.
- The More button is an independent action control. Pointer or keyboard activation must not initiate drag, change Batch Selection, change Item Selection, or move a selection anchor.
- The menu is anchored to More, overlays without resizing the Shelf/Edge Rail/Manage Items surface, and remains within the current surface bounds.
- At most one Item Actions Menu is open per presentation surface.
- The menu closes after an action, on `Escape`, on outside pointer interaction, when another item menu opens, when the target item disappears, when projection ownership changes, when virtualization unmounts the target, when the surface scrolls the target away, or when the surface closes/hides.
- The menu also closes when drag, resize, window movement, modal confirmation, prepared transfer, Clear Temporary Items, or another busy transaction takes presentation ownership.
- Right-click never opens the Item Actions Menu, and no native Windows context menu is added. The only `contextmenu` handling is the 2026-10-27 amendment's scoped suppression of the WebView default menu on Manage Items rows.

### Keyboard, focus, and accessibility

- The menu follows the existing Drop Shelf Utility Menu interaction contract: `role="menu"`, menu items, Arrow Up/Down wrapping, Home/End, Enter/Space activation, `Escape` dismissal, outside-click dismissal, and normal Tab exit.
- Opening focuses the first menu item on focus-capable surfaces.
- Dismissal without launching another foreground application restores focus to the originating More button when it still exists.
- Copy success closes the menu and restores focus to More.
- Show in File Explorer may transfer foreground focus to Explorer and does not force DropCove back to foreground. If the user later returns, the originating control remains the logical focus-restoration target when still present.
- The Edge Rail remains pointer-only/no-activate. Its item menu must not call foreground activation or force keyboard focus into DropCove.
- Accessible names identify the target without announcing internal Managed File paths.
- External References retain exact full-path tooltip and accessibility information.
- Managed Text uses its user-visible name and text content/preview where appropriate, never its Profile Data path.
- Managed Image uses its user-visible name plus `DropCove-created image`, never its Profile Data path.
- Menu/action success uses polite local announcement where the surface owns an announcement channel. Failures use the surface's visible local error/status treatment.
- Edge Rail success uses local non-toast visual feedback without claiming keyboard or screen-reader focus ownership; failures remain visible on the Rail.

### Lifecycle and failure behavior

- Copy Path, Copy Text, and Show in File Explorer are read-only convenience actions.
- Temporary Items remain Temporary and retained after every item action outcome.
- Pinned Items remain Pinned.
- Batch Selection and Item Selection remain unchanged.
- Successful actions do not refresh, reorder, resize, hide, dock, open Manage Items, or close a Shelf Batch.
- Failed actions do not clean Missing Items, reclassify availability, remove Managed Files, or create pending cleanup responsibility.
- An action targets only the Shelf Item whose More button opened the menu, even if that item belongs to the current selection.
- If the target disappears before action resolution, the request fails as stale and no native side effect occurs.
- No Windows notification is emitted for routine success or failure; feedback remains on the originating DropCove surface.

### Version and documentation

- The complete behavior ships as DropCove 0.0.27; there is no partial classification-only or menu-only release.
- Behavior implementation retains version 0.0.26 until the release slice synchronizes canonical, package, crate, lockfile, Tauri, installer, executable, About Surface, and uninstall metadata.
- Canonical Product Specification, Architecture, and Roadmap documentation must be updated with the final 0.0.27 contract and evidence.
- No new ADR is required. The feature applies the existing External Reference versus Managed File ownership decision rather than introducing a new hard-to-reverse architecture choice.

## Testing Decisions

- Tests must assert consumer-visible behavior and safety boundaries, not implementation wording or forwarding.
- The highest seam is one exact installed Windows x64 0.0.27 Resident Instance using embedded production assets and owned Profile Data.
- Installed proof covers the Item Actions Menu on a single-item Drop Shelf card, a single-item expanded Edge Rail card, and Manage Items rows in both ordinary and more-than-50-item collections.
- Installed proof uses real External file, External folder, filesystem-root, Managed Text, and Managed Image fixtures with Available, Missing, and practically reproducible Unavailable conditions where possible.
- Installed proof reads back the real Windows clipboard to verify exact Unicode path/text bytes, line endings, no quotes, no newline, and full content beyond the presentation preview length.
- Installed proof observes File Explorer selecting the exact file and folder and opening a root. Failure cases must produce local feedback and retain state.
- Installed proof observes foreground ownership: Edge Rail Copy actions do not activate DropCove; explicit Show in File Explorer may activate Explorer.
- Installed proof inspects rendered tooltip/accessibility information to prove Managed File Profile Data paths are absent and External Reference paths remain available.
- Installed proof exercises pointer and keyboard menu opening, standard navigation, activation, Escape, outside click, focus restoration, stale-target closure, busy/modal closure, virtualization unmount, and surface close/hide.
- Installed proof confirms Item Selection and Batch Selection remain unchanged and actions target only the clicked item.
- Supporting frontend tests extend the existing component renderer seams for Shelf Batch cards and Manage Items rows and cover a shared Item Actions Menu through observable DOM, emitted events, focus, ARIA state, and labels.
- Frontend tests cover capability mapping for External, Managed Text, and Managed Image items; menu item order; More visibility; selection isolation; keyboard behavior; one-menu ownership; close triggers; local feedback; and privacy text.
- Supporting native tests use real temporary SQLite databases, temporary Profile Data, and actual Managed File bytes to exercise authoritative identity resolution, ownership/content-kind projection, exact path/text extraction, invalid action rejection, stale identity rejection, and no Shelf mutation.
- Native tests compare complete pre/post Shelf snapshots and source/Managed bytes where needed to prove actions do not consume, pin, remove, reorder, clean, or persist new state.
- Windows clipboard and Explorer side effects are proven at the installed seam rather than by tests that merely assert a mock adapter call.
- Regression verification includes the complete frontend test suite, Vue typecheck/production build, freshly cargo-xwin-compiled native test executable, cargo-xwin Windows build, and NSIS packaging.
- Release verification includes exact candidate installation, restart, same-candidate reinstall/update, normal uninstall, and Profile Data preservation.
- Record exact test counts, installer/executable sizes and SHA-256 hashes, Windows build, surfaces exercised, clipboard/Explorer observations, accessibility tooling, focus observations, and any physical-environment limits.
- Do not add tests for SQL text, private helpers, exact internal enum layout, CSS class strings, command forwarding, mock echoes, source wording, or bare non-throw behavior.

## Acceptance Criteria

### Classification and privacy

- Every projected Shelf Item has an authoritative valid ownership/content-kind classification.
- Frontend behavior contains no `/managed/` or filename-extension inference for ownership or Managed Text capability.
- External References retain complete user-facing paths.
- Managed Text and Managed Image surfaces expose no Profile Data path in tooltips, accessible names/descriptions, menu labels, announcements, or status messages.
- Managed Image shows no More action in 0.0.27.

### Menu and targeting

- More appears on eligible single-item Drop Shelf and Edge Rail cards and eligible Manage Items rows.
- Multi-item cards do not expose an ambiguous Item Actions Menu.
- External menus contain Show in File Explorer followed by Copy Path; Managed Text menus contain Copy Text only.
- Managed Text menus remain available for cached Available, Missing, and Unavailable items; Copy Text attempts a live read and either copies current bytes or reports a non-mutating local error.
- Item Actions never alter Batch Selection, Item Selection, selection anchors, Pin state, Temporary/Pinned lifecycle, ordering, sizing, or display state.
- A menu action always targets the item whose More button opened it and rejects a stale or invalid identity.
- Only one menu is open per surface, and all approved lifecycle/ownership changes close it.

### Clipboard and Explorer

- Copy Path writes the exact retained absolute path as Unicode text with no quotes or newline for Available, Missing, and Unavailable External References.
- Copy Text writes the complete exact Managed Text content with Unicode and line endings preserved.
- A Managed Text file restored after cached Missing/Unavailable classification can copy successfully without a projection refresh; an item whose bytes remain unreadable is retained unchanged.
- Copy actions replace the real Windows clipboard and retain the target Shelf Item after success or failure.
- Show in File Explorer selects an existing external file and non-root folder, opens a filesystem root, and reports inaccessible/missing failures without Shelf mutation.
- No action can reveal or copy a Managed File internal path.

### Accessibility, focus, and feedback

- More and all menu items have specific accessible names and visible focus treatment in Dark, Light, and Windows Contrast presentation.
- Arrow Up/Down, Home/End, Enter/Space, Escape, Tab, and outside-click behavior match the existing menu contract.
- Copy success announces `Path copied.` or `Text copied.` locally and restores focus where the surface supports focus.
- Failures identify the failed operation locally without a Windows notification.
- Show in File Explorer success does not add redundant DropCove success feedback.
- Edge Rail remains no-activate for menu opening and Copy actions; explicit Explorer reveal may activate Explorer.

### Release proof

- Frontend regressions, production build, native regressions, cargo-xwin Windows build, and exact NSIS packaging pass.
- One exact installed 0.0.27 candidate runs embedded production assets with the development server unavailable and passes the primary testing seam.
- Restart and reinstall preserve Shelf/Profile Data; uninstall returns successfully and follows existing Profile Data preservation behavior.
- Exact artifact hashes and all observed/unobserved acceptance limits are recorded before human candidate acceptance.

## Demo Path

1. Install the exact 0.0.27 candidate with an owned profile and create one external file batch, one external folder batch, one Managed Text batch longer than 500 characters, one Managed Image batch, and one multi-item mixed batch.
2. On the Drop Shelf, open an External file Item Actions Menu by pointer and keyboard; inspect menu order, Copy Path exact clipboard contents, focus restoration, and unchanged Shelf/selection state.
3. Choose Show in File Explorer and observe the exact file selected. Repeat for a folder and filesystem root.
4. Remove or disconnect an external fixture after intake; invoke Show in File Explorer and observe local failure with the External Reference retained. Copy its path successfully despite cached Missing/Unavailable state.
5. Open the Managed Text Item Actions Menu and copy full multilingual/multiline content; verify clipboard bytes exceed the hover preview and the Temporary Item remains.
6. Inspect the Managed Image card and Manage Items row; verify no More button and no Profile Data path in tooltip or accessibility output.
7. Open Manage Items for the mixed batch, create a multi-item selection, invoke an action on one selected and one unselected row, and verify only the clicked row is targeted while selection remains unchanged.
8. Repeat the eligible card actions from the expanded Edge Rail while another application owns foreground focus; verify Copy does not activate DropCove and Show in File Explorer intentionally activates Explorer.
9. In a collection above 50 items, open a row menu, navigate by keyboard, scroll/unmount or remove the target, and verify the stale menu closes without an action.
10. Exercise Escape, outside click, another More button, drag start, modal confirmation, surface hide, and Manage Items close; verify one-menu ownership and focus/lifecycle behavior.
11. Force clipboard contention and inaccessible Explorer targets; verify operation-specific local errors, no Windows toast, no mutation, and successful retry after the external condition clears.
12. Restart, reinstall/update, and uninstall the exact candidate; verify Profile Data preservation, unchanged source objects, synchronized 0.0.27 metadata, and clean owned-test cleanup.

## Dependencies and Blockers

- Depends on the existing Core Shelf State ownership records, Shelf snapshots, management projections, Managed File Store, and exact Managed Text reader.
- Depends on the existing Shelf Batch card, Manage Items row, Drop Shelf Utility Menu interaction pattern, announcement/status channels, virtualization, and Edge Rail no-activate contract.
- Depends on Windows Shell, clipboard, and window/foreground facilities already available through the project's Windows bindings.
- Depends on completed 0.0.26 Item Selection behavior because Item Actions must remain target-local and selection-neutral.
- No new dependency, Settings field, database migration, durable state, Managed File lifecycle, or ADR is required.
- Tickets 01–05 and the original full-spec implementation were authorized and resolved. Ticket 06 implementation is authorized and completed; remaining candidate acceptance limits are recorded in its evidence. Human candidate acceptance, Git commits, publishing, and deployment remain separate explicit gates.

## Out of Scope

- Batch-level Copy Paths or Show in File Explorer.
- Item Actions applied to Batch Selection or Item Selection.
- Multi-path clipboard formatting, separators, quoting modes, shell-command escaping, or destination-specific path syntax.
- Copy Image, bitmap clipboard formats, animated-image handling, Save As, export, Open With, or opening Managed File storage.
- Clipboard file-list formats, paste intake, clipboard history, or background clipboard monitoring.
- Native Windows right-click context menus, keyboard context-menu keys, or additional pointer shortcuts.
- New Settings, hotkeys, audio feedback, notifications, telemetry, or action history.
- Reclassification watchers, filesystem watchers, path canonicalization, link resolution policy, or availability mutation from Item Actions.
- Changes to Temporary/Pinned consumption, Managed File cleanup, pending deletion, persistence schema, recovery, or uninstall retention.
- Changes to card/row density, Manage Items geometry, Edge Rail geometry, Drop Shelf sizing, virtualization threshold, or selection behavior.
- Non-Windows behavior or elevated cross-integrity guarantees.

## Further Notes

- `GLOSSARY.md` now defines **External Reference** and **Item Actions Menu**.
- The approved design deliberately distinguishes user-facing external paths from DropCove-owned Profile Data paths.
- A future Managed Image export feature should be designed separately because Copy Image and Save As require format, alpha, animation, filename, overwrite, and lifecycle decisions not present here.
- Tickets 01–05 and their original DAG remain the approved, resolved pre-amendment implementation record. Ticket 06 records the implemented 2026-10-27 amendment, with interactive clipboard functionality verified by the user; Git commits, publishing, and deployment remain separate explicit gates.
