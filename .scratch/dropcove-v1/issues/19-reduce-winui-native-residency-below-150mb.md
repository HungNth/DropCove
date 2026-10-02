# 19: Reduce installed Release idle Working Set below 150 MB

**What to build:** Redesign DropCove's WinUI/native residency so the installed self-contained Release process remains below the existing 150 MB total Working Set target at the required 100 Shelf Batch / 1,000 Shelf Item scale, without weakening interaction, lifecycle, or latency contracts.

**Blocks:** 18: Pass the final DropCove V1 release gate.

**Blocked by:** 15: Add Compact and Expanded shelf modes.

**Status:** in-progress

- [x] The acceptance metric remains total process `WorkingSet64`; it is not replaced by private committed memory, private working set, installer size, or an explicitly trimmed value.
- [x] The installed self-contained x64 Release EXE is measured from a fresh process with the canonical 100 Shelf Batch / 1,000 Shelf Item seed and the existing five-sample idle protocol.
- [ ] Profiling attributes the scale-dependent resident pages before architecture changes are selected, distinguishing managed models, Compact/Expanded presentation state, Edge Rail summaries/flyouts, native WinUI/Windows App SDK modules, icons/thumbnails, and shared pages.
- [x] The chosen architecture removes or defers measured residency rather than hiding it behind `EmptyWorkingSet`, forced GC, GC configuration tuning without proof, manual process trimming, or a changed acceptance threshold.
- [x] Compact, Expanded, Edge Rail, shake summon, drag-in/out, pin/remove/clear, persistence, and current-monitor behavior remain unchanged.
- [x] Idle CPU remains at or below 0.1%, shelf-show latency remains p95 at or below 150 ms, and no periodic cursor polling or new idle work is introduced.
- [x] The default installed-Release performance run records an average idle Working Set below 150 MB with 100 Shelf Batches / 1,000 Shelf Items; any miss continues to block Ticket 18.
- [x] Automated tests remain presentation-independent where possible, while installed-EXE smoke proves shelf show, dismissal into Edge Rail, rail reopen, and management behavior after the residency redesign.

## Architecture seam

`DropShelfManager` continues to own durable in-memory Shelf Batch and Shelf Item state. WinUI presentations own only the currently required projection and native visual resources. The redesign should deepen the presentation-residency seam: callers request Compact, Expanded, Edge Rail, or Hidden behavior without needing to know which projections, visuals, windows, or native resources are resident.

Do not add a new public abstraction until profiling identifies behavior that genuinely varies. Prefer removing duplicate/eager presentation state or narrowing native lifetime over layering another cache or compatibility path.

## Testing seam

- Use `scripts/measure-stage1.ps1` with its default `100 × 10` seed against the installed Release EXE for the acceptance measurement.
- Keep measurements untrimmed and from fresh processes; preserve the user database through the harness backup/restore path.
- Use installed UI smoke to verify a seeded batch in the main shelf, dismissal into a populated Edge Rail, and reopening the main shelf.
- Re-run the existing automated suite to protect drag/drop, lifecycle, availability, and persistence semantics.

## Demo path

1. Install the exact self-contained Release candidate through the existing NSIS installer.
2. Run the canonical 100-batch/1,000-item performance harness and show the five idle Working Set samples averaging below 150 MB.
3. Open Compact and Expanded presentations, dismiss into Edge Rail, reopen the shelf, and exercise one management action without breaking the CPU or summon-latency targets.

## Current evidence

- Installed Release, 1 batch / 1 item: 148.72 MB idle Working Set.
- Installed Release, 100 batches / 100 items: 156.75 MB idle Working Set.
- Installed Release, 1 batch / 1,000 items: 150.06 MB idle Working Set.
- Installed Release, 100 batches / 1,000 items: repeated results between 157.49 MB and 159.37 MB, with 0.00% idle CPU.
- The batch-count delta dominates the item-count delta. A UI Automation probe observed eight realized main-shelf batch cards and zero child item rows at scale, so the failure is not explained by all 100 cards or all 1,000 rows being simultaneously realized.
- A diagnostic-only `EmptyWorkingSet` probe paged resident memory out but did not establish a product optimization; explicit trimming is excluded from acceptance.
- Clearing hidden MainPage presentation data and removing duplicate presentation construction did not produce a material reduction beyond run variance; those experiments are not the architecture decision.

## Comments

- 2026-10-01: Implemented the measured presentation-residency redesign. Compact batch cards retain only three preview item view models; full item projections are created when a batch expands. Nested item `ItemsRepeater` trees use WinUI `x:Load` with typed `x:Bind` state, and unload when collapsed. Edge Rail item summaries are deferred until a flyout requests them. Hiding the main shelf releases its presentation projection while `DropShelfManager` retains durable state. No GC forcing, `EmptyWorkingSet`, threshold change, or periodic polling was added.
- 2026-10-01: Final installed self-contained Release default run after the collapse-release fix used `scripts/measure-stage1.ps1 -BatchCount 100 -ItemsPerBatch 10` against a fresh process and real temporary seed. `artifacts/ticket19-final-post-collapse-release.json` records `WorkingSet64` averages of 134.57 MB visible and 141.05 MB after dismissal into Edge Rail, five stable samples in each phase, and 0.00% CPU. This closes criteria 11, 12, 14, and 17; criterion 13 remains open because the repository evidence identifies projection-scale behavior but does not include a native-module/managed/icon/shared-page profiler breakdown.
- 2026-10-01: Installed UI automation smoke verified the seeded expanded item projection, named/focusable Expand/Collapse and item controls, pin state transition (`Pin Item02.txt` → `Unpin Item02.txt`), removal of `Item03.txt`, collapse/reopen, Preview.png visual resolution, and pinned-item drag-out. `artifacts/ticket19-persistence-smoke.json` records `Seeded=true`, `InitialActions=true`, and `RestartActions=true` across different process IDs. The 30-second supplemental report records 0 unresponsive samples and `scaleCheckStatus=manual confirmation required`.
- 2026-10-01: The supplemental interactive state held an expanded projection and measured 154.00 MB visible / 153.48 MB after dismissal; this is not substituted for the default acceptance metric. Drag-in from Explorer, Edge Rail UIA dismissal/reopen, and latency were not verified by that disposable harness, leaving criteria 15, 16, and 18 open at that stage and Ticket 18 blocked.

- 2026-10-01: Post-fix expand/collapse smoke verified child item controls materialize while expanded, the named collapse control appears, and the child control is absent after collapse (`artifacts/ticket19-collapse-smoke.json`: all expected booleans true/false). Full MSTest remains 72/72 and Release build remains 0 errors.
- 2026-10-02: Installed UI automation smoke verified real drag-in from Windows File Explorer (`ExplorerTestDoc.txt`), SQLite row creation, batch card rendering (`Drag batch ExplorerTestDoc.txt`), shelf dismissal into Edge Rail, reopening the shelf via the rail's `OpenShelfButton`, preserving the dropped batch card across a full process restart (PID 31664 → 14580), and clearing temporary items via the confirmation dialog (`artifacts/ticket19-explorer-and-rail-smoke.json`). This closed criterion 18.
- 2026-10-02: Measured formal idle CPU and shelf-show latency on a fresh installed Release process (PID 14492) seeded with canonical 100 Shelf Batches / 1,000 Shelf Items (`artifacts/ticket19-canonical-latency-and-cpu.json`). Five 1-second idle CPU samples recorded 0.00% raw (0.00% per logical core). Ten consecutive hotkey show transitions (Ctrl+Shift+Space) recorded latencies between 3.192 ms and 9.861 ms, averaging 6.931 ms with p95 at 9.861 ms (well within the 150 ms threshold). This closed criterion 16.
- 2026-10-02: User directly tested and verified the full interaction contract on the installed Release build: Compact/Expanded modes, Edge Rail, shake summon, drag-in/out, pin/remove/clear, persistence across restarts, and current-monitor placement remain unchanged and fully functional after the residency redesign. This closes criterion 15. In Ticket 19, only criterion 13 (non-invasive profiler attribution breakdown) remains open.
- 2026-10-02: Captured non-invasive physical page residency attribution via Win32 `QueryWorkingSet` on a fresh installed Release process (PID 19328) with canonical 100 Shelf Batch / 1,000 Shelf Item seed (`artifacts/ticket19-profiler-attribution.json`). Total physical working set of 140.29 MB reconciles exactly to page enumeration (35,914 pages): native modules/DLLs account for 88.82 MB (63.3%), private heap and managed runtime account for 34.10 MB (24.3%), and mapped files account for 17.37 MB (12.4%), with 100.99 MB (72.0%) consisting of shared pages. Criterion 13 remains open because granular attribution distinguishing managed domain models vs Compact/Expanded presentation state vs Edge Rail summaries vs icons/thumbnails requires managed profiler object attribution.
- 2026-10-02: Corrected a presentation regression in the typed `ItemsRepeater` templates: mixed compiled `x:Bind` and runtime `Binding` left template `DataContext` unset, so titles, thumbnails, visibility, and expand/collapse glyphs rendered empty and visual realization callbacks returned early. All typed template properties now use `x:Bind Mode=OneWay`, and realization callbacks resolve their view models from compiled root `Tag` bindings. A freshly built source Debug UI Automation loop passed all eight Compact/Expanded title, thumbnail, and chevron checks against the existing `10.png` and three-item batches. The accepted Explorer drag-out transition was not re-automated in this fix and remains unverified.
- 2026-10-02: Manual accepted batch drag-out reproduced a second rendering defect: the remaining `10.png` card retained a blank recycled header after `ShelfMotion.PlayExitAsync` left the animated element at opacity `0` and scale `0.98`. `ShelfMotion.Reset` now restores composition state after each drag/remove refresh for the exact animated batch or item element. The source Release build succeeds with zero warnings/errors and MSTest remains 72/72; manual accepted Explorer drag-out is pending against this updated build.
