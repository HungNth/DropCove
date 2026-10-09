# Duplicate Shelf Batch Suppression Specification

Status: ready-for-agent

## Problem Statement

DropCove currently creates a new Shelf Batch whenever a supported drop contains at least one usable path, even when another retained Shelf Batch already contains exactly the same paths. Repeating a drop therefore creates visually identical batches, duplicates persisted Shelf Item references, changes ordering and sizing, and makes the temporary workspace harder to scan.

The user wants a repeated batch to disappear silently rather than becoming another Shelf Batch. This must not weaken the existing rule that a path may appear in multiple distinct batches when the complete groups differ, and it must not misreport a duplicate batch as unsupported input.

## Solution

Suppress a **Duplicate Shelf Batch** at the shared Drop Shelf manager boundary. After unsupported entries and repeated paths within the incoming operation are removed, compare the remaining path strings as an unordered set against the current Shelf Items of every retained Shelf Batch. Equality uses ordinal case-insensitive string comparison and performs no path canonicalization.

When an exact match exists, return an explicit duplicate-batch outcome without creating or persisting a Shelf Batch. The application treats that outcome as a complete silent no-op: no status change, notification, refresh, scroll, ordering change, Automatic Shelf Growth, Edge Rail refresh, or other geometry change. A shelf summoned by shake-to-open restores its pre-shake Hidden or EdgeDocked state.

Partial overlap is not duplication. If no complete path-set match exists, DropCove creates the full new Shelf Batch from all supported, within-drop-unique paths in their first-seen source order, preserving the context of that drop operation.

## User Stories

1. As a user who drops the same single file twice, I want the second drop discarded, so that the Drop Shelf contains only one equivalent Shelf Batch.
2. As a user who repeats a multi-item drop, I want the repeated batch discarded, so that identical work groups do not clutter the shelf.
3. As a user who selects the same files in a different order, I want DropCove to recognize the same batch, so that source enumeration order does not create a duplicate.
4. As a Windows user, I want path casing differences ignored when batches are compared, so that casing variations do not create visually identical batches.
5. As a user, I want a duplicate compared against every retained Shelf Batch, so that an older matching batch still prevents another copy.
6. As a user, I want duplicate suppression to work after restarting DropCove, so that restored batches remain authoritative.
7. As a user, I want the same duplicate rule on the Drop Shelf and Edge Rail, so that the result does not depend on the surface receiving the drop.
8. As a user, I want duplicate identity based on the current Shelf Items in a batch, so that comparison reflects what the shelf contains now.
9. As a user who removes an item from a Shelf Batch, I want later drops compared with the remaining items, so that hidden historical contents do not block a useful new batch.
10. As a user who removes an entire Shelf Batch, I want its former path set to stop blocking later drops, so that deleted shelf state is not remembered as hidden history.
11. As a user, I want a partially overlapping drop to create a complete new Shelf Batch, so that DropCove preserves which files were added together.
12. As a user, I want a path to remain usable in multiple distinct Shelf Batches, so that partial overlap does not become global item deduplication.
13. As a user, I want DropCove not to merge a new drop into an existing Shelf Batch, so that each non-duplicate batch continues to represent one drop operation.
14. As a user, I want DropCove not to remove overlapping paths from a non-duplicate incoming batch, so that the new batch keeps its complete drop context.
15. As a user, I want repeated paths inside one incoming drop removed before batch comparison, so that accidental repetition in the payload does not defeat duplicate detection.
16. As a user, I want unsupported entries removed before batch comparison, so that only usable Path References determine the incoming batch.
17. As a user who drops a duplicate batch together with unsupported entries, I want the operation to remain silent, so that duplicate suppression does not produce a misleading skipped-item message.
18. As a user who drops only unsupported entries, I want the existing unsupported-input warning retained, so that invalid input is still distinguishable from a duplicate batch.
19. As a user who drops a new batch containing supported and unsupported entries, I want the supported paths added and the existing skipped-item warning retained, so that non-duplicate mixed input keeps its current behavior.
20. As a user, I want item names ignored for duplicate identity, so that display metadata cannot create duplicate path groups.
21. As a user, I want file-versus-folder metadata ignored for duplicate identity, so that one path has one identity rule within this feature.
22. As a user, I want Pinned Item and Temporary Item state ignored for duplicate identity, so that retention state does not permit an equivalent path group to be added again.
23. As a user, I want Available, Unavailable, and Missing Item classifications ignored for duplicate identity, so that availability changes do not create duplicate batches.
24. As a user, I want a duplicate drop to leave the current status surface unchanged, so that the application remains silent.
25. As a user, I want a duplicate drop not to refresh Shelf Batch cards, so that a no-op does not disturb the visible surface.
26. As a user, I want a duplicate drop not to move the batch viewport, so that my current scroll position is preserved.
27. As a user, I want a duplicate drop not to reorder existing Shelf Batches, so that the newest-first sequence changes only when content is added.
28. As a user, I want a duplicate drop not to resize or reposition the Drop Shelf, so that Automatic Shelf Growth responds only to a newly created Shelf Batch.
29. As an Edge Rail user, I want a duplicate drop not to refresh, expand, resize, or notify the rail, so that the rail remains unchanged.
30. As a user who summoned the Drop Shelf from Hidden with shake-to-open, I want a duplicate drop to restore Hidden state, so that a discarded batch leaves no lasting presentation change.
31. As an Edge Rail user who summoned the Drop Shelf with shake-to-open, I want a duplicate drop to restore EdgeDocked state, so that the rail workflow returns to where it started.
32. As a user, I want a successfully created non-duplicate batch to retain the existing shake-to-open behavior, so that I can inspect newly added content near the cursor.
33. As a user, I want a duplicate drop not to create persisted Shelf Items, so that restart cannot reveal a batch that was silently discarded.
34. As a user, I want existing Shelf Batch identities, creation times, item identities, and ordering unchanged after a duplicate drop, so that a no-op is durable and observable as no change.
35. As a user performing two identical drops concurrently, I want at most one new Shelf Batch, so that timing cannot bypass duplicate suppression.
36. As a user, I want a same-path comparison to use the path strings DropCove receives and stores, so that this feature does not unexpectedly reinterpret paths.
37. As a user, I want differently written path strings that require separator rewriting, full-path resolution, link resolution, or filesystem identity lookup to remain distinct, so that duplicate suppression does not guess beyond the established Path Reference model.
38. As an existing user with duplicate Shelf Batches already stored, I want the update not to delete or merge them automatically, so that installation cannot silently remove retained references.
39. As a user, I want DropCove never to copy, move, rename, overwrite, or delete source filesystem objects while suppressing duplicates, so that the feature remains reference-only.
40. As a performance-conscious user, I want duplicate detection to remain responsive at the established shelf scale, so that preventing clutter does not slow normal drag-in.
41. As a maintainer, I want one manager boundary to own duplicate identity and mutation ordering, so that the Drop Shelf and Edge Rail cannot diverge.
42. As a maintainer, I want duplicate, unsupported, and created outcomes represented explicitly, so that callers cannot confuse a silent duplicate with an error.
43. As a maintainer, I want duplicate behavior tested through the existing manager and real SQLite seam, so that tests prove production state and persistence behavior without a new persistence abstraction.
44. As a maintainer, I want the existing shake session seam retained, so that duplicate restoration uses the established state machine rather than adding a second shake workflow.
45. As a maintainer, I want obsolete tests that permit an identical later batch replaced by the new contract, so that the suite has one authoritative rule.
46. As a maintainer, I want the complete existing automated suite and installed Release smoke to remain green, so that duplicate suppression does not regress drag-in, persistence, sizing, Edge Rail, or shake behavior.

## Implementation Decisions

- Duplicate suppression is owned by `DropShelfManager`, the existing shared insertion boundary used by both the Drop Shelf and Edge Rail. UI callers do not implement their own comparisons.
- The manager first applies the existing incoming-path rules: unsupported or unusable entries are skipped, repeated incoming path strings are deduplicated with ordinal case-insensitive equality, and first-seen source order is retained.
- An incoming batch is duplicate when its resulting non-empty set of path strings has the same count and the same ordinal case-insensitive members as the current Shelf Items of any retained Shelf Batch. Item order is ignored only for duplicate identity.
- Comparison uses the path strings supplied and stored by DropCove. It does not trim or rewrite separators, call full-path normalization, resolve short names, links, hard links, junctions, casing from the filesystem, or filesystem-object identifiers.
- Only paths participate in duplicate identity. Item name, file-versus-folder metadata, occurrence identity, creation time, pinned lifecycle state, and availability classification are ignored.
- The comparison uses each retained batch's current Shelf Items. No original-drop fingerprint, deleted-item history, tombstone, or historical duplicate ledger is stored.
- A complete match against any retained batch suppresses the incoming batch. Matching is not limited to the newest Shelf Batch or current application session.
- Partial overlap creates a normal Shelf Batch containing every supported, within-drop-unique incoming path. Existing paths are not removed from it, and it is not merged into a retained batch.
- Empty supported input remains rejected rather than becoming an empty duplicate. Empty Shelf Batches remain invalid and continue to be pruned by existing lifecycle behavior.
- Duplicate comparison happens before new Shelf Item identities, Shelf Batch identity, or creation time are committed and before any persistence call. A discarded batch leaves existing in-memory state unchanged.
- The asynchronous production path keeps duplicate comparison and insertion inside the manager's existing mutation gate. Concurrent identical drops therefore serialize: one may create a batch, and later contenders observe it and return duplicate.
- No database schema, unique index, content fingerprint, migration, compatibility reader, or cleanup pass is added. Duplicate identity is derived from the manager's current retained batches.
- Existing duplicate batches already persisted before this feature remain untouched. The rule prevents new duplicates only.
- `DropAcceptance` gains an explicit disposition that distinguishes three outcomes: a new batch was added, the incoming batch duplicated a retained batch, or no supported batch could be formed. All production callers switch exhaustively on that disposition.
- For an added outcome, `Batch` is the newly created Shelf Batch and `AcceptedCount` is its Shelf Item count. Existing skipped-unsupported reporting and within-payload duplicate counting remain unchanged.
- For a duplicate outcome, `Batch` is absent and `AcceptedCount` is zero because no Shelf Item was accepted into DropCove. Skipped-unsupported and within-payload duplicate counts may remain available for diagnostics, but application surfaces do not report them for this silent no-op.
- For an unsupported/empty outcome, `Batch` is absent and `AcceptedCount` is zero. Existing unsupported-input presentation remains governing.
- The existing within-payload duplicate counter keeps only its current meaning and is renamed to make its path-level scope explicit. It is never reused to signal a Duplicate Shelf Batch.
- Persistence is invoked only for the added outcome. Duplicate and unsupported/empty outcomes perform no database mutation and cannot create or update durable sizing state.
- The shared storage adapter continues to convert Windows storage items into incoming domain items once, then delegates to the manager. No source-specific duplicate rule or event-coalescing token is introduced.
- The event that announces a newly accepted Shelf Batch is raised only for the added outcome. Duplicate suppression therefore cannot trigger Automatic Shelf Growth or any other accepted-batch subscriber.
- The Drop Shelf handler maps an added outcome to the existing success path. It refreshes cards, restores the newest-first viewport, applies existing skipped-item reporting, and completes shake as accepted.
- The Drop Shelf handler maps a duplicate outcome to a silent early return. It completes a summoned shake as not accepted so the established shake restoration path runs, but it does not show the unsupported warning, hide or replace current status, refresh cards, scroll, resize, or grow the shelf.
- The Drop Shelf handler maps unsupported/empty input to the existing warning and rejected-shake behavior.
- The Edge Rail handler maps a duplicate outcome to a silent early return. It performs no notification and does not call the rail refresh path; normal drag-inside cleanup and deferral completion still occur.
- A duplicate drop preserves all current presentation state, including batch order, realized cards, open scroll position, shelf bounds, Manual Height Override, Automatic Shelf Growth state, Edge Rail bounds, and status content.
- `ShakeSessionCoordinator` keeps its existing accepted/rejected state machine. A duplicate outcome uses the existing non-accepted completion path because no new Shelf Batch was created; no third shake state or second restoration mechanism is added.
- Persistence failures remain exceptions only when a new batch is being written. A duplicate no-op cannot surface a persistence failure because it performs no write.
- Existing drag-out, pinning, removal, availability, sizing, thumbnail, popup, and source-filesystem ownership behavior remains unchanged.
- No new NuGet dependency, background worker, cache, database index, telemetry event, setting, confirmation dialog, toast, or status string is introduced.
- The implementation remains within the existing CLI-built WinUI 3 toolchain and keeps all changes uncommitted until explicit user approval.

## Testing Decisions

- Good permanent tests assert externally observable outcomes: created versus duplicate versus unsupported disposition, retained Shelf Batch contents and order, durable restart state, concurrent behavior, and presentation effects. They do not assert private comparator calls, SQL text, exact allocation counts, event forwarding, source-code strings, or mock echoes.
- The primary automated seam is `DropShelfManager.AcceptDropAsync` backed by a real temporary SQLite database. It is the highest existing seam that covers incoming filtering and within-drop deduplication, duplicate identity, mutation serialization, persistence, retained current contents, and restart behavior without a new abstraction.
- Existing synchronous manager acceptance tests are prior art for fast path-set and source-order behavior. The obsolete expectation that an identical later path group creates another batch is replaced rather than retained under a second interpretation.
- Existing real-SQLite persistence tests are prior art for immediate commit, restart restoration, batch ordering, item identity, pinned lifecycle, availability restoration, and isolated temporary databases.
- Existing `ShakeSessionCoordinator` tests are prior art for accepted drops keeping a summoned shelf open and non-accepted outcomes restoring prior state. The coordinator contract remains unchanged; installed behavior proves that the duplicate disposition is mapped to its non-accepted path without an unsupported warning.
- Manager tests cover exact single-item and multi-item duplicates, reversed order, path-casing differences, and a match against an older rather than newest retained Shelf Batch.
- Manager tests cover partial overlap and verify that the complete new batch is created in first-seen source order, including paths that also occur in another batch.
- Manager tests cover metadata differences, pinned state, and Available, Unavailable, and Missing classifications and verify that path-set equality alone determines duplication.
- Manager tests cover repeated paths and unusable entries inside the incoming payload and verify that comparison occurs after the existing filtering and within-drop deduplication rules.
- Manager tests cover duplicate supported paths mixed with unsupported entries and verify the duplicate disposition, absent batch, zero accepted count, unchanged retained batches, and retained diagnostic counts without treating the operation as unsupported.
- Manager tests cover unsupported-only input separately and preserve its rejected disposition and existing counts.
- Current-content tests create a batch, remove one item, and verify that later comparison uses the remaining set. They also remove a whole batch and verify that its former set no longer blocks a new batch.
- Persistence tests reopen the manager after a duplicate outcome and verify that batch count, batch order, batch identities, creation times, item identities, paths, folder flags, and pin states remain exactly the retained state from before the duplicate drop.
- A concurrent async test submits identical drops through one manager and verifies exactly one added outcome, duplicate outcomes for the rest, and one durable Shelf Batch after reopening the database.
- No permanent test is added solely to prove that the Drop Shelf or Edge Rail forwarded a disposition to the manager. Installed self-contained Release smoke is the authority for UI wiring and native drag behavior.
- Installed Drop Shelf smoke creates a retained batch, moves the viewport away from the newest-first position where practical, drops an exact duplicate, and verifies no new card, no refresh-visible disturbance, no scroll change, no status change, and unchanged window bounds.
- Installed Edge Rail smoke drops an exact duplicate and verifies no new row, notification, refresh-visible disturbance, expansion, resize, or placement change.
- Installed shake-to-open smoke starts once from Hidden and once from EdgeDocked, drops an exact duplicate, and verifies restoration to the precise pre-shake state without the unsupported-input warning.
- Installed smoke also drops a partial-overlap batch and verifies normal creation, full incoming contents, newest-first placement, and unchanged existing skipped-item reporting.
- Geometry smoke places the shelf at a row-growth boundary and verifies that a duplicate drop does not trigger Automatic Shelf Growth while a subsequent non-duplicate batch still follows the established growth policy.
- The established scale scenario remains governing. Duplicate comparison must remain responsive with `100` Shelf Batches and `1,000` Shelf Items and must not weaken existing working-set, idle CPU, interaction, or shelf-show latency gates.
- The complete existing automated suite must remain green after obsolete duplicate-permitting assertions are replaced. Drag-in, persistence, shake, sizing, Edge Rail, pinning, removal, availability, drag-out, popup, and visual-coordinator behavior must not be re-pinned to implementation details.

## Out of Scope

- Retroactively deleting, merging, or rewriting Duplicate Shelf Batches already stored before this feature.
- Global Shelf Item deduplication across distinct batches.
- Removing overlapping paths from a partially overlapping incoming batch.
- Appending new paths to, merging into, or otherwise mutating an existing Shelf Batch during drag-in.
- Remembering deleted batches or original batch contents as hidden duplicate history.
- Path canonicalization, separator rewriting, relative-to-absolute conversion, short-name expansion, symbolic-link or junction resolution, hard-link detection, filesystem-object identity, or content hashing.
- Treating two different paths to the same filesystem object as duplicate.
- A database uniqueness constraint, persisted path-set fingerprint, duplicate ledger, schema migration, or cleanup migration.
- User-visible duplicate warnings, informational messages, confirmations, sounds, badges, toasts, or settings.
- A general redesign of `DropAcceptance`, drag status messages, shake detection, Automatic Shelf Growth, Edge Rail presentation, or drop feedback beyond the explicit duplicate disposition and mappings required here.
- Changing within-drop path deduplication, first-seen source order, unsupported-item classification, or Copy-only drag acceptance.
- Changing Temporary Item, Pinned Item, Missing Item, Unavailable Item, Path Reference, Successful Drag-Out, or source-filesystem ownership semantics.
- New telemetry, background scanning, caches, polling, NuGet packages, persistence interfaces, UI test projects, or alternate drag-in pipelines.

## Further Notes

- The canonical terms **Shelf Batch**, **Shelf Item**, **Path Reference**, and **Duplicate Shelf Batch** are defined in `CONTEXT.md`. This specification uses those terms rather than collection, folder, attachment, or duplicate item.
- Current product documentation already records exact path-set suppression, ordinal case-insensitive string comparison, no canonicalization, silent presentation behavior, and shake restoration. Implementation must make runtime behavior match that documented contract.
- The global mouse-hook ADR remains governing. A duplicate batch is a supported payload but not an accepted state mutation; mapping it to the existing non-accepted shake completion restores the previous shelf state without changing the hook or shake heuristic.
- No new ADR is required. The feature changes a reversible product rule inside the existing manager, persistence, and shake boundaries and introduces no new architectural dependency or storage model.
- The selected testing seams are the existing manager-plus-real-SQLite boundary for automated behavior, the existing shake coordinator for its unchanged restoration contract, and installed self-contained Release smoke for native UI mapping. No test-only seam is introduced.
- All repository changes remain uncommitted until the user explicitly authorizes a commit.
