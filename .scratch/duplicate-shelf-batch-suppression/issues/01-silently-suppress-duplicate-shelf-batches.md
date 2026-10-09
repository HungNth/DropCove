# 01: Silently suppress Duplicate Shelf Batches

**Parent specification:** [Duplicate Shelf Batch Suppression Specification](../spec.md)

**What to build:** Silently discard an incoming Duplicate Shelf Batch across the Drop Shelf and Edge Rail. Exact path-set matches must produce no new Shelf Batch, persistence, notification, refresh, scroll, growth, or geometry change, while partial-overlap and unsupported-input behavior remain distinct and unchanged.

**Blocked by:** None (can start immediately).

**Status:** needs-info

**Testing seam:** Use the existing manager acceptance boundary backed by a real temporary SQLite database for domain, persistence, restart, and concurrency behavior. Retain the existing shake session seam for restoration semantics; use a focused running-app smoke for Drop Shelf and Edge Rail wiring rather than introducing a test-only UI coordinator.

**Demo path:** Add a multi-item Shelf Batch, drop the same paths in reversed order and different casing on both the Drop Shelf and Edge Rail, and show that content, status, viewport, ordering, and bounds remain unchanged. Then drop a partially overlapping group and show that the complete new batch is added. Repeat a duplicate after restart and through shake-to-open from Hidden and EdgeDocked.

- [x] Incoming filtering and within-drop path deduplication run before duplicate-batch comparison, preserving first-seen source order for any batch that is created.
- [x] A non-empty incoming path set is duplicate when it exactly matches the current Shelf Items of any retained Shelf Batch using ordinal case-insensitive path-string equality with item order ignored.
- [x] Duplicate identity uses paths only. Name, file-versus-folder metadata, occurrence identity, creation time, pin state, and availability classification do not affect the result.
- [x] Comparison performs no separator rewriting, full-path canonicalization, link resolution, filesystem-object lookup, content hashing, persisted fingerprint, or duplicate ledger.
- [x] Matching uses each retained batch's current contents, including restored SQLite batches. Removing an item changes the set used for later comparison, and removing a batch removes its blocking effect.
- [x] A partial overlap creates a normal Shelf Batch containing every supported, within-drop-unique incoming path; overlapping paths are neither removed nor merged into an existing batch.
- [x] Existing Duplicate Shelf Batches already stored before the feature are preserved; no startup cleanup, migration, or automatic merge runs.
- [x] The drop result contract exposes an observable distinction between added, duplicate, and unsupported/empty outcomes. All production callers handle that distinction explicitly.
- [ ] Exact duplicate takes precedence after incoming filtering even when unsupported entries were skipped. It remains silent rather than showing the unsupported or skipped-item warning.
- [x] A duplicate outcome has no created Shelf Batch and zero accepted Shelf Items. The existing within-payload duplicate-path count remains separate and cannot signal a Duplicate Shelf Batch.
- [x] Duplicate comparison and insertion remain inside the existing asynchronous mutation gate. Concurrent identical drops create exactly one durable Shelf Batch and return duplicate outcomes for later contenders.
- [x] Persistence runs only for an added outcome. Duplicate and unsupported/empty outcomes do not create Shelf Items, write a Shelf Batch, alter durable sizing state, or change existing identities, creation times, pin states, or order.
- [ ] On the Drop Shelf, a duplicate is a silent no-op: no warning, status replacement, card refresh, viewport movement, ordering change, Automatic Shelf Growth, resize, or reposition occurs.
- [ ] On the Edge Rail, a duplicate is a silent no-op: no notification, batch refresh, expansion, resize, placement change, or other visible mutation occurs; normal drag cleanup and deferral completion still finish.
- [ ] A duplicate received after shake-to-open completes through the existing non-accepted restoration path without showing the unsupported warning, restoring the precise prior Hidden or EdgeDocked state.
- [ ] Added non-duplicate drops retain current refresh, newest-first viewport, skipped-unsupported reporting, persistence, growth, and accepted-shake behavior. Unsupported-only input retains its current warning and rejected-shake behavior.
- [x] Manager and real-SQLite tests cover single-item and multi-item duplicates, reversed order, casing differences, an older matching batch, partial overlap, metadata and lifecycle differences, mixed unsupported input, current-content changes, restart, and concurrent identical drops.
- [x] Obsolete tests that expect an identical later path set to create another Shelf Batch are replaced rather than retained under a second interpretation.
- [ ] A focused running-app smoke exercises exact duplicate and partial-overlap drops on both surfaces and shake restoration before formal installed Release qualification.
- [x] No database schema change, compatibility path, new package, cache, background worker, telemetry, setting, status string, or test-only application seam is introduced.

## Comments

### 2026-10-09 — Core and application cutover implemented; native verification blocked

- The real-SQLite regression failed before the fix because an exact duplicate created a Shelf Batch; it passed after the manager cutover.
- `DropDisposition` explicitly distinguishes Added, Duplicate, and Unsupported. `DuplicatePathCount` retains within-payload meaning. Duplicate comparison reuses the incoming case-insensitive path set before creating Shelf Item identities.
- Targeted manager suite passed 39/39; targeted persistence suite passed 47/47; final automated suite passed 232/232. The existing partial-overlap test remains valid; its misleading within-drop-only name was corrected rather than deleting its behavior coverage.
- Release build/publish succeeded with zero warnings/errors. The installed application smoke-launched with a visible `DropCove — Test profile` window.
- Standards review found no documented-standard breach; three non-actionable smell suggestions were rejected because the manager owns cross-batch identity, explicit disposition handling is required, and diagnostic counts already belong to the result record. Spec review reported no source-code finding.
- Native Drop Shelf/Edge Rail/shake acceptance is not claimed by the automated tests. The unchecked presentation criteria require the running-app evidence tracked by Ticket 02.
- Native automation did not deliver its real Explorer positive control (batch count stayed at 3), and the supplemental OLE source timed out after 240 seconds without an accepted-result log. Unchanged database counts are not reported as duplicate passes. No application defect is diagnosed from that harness failure.
- [Qualification evidence](../qualification.json) records the observed successes, failures, unobserved scenarios, artifact hashes, and cleanup. All owned helper processes, fixture profiles, diagnostic screenshots, and disposable driver sources were removed; the installed app is left visibly running for user inspection.
- Resume requires a reliable real native-drop interaction, with a successful non-duplicate positive control first, before certifying the unchecked Drop Shelf, Edge Rail, and shake criteria. The user explicitly approved committing the implementation while retaining blocked qualification; this is not release approval.


