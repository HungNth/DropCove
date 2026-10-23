# DropCove 0.0.23 — Managed File Lifecycle and Plain-Text Intake

Status: ready-for-agent
Design confirmation: approved by the user through `grill-with-docs` on 2026-10-23
Testing seam: the actual Windows Resident Instance receiving real OLE Unicode text on the Drop Shelf and expanded Edge Rail, then performing native Copy drag-out into a real destination, supported by the existing Core Shelf State, temporary-profile SQLite/filesystem, failure-injection, and Recovery Backup regression seams
Release assignment: 0.0.23
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md#0023--managed-file-lifecycle-and-plain-text-intake)
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 12.3, 13, 14, and 17
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 2.3–2.7 and 4–10
Architectural decision: [ADR-0006 — Materialize non-filesystem drops as app-owned Managed Files](../../docs/adr/0006-managed-files-and-durable-cleanup.md)
Research: [Windows non-path drag payloads](../content-intake-multi-batch/research/windows-non-path-drag-payloads.md)
Ticket decomposition: user approved four tracer-bullet tickets with DAG `01 → {02, 03} → 04` on 2026-10-23: [01 — Managed-text Drop Shelf round trip](issues/01-managed-text-drop-shelf-round-trip.md), [02 — Lifecycle actions and Edge Rail parity](issues/02-lifecycle-actions-edge-rail-parity.md), [03 — Managed File recovery and profile preservation](issues/03-managed-file-recovery-profile-preservation.md), and [04 — Release DropCove 0.0.23](issues/04-release-dropcove-0.0.23.md)
Implementation approval: user requested implementation of all four tickets on 2026-10-23; exact-candidate human acceptance and Git commit authorization remain separate gates

## Problem Statement

DropCove currently accepts only Windows storage items that resolve to filesystem paths. A user can stage files and folders, but cannot drag selected text into either Shelf surface and later use that text as a normal file. The existing Shelf Item model also has no explicit representation for a filesystem file created and owned by DropCove.

Creating app-owned files introduces a safety problem that does not exist for external references. Successful Drag-Out, Remove Item, Remove Batch, Clear Temporary Items, and confirmed-Missing cleanup currently remove database references while deliberately leaving source objects untouched. A Managed File must instead be permanently deleted when its Shelf Item lifecycle ends, without ever extending deletion authority to an external source path. SQLite and the filesystem do not share one transaction, so intake and cleanup must remain deterministic through persistence failure, sharing violations, process termination, restart, update, uninstall, and database recovery.

The user needs a complete plain-text round trip that behaves like the existing file workflow while preserving source-object safety, durable Temporary/Pinned semantics, Profile Data retention, rollback guarantees, and Recovery Backup evidence.

## Solution

Add Managed File ownership as an optional relation attached to a Shelf Item. Existing files and folders remain external path references. A Shelf Item is app-owned only when an authoritative Managed File ownership record exists; path location alone never grants deletion authority.

Extend the native Windows OLE intake adapter to accept non-empty `CF_UNICODETEXT` after existing filesystem-path input. Encode the text as a bounded UTF-8 `.txt` Managed File, create one one-item Shelf Batch, and expose it through the established Shell Visual, Pin/Unpin, Manage Items, Copy-only drag-out, Remove, Clear, sizing, persistence, Drop Shelf, Edge Rail, and Shake-to-open workflows.

Store Managed Files under Profile Data using per-item directories and a durable `Staged` → `Ready` intake lifecycle. Only `Ready` items are user-visible. Reconcile interrupted staging and promotion before restoring the Shelf so every crash window produces either one complete visible item or no visible item and no unowned file.

When a Managed File-backed Shelf Item ends its lifecycle, atomically remove the Shelf reference and record Pending Managed File Deletion responsibility before attempting permanent deletion. Delete immediately when possible; otherwise keep a durable pending record, warn with the pending count, and retry at later cleanup-producing events and startup. External references never enter this deletion path.

## User Stories

1. As a Drop Shelf user, I want to drag selected plain text into DropCove, so that I can stage text without first creating a file manually.
2. As an Edge Rail user, I want to drag selected plain text into the expanded Rail, so that text intake is available from either Shelf surface.
3. As a user dropping text, I want one drop to create one Shelf Batch with one Shelf Item, so that text follows the same grouping model as other intake.
4. As a user dropping multiline text, I want every line preserved, so that the later `.txt` file contains the content I selected.
5. As a user dropping non-ASCII text, I want Unicode characters preserved exactly, so that names and content in my language are not corrupted.
6. As a user dropping URL-shaped text, I want it stored as literal text, so that DropCove does not unexpectedly open or download the URL.
7. As a user dragging empty text, I want the drop rejected without creating Shelf state, so that meaningless items do not appear.
8. As a user dragging whitespace-only text, I want the whitespace preserved as content, so that DropCove does not silently reinterpret a non-empty selection.
9. As a user dragging text whose UTF-8 representation is at the `10 MiB` limit, I want the boundary handled deterministically, so that accepted and rejected sizes are predictable.
10. As a user dragging text larger than `10 MiB`, I want the drop rejected with feedback, so that one drag cannot consume unbounded Profile Data.
11. As a user, I want a timestamped `.txt` filename, so that the created file is recognizable at its destination.
12. As a user creating multiple text items in the same second, I want numeric filename suffixes, so that every Managed File keeps a distinct user-visible name.
13. As a user viewing a text Shelf Item, I want the normal Windows text-file icon or Shell fallback, so that it looks and behaves like a file.
14. As a user opening Manage Items, I want the text item to expose its exact Managed File path and availability through established tooltip and accessibility behavior, so that its identity remains inspectable.
15. As a user dragging a text item into another application, I want native Windows Copy behavior, so that destinations receive a normal `.txt` file.
16. As a user completing Copy for a Temporary text item, I want its Shelf reference consumed, so that successful use follows existing Temporary Item semantics.
17. As a user completing Copy for a Temporary text item, I want DropCove to permanently delete the owned Managed File, so that used text does not accumulate on disk.
18. As a user completing Copy for a Pinned text item, I want its Shelf reference and Managed File retained, so that I can reuse it later.
19. As a user canceling a text drag-out with Escape, I want the Shelf Item and Managed File retained, so that cancellation cannot lose content.
20. As a user dropping a text item on a rejecting destination, I want the Shelf Item and Managed File retained, so that rejected Copy cannot consume it.
21. As a user encountering a pre-acceptance drag failure, I want the Shelf Item and Managed File retained, so that integration failure cannot lose content.
22. As a user removing a Managed File-backed item, I want the item removed and its owned file permanently deleted, so that explicit cleanup reclaims disk space.
23. As a user removing a batch containing a Managed File, I want every owned file in that batch permanently deleted, so that batch removal is complete.
24. As a user confirming Clear Temporary Items, I want every Temporary Managed File permanently deleted while every Pinned Item remains, so that cleanup respects lifecycle state.
25. As a user canceling Clear Temporary Items, I want references and files unchanged, so that the safe default remains non-destructive.
26. As a user clearing a mixed Shelf, I want external Temporary references removed without deleting their source objects, so that Managed File cleanup never weakens source-file safety.
27. As a user removing a Pinned external file reference, I want only the reference removed, so that Pin state cannot grant source deletion authority.
28. As a user removing a Pinned Managed File, I want the owned file deleted because I explicitly removed it, so that Pin affects Successful Drag-Out but not explicit removal.
29. As a user, I want Managed Files permanently deleted rather than sent to the Recycle Bin, so that cleanup does not move disk usage elsewhere.
30. As a user whose Managed File is temporarily locked, I want the Shelf mutation to complete and cleanup responsibility to remain durable, so that antivirus or destination handles do not block Shelf use.
31. As a user with pending Managed File cleanup, I want a visible count warning, so that I know disk cleanup is not yet complete.
32. As a user who releases a locked Managed File, I want a later cleanup-producing event or application restart to retry deletion, so that manual repair is normally unnecessary.
33. As a user restarting DropCove, I want retained Temporary and Pinned Managed Files restored, so that restart does not lose staged text.
34. As a user updating DropCove, I want retained and pending Managed Files preserved with Profile Data, so that an update does not destroy content or cleanup responsibility.
35. As a user uninstalling DropCove, I want Profile Data—including retained and pending Managed Files—preserved under the existing uninstall contract, so that reinstall can restore state.
36. As a user reinstalling DropCove, I want retained Managed Files to remain usable and pending deletion to resume, so that uninstall/reinstall does not orphan ownership state.
37. As a user whose Shelf database becomes corrupt, I want Managed File storage preserved in the same Recovery Backup as the database and sidecars, so that the evidence remains internally meaningful.
38. As a user whose recovery preservation cannot complete, I want startup to fail rather than partially move or delete Managed Files, so that recovery never destroys uncertain content.
39. As a user, I want interrupted text intake before durable ownership to leave no visible item and no untracked staging file, so that crashes do not leak data.
40. As a user, I want interrupted intake after the `Staged` commit to reconcile on restart, so that the item either finishes completely or disappears before presentation.
41. As a user, I want interrupted intake after file promotion but before `Ready` commit to reconcile on restart, so that a complete file is not discarded or exposed inconsistently.
42. As a user, I do not want `Staged` Managed Files rendered in the Drop Shelf or Edge Rail, so that incomplete intake is never mistaken for accepted content.
43. As a user, I want a missing owned file discovered during cleanup to count as already deleted, so that stale cleanup records do not persist forever.
44. As a user removing the final Shelf Item, I want the established sizing reset and Hidden transition preserved, so that Managed File cleanup does not change display behavior.
45. As a user, I want the Clear confirmation to state that external sources stay while DropCove-created temporary files are permanently deleted, so that destructive scope is explicit.
46. As a Shake-to-open user, I want accepted text to keep the Shelf visible and unsupported or failed text intake to restore the prior display state, so that text follows the established Shake session contract.
47. As a user dropping identical text twice, I want two intentional Shelf Batches rather than silent content deduplication, so that repeated user actions remain visible.
48. As a user, I want ordinary file and folder intake, duplicate-path rules, Pinning, Manage Items, Edge Rail, sizing, appearance, accessibility, and Copy-only drag-out preserved, so that text support composes with existing workflows.
49. As a maintainer, I want Managed File ownership represented explicitly rather than inferred from a path prefix, so that deletion can never spread to external objects.
50. As a maintainer, I want Core Shelf State to remain the single lifecycle owner, so that database state, files, sizing, surfaces, and cleanup cannot diverge across callers.
51. As a maintainer, I want one deep Managed File Store module behind Core Shelf State, so that staging, promotion, deletion, reconciliation, and recovery stay local.
52. As a maintainer, I want existing external Shelf Items to require no compatibility alias or second lifecycle path, so that absence of a Managed File relation simply means external ownership.
53. As a release reviewer, I want the actual installed Windows application to prove text intake and cleanup end to end, so that unit or build success is not mistaken for product acceptance.
54. As a release owner, I want version metadata to remain `0.0.22` during behavior work and become `0.0.23` only at the approved release gate, so that roadmap sequencing stays auditable.
55. As a project owner, I want specification, ticket approval, implementation, release acceptance, and Git commit authorization to remain separate decisions, so that planning cannot imply deployment or commit approval.

## Implementation Decisions

- Scope this specification to roadmap version `0.0.23`: Managed File ownership, durable lifecycle cleanup, Recovery Backup integration, and plain-text intake. Website-image virtual files belong to `0.0.24`; multi-batch selection belongs to `0.0.25`.
- Preserve ordinary file and folder intake as external filesystem references. Do not copy their bytes into Profile Data and never delete, move, rename, or overwrite their source objects.
- Keep `Shelf Item` as the user-visible entity. Determine app ownership only through an optional authoritative Managed File relation. A path inside `%LOCALAPPDATA%\DropCove` without that relation is not deletable by Managed File cleanup.
- Deepen Core Shelf State rather than adding another lifecycle owner. Presentation and the Windows OLE Adapter request intake or existing Shelf mutations; Core Shelf State coordinates SQLite, Managed File Store, sizing, display synchronization, warnings, and lifecycle results.
- Add one internal Managed File Store module. Its interface owns staging, final promotion, permanent deletion, startup reconciliation, and Recovery Backup participation. Do not expose filesystem ordering requirements to Vue or duplicate them in command handlers.
- Extend the Windows OLE Adapter to identify one representation family for `0.0.23`: valid existing filesystem paths first, otherwise non-empty `CF_UNICODETEXT`. If a data object offers higher-priority filesystem or virtual-image file formats but they are invalid or unsupported in this version, reject the drop rather than silently materializing companion URL/text data.
- Keep borrowed `IDataObject`, `FORMATETC`, `STGMEDIUM`, and text memory access on the registered drop-window thread. Copy the accepted Unicode text into owned memory before returning from the OLE callback; COM-owned media never cross to blocking persistence work.
- Reject application-owned drag-out as intake, preserve Copy-only eligibility, Clear-confirmation blocking, Drop Placeholder behavior, Edge Rail expansion timing, Shake session handling, and native cursor feedback.
- Treat text as non-empty when at least one Unicode code unit remains after the required terminal NUL is removed. Whitespace-only text is valid. Reject malformed Unicode rather than replacing it silently.
- Encode accepted text as UTF-8 without a BOM. Preserve the selected text's code points and line endings; do not trim, normalize whitespace, reinterpret URLs, retain HTML/RTF, or add formatting.
- Enforce the `10 MiB` limit against final UTF-8 bytes. The exact limit is accepted; one byte over is rejected. A rejected text drop creates no Shelf Batch, Managed File, staging residue, sizing change, scroll change, or Shake promotion.
- Generate the user-visible filename from local time as `Dropped text YYYY-MM-DD HH-mm-ss.txt`. If that name is already retained or staged for the same timestamp, add ` (2)`, ` (3)`, and so on before `.txt`. Keep time/collision calculation deterministic through an internal test seam rather than a user setting.
- Create one Shelf Batch with one Temporary Shelf Item for each accepted text drop. Repeated identical text creates another Managed File and batch; do not hash content, suppress by bytes, or share backing files.
- Store final bytes under `%LOCALAPPDATA%\DropCove\managed\<item-id>\<sanitized-name>`. Use a dedicated same-volume staging root. Per-item directories preserve the filename without filename collision or reference counting.
- Extend shelf persistence with an optional `managed_files` relation keyed by Shelf Item identity and containing final relative path, `Staged | Ready` ingestion state, and an optional staging relative path. Existing Shelf Items with no relation are external references.
- Add durable `pending_managed_file_deletions` records containing only paths whose app ownership was established by the Managed File relation. Cleanup must not infer authority from a directory prefix.
- Use the approved crash-recoverable intake order: write/flush text staging; commit batch, item, and Managed File ownership as `Staged`; atomically rename to the final per-item path; commit `Ready`; then publish the Shelf snapshot and accepted-drop result.
- Exclude `Staged` rows from every Shelf, Rail, Manage Items, visual, sizing, and duplicate projection. Only `Ready` Managed Files are user-visible and eligible for drag-out.
- Reconcile Managed File intake before normal Shelf restoration. Remove unowned staging entries; promote a valid staged file; mark an already-promoted valid file `Ready`; or remove the unprojected item/batch with a warning when neither valid file exists. Reconciliation must also preserve final-item sizing invariants.
- On runtime failure after `Staged` commit, perform the same compensating item/batch removal and staging/final cleanup. If process termination interrupts compensation, startup reconciliation owns completion.
- On Successful Drag-Out consumption, Remove Item, Remove Batch, Clear Temporary Items, or confirmed-Missing cleanup, remove the Shelf Item and insert Pending Managed File Deletion in the same SQLite transaction. External items create no deletion record.
- Attempt permanent file and empty per-item-directory deletion only after the transaction commits. Missing targets count as successful deletion. Successful cleanup removes the pending record.
- If deletion fails because of sharing, antivirus, permissions, or another retryable filesystem condition, keep the Shelf mutation committed, retain the pending record, and report `1 DropCove-created file is pending deletion.` or `N DropCove-created files are pending deletion.`
- Retry pending deletion during startup and after later cleanup-producing mutations. Do not add a timer, watcher, resident polling loop, manual cleanup command, setting, or second cleanup UI.
- Keep Pinned Managed Files through Successful Drag-Out, restart, update, and uninstall. Explicit Remove Item or Remove Batch still ends their lifecycle and queues permanent deletion.
- Preserve the existing dedicated Drop Shelf Clear confirmation at `360 × 130` logical pixels and the Edge Rail's local confirmation lifecycle. Use the canonical explanation: `Pinned items stay. External sources stay. DropCove-created temporary files are permanently deleted.`
- Keep Cancel as the initially focused Clear action. `Escape`, window close, stale empty confirmation, setup failure, Exit DropCove, and Cooperative Shutdown retain their established authority and safety ordering.
- Extend Recovery Backup as defined by ADR-0003 and ADR-0006: preserve `shelf.db`, existing WAL/SHM sidecars, and complete Managed File storage together before creating a clean database and empty active managed root. Any incomplete preservation or recreation remains fatal.
- Preserve Profile Data during update and uninstall. Retained and pending Managed Files remain under the profile; complete removal continues to require deleting the profile directory separately.
- Preserve availability classification through the existing path seam. Ready Managed Files are ordinary filesystem paths for restore, Manage Items, Shell Visuals, and drag-out. A missing owned target is a Missing Item for Shelf behavior and already absent for later cleanup.
- Reuse existing Shelf events and synchronization after successful `Ready` publication or lifecycle mutation. Do not add frontend-owned file state, browser-local persistence, a Pinia store, a settings field, or another durable projection.
- Preserve the existing source-order, exact-path duplicate, responsive sizing, final-item reset, display-state, Edge Rail no-activate, fullscreen, topmost, appearance, reduced-motion, accessibility, and resident-lifetime contracts.
- Keep package, crate, lockfile, Tauri, PE, About, installer, and uninstall metadata at `0.0.22` during implementation. Synchronize to `0.0.23` only in an approved release ticket after behavior acceptance.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Primary seam: the actual Windows Resident Instance with the real native OLE drop target, WebView2 Drop Shelf and expanded Edge Rail, owned test Profile Data, real managed files, and a real native Copy destination. This is the highest seam that proves OLE intake, Core Shelf State, SQLite, filesystem ownership, surfaces, drag-out, lifecycle consumption, and cleanup compose correctly.
- Use a controlled native OLE source plus real browser text selection where available. Exercise the data object and `CF_UNICODETEXT` format rather than invoking the Core mutation directly and calling that drag-in coverage.
- Use the existing Core Shelf State integration seam with real temporary SQLite databases and real temporary directories for deterministic permanent tests. Seed database/filesystem combinations to exercise `Staged`, promoted-before-`Ready`, `Ready`, missing, pending-deletion, and Recovery Backup states.
- Extend the existing intake, accepted-Copy, canceled-drag, persistence-rollback, Pin lifecycle, Remove Item, Remove Batch, Clear Temporary Items, sizing-reset, and recovery regressions. These are prior art for asserting authoritative snapshots and bytes after public Shelf operations rather than private helper choreography.
- Extend the native drop-eligibility regression seam to cover valid text, empty text, over-limit text, application-owned drag rejection, Copy-only offered effects, modal blocking, and representation precedence. Tests should assert eligibility/result behavior, not COM helper call counts.
- Add permanent behavior coverage for Unicode round trip, multiline content, whitespace-only content, URL-shaped text, UTF-8 exact bytes, no BOM, preserved line endings, exact `10 MiB` acceptance, one-byte-over rejection, timestamp naming, and collision suffixes.
- Add permanent lifecycle coverage for Temporary accepted Copy deletion, Pinned Copy retention, cancellation/rejection/pre-acceptance retention, Unpin then consumption, explicit removal of Pinned and Temporary Managed Files, Clear mixed ownership, confirmed-Missing cleanup, and final-item sizing/display reset.
- Assert external source safety with overlapping paths and mixed external/Managed File batches. Tests must prove external source bytes and directories remain unchanged after Copy consumption, Remove Item, Remove Batch, Clear, missing cleanup, cleanup retry, and recovery.
- Force a real sharing violation on Windows by holding the Managed File open with deletion-denying sharing flags. Verify the Shelf mutation commits, the owned file remains, the exact pending count is reported, and a later event/startup retry deletes the file and pending record after the handle closes.
- Cover crash windows by seeding or terminating at: before the `Staged` commit, after the `Staged` commit, after file promotion, and before the `Ready` commit. Startup reconciliation must yield either one complete `Ready` item or no visible item, with no untracked staging/final file.
- Exercise failure after `Staged` commit without process termination. Verify compensating removal, batch pruning, sizing restoration, warning behavior, and removal of staging/final remnants.
- Extend recovery regressions to preserve Managed File storage byte-for-byte with the corrupt database and present sidecars, retain earlier backup collision behavior, restore all moved artifacts after partial move failure, and fail fatally without modifying source evidence when full preservation cannot complete.
- Exercise restart, update, same-candidate reinstall, standard uninstall, and reinstall using owned Profile Data. Verify Ready Managed Files remain usable, pending deletion resumes, the uninstaller preserves profile bytes, and complete manual profile deletion remains outside installer behavior.
- Exercise the actual Drop Shelf and expanded Edge Rail surfaces for placeholder, accepted batch insertion, status feedback, Shell Visual, Pin/Unpin, Manage Items, item and batch Copy, Remove, Clear confirmation text, sizing, Hidden/EdgeDocked transitions, and Shake accepted/failed restoration.
- Verify Clear confirmation content and layout on both surfaces in Dark, Light, and Windows Contrast Theme while retaining the `360 × 130` native-window contract, safe default focus, Escape/window-close cancellation, modal interaction blocking, and no text clipping.
- Good permanent tests catch consumer-visible failures: corrupted text, wrong file bytes/name, visible staged item, lost retained item, leaked owned file, deleted external source, missing cleanup retry, false warning, partial batch, broken rollback, incomplete recovery evidence, wrong sizing/display state, or changed drag outcome.
- Do not add tests for copied command forwarding, SQL text, helper call order, source-code strings, CSS classes, non-empty collections, or mocked filesystem echoes. Assert authoritative Shelf state and actual filesystem/database outcomes.
- Run the existing frontend Node regressions, frontend typecheck/production build, cargo-xwin Windows release/test compilation, and the freshly produced native Windows regression executable directly.
- After implementation acceptance, synchronize version `0.0.23`, build one exact current-user Windows x64 NSIS candidate with cargo-xwin, run from installed embedded assets with the development server unavailable, and exercise startup, secondary activation, tray/hotkey access, Cooperative Shutdown, reinstall, uninstall, and Profile Data preservation.
- Record physical or policy-dependent limits—such as antivirus behavior, cross-integrity drag, actual Windows Contrast Theme, high/mixed DPI, and browser drag-source differences—as unobserved when not exercised. Do not infer them from deterministic tests.

## Out of Scope

- Website-image virtual-file intake, image format validation, raw SVG handling, Windows Attachment Services, Mark-of-the-Web behavior, browser acceptance, or any `0.0.24` capability.
- Multi-batch selection, selected-batch preparation, cross-batch path deduplication, Drop Shelf batch-selection keyboard semantics, or any `0.0.25` capability.
- HTTP or authenticated URL download, cookie/referrer forwarding, URL items, HTML/RTF files, bitmap clipboard capture, clipboard history, editable text items, or rich-text preservation.
- Generic virtual-file intake, arbitrary application-owned copies of ordinary filesystem drops, file/folder import, Move semantics, Recycle Bin cleanup, content hashing, shared backing files, reference counting, storage quotas, or user-selected Managed File locations.
- A new Settings option, manual cleanup command, pending-file management UI, storage browser, telemetry, background timer, filesystem watcher, cloud synchronization, accounts, plugins, named shelves, or non-Windows support.
- Multi-item text parsing, splitting text by line/selection, custom filename entry, content-derived filenames, encoding selection, newline normalization, text preview/editing, or clipboard paste commands.
- Changing Edge Rail placement, activation, fullscreen policy, Shake detector thresholds, Drop Shelf geometry, Manage Items geometry, virtualization thresholds, appearance palette, icon system, or update behavior.
- Automatic schema migration framework, compatibility alias, deprecated ownership field, or dual cleanup model. The active schema uses the optional Managed File relation as the only ownership authority.
- Creating implementation tickets, beginning implementation, synchronizing release metadata, packaging a candidate, accepting a release, or creating a Git commit as part of this specification step.

## Further Notes

`0.0.23` deliberately proves the Managed File lifecycle with plain text before browser virtual-image streams are introduced. Text provides the smallest complete non-filesystem input: one bounded payload, one deterministic output file, and the same Pin/Copy/Remove/Clear behavior later required by website images.

The testing strategy uses one primary end-to-end seam—the real Windows Resident—and one supporting deterministic seam at Core Shelf State with actual SQLite/filesystem state. No frontend component-test framework or generalized storage abstraction is required. The Managed File Store is a deep module because it hides cross-store ordering, crash reconciliation, deletion authority, and recovery behind the Core Shelf State interface.

The hardest implementation invariant is not file creation or deletion by itself. It is preserving one authoritative ownership record across SQLite and the filesystem without claiming a cross-resource transaction. `Staged` → `Ready` intake and committed Pending Managed File Deletion provide deterministic recovery for both directions.

The approved ticket DAG is published and ready for agent execution after explicit implementation approval. Release metadata synchronization remains isolated in Ticket 04, and no implementation or Git commit is authorized by this spec or ticket approval.