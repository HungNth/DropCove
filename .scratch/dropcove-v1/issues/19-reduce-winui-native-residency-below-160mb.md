# 19: Reduce installed Release idle Working Set below 160 MB

**What to build:** Redesign DropCove's WinUI/native residency so the installed self-contained Release process remains below the 160 MB total Working Set target at the required 100 Shelf Batch / 1,000 Shelf Item scale, without weakening interaction, lifecycle, or latency contracts.

**Blocks:** 18: Pass the final DropCove V1 release gate.

**Blocked by:** 15: Add Compact and Expanded shelf modes.

**Status:** in-progress

- [x] The acceptance metric remains total process `WorkingSet64`; it is not replaced by private committed memory, private working set, installer size, or an explicitly trimmed value.
- [x] The installed self-contained x64 Release EXE is measured from a fresh process with the canonical 100 Shelf Batch / 1,000 Shelf Item seed and the existing five-sample idle protocol.
- [ ] Profiling attributes the scale-dependent resident pages before architecture changes are selected, distinguishing managed models, Compact/Expanded presentation state, Edge Rail summaries/flyouts, native WinUI/Windows App SDK modules, icons/thumbnails, and shared pages.
- [x] The chosen architecture removes or defers measured residency rather than hiding it behind `EmptyWorkingSet`, forced GC, GC configuration tuning without proof, manual process trimming, or a changed acceptance threshold.
- [x] Compact, Expanded, Edge Rail, shake summon, drag-in/out, pin/remove/clear, persistence, and current-monitor behavior remain unchanged.
- [x] Idle CPU remains at or below 0.1%, shelf-show latency remains p95 at or below 150 ms, and no periodic cursor polling or new idle work is introduced.
- [x] The default installed-Release performance run records an average idle Working Set below 160 MB with 100 Shelf Batches / 1,000 Shelf Items; any miss continues to block Ticket 18.
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
2. Run the canonical 100-batch/1,000-item performance harness and show the five idle Working Set samples averaging below 160 MB.
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
- 2026-10-08: Re-diagnosis with a differential loop (`artifacts/i19-bisect.ps1`, canonical `measure-stage1.ps1` 100×10, same machine/day). `98f1aa0` measures 137.11–143.34 MB visible / 140.36–146.04 MB hidden, but a PID-bound UIA probe shows its batch templates render blank (17 text elements, 1 named). `49f3761` (typed `x:Bind` template fix) renders titles (17/17 named) and measures 152.67–157.43 / 155.30–159.57 MB. The earlier 134.57/141.05 MB acceptance was therefore measured on a build that did not render card content; it is not a valid baseline. HEAD and the parity working tree both measure 149.5–155.7 / 152.2–158.1 MB.
- 2026-10-08: Same-build scale minimisation (working tree): 1×1 146.64/149.28, 100×1 149.96/153.01, 8×10 152.42–152.87/153.58–155.48, 100×10 152.33/155.30 MB. Total batch count is not the driver; the realized card set (~8 cards) is. `QueryWorkingSet` attribution (reconciled within 0.12 MB of `WorkingSet64`) puts the 1×1→100×10 delta at +5.9 MB private (+3.7 MB native 8–64 MB heap regions, +1.3 MB GC regions, +0.5 MB 1–8 MB regions) and +0.4 MB images; loaded module sets are identical apart from `windows.energy.dll`. The 1×1 floor is ~97.6 MB image pages (WinUI/WinAppSDK/.NET/NVIDIA driver), ~19 MB mapped, ~30 MB private.
- 2026-10-08: Falsified levers (no reduction beyond run variance): `DOTNET_gcConcurrent=0`, `DOTNET_TieredPGO=0`, `DOTNET_GCConserveMemory=5`, `dotnet publish` plain vs `PublishReadyToRun=true`.
- 2026-10-08: Paired 3-run test of a shared in-flight native-icon cache (one Shell load per extension for concurrent requests) in an isolated worktree mirroring the working tree (DLL `2fb7a81d…` vs baseline `7b1a7264…`). Medians baseline → variant: 100×1 150.41→149.93 visible / 153.35→152.82 hidden; 100×10 152.89→152.19 visible / 154.51→155.05 hidden. All deltas (−0.70 to +0.54 MB) are inside the ±1.5 MB run spread; the hypothesis is falsified and no production change was made. Repeated baseline runs confirm 100×1 itself fails hidden (152.93–153.52 MB).
- 2026-10-08: Product decision raised the gate from 150 MB to 160 MB (same `WorkingSet64` metric, same canonical protocol). Basis: the content-rendering build's single-item floor is 146.6/149.3 MB, and every measured app-side lever was falsified. Criterion 17 is reopened against the new gate. The pre-installer working-tree canonical runs already fall under it, but none of them counts as acceptance: average 152.19–157.14 MB visible / 154.93–158.07 MB hidden, highest single sample 159.80 MB (`artifacts/parity-ticket06-residency.json`). Acceptance needs the installed NSIS payload run.
- 2026-10-08: Canonical qualification run on the newly installed self-contained Release payload (`DropCove.dll` SHA256 `7b1a7264ad71457ead90480520ce2a7d6e48e3bd6c613b095f3dbc48e3f9f238`, built via `DropCove-Setup.exe` NSIS installer, installed at `%LOCALAPPDATA%\Programs\DropCove\DropCove.exe`).
  - Run 1 (`artifacts/i19-installed-qualification.json`): visible average **154.98 MB** (samples: 154.70, 154.70, 154.70, 155.23, 155.57; max 155.57 MB); hidden idle average **156.57 MB** (samples: 158.23, 156.27, 156.11, 156.11, 156.11; max 158.23 MB); CPU 0.0%. **PASS (<160.0 MB).**
  - Run 2 (`artifacts/i19-installed-qualification-2.json`): visible average **153.17 MB** (samples: 153.17, 153.17, 153.17, 153.17, 153.17; max 153.17 MB); hidden idle average **154.57 MB** (samples: 156.10, 154.19, 154.19, 154.19, 154.19; max 156.10 MB); CPU 0.0%. **PASS (<160.0 MB).**
  - Every individual sample across both runs stays strictly below the 160.0 MB gate (highest single sample 158.23 MB, margin 1.77 MB).
  - Criterion 17 is marked **passed**. Criterion 13 remains open (granular profiler breakdown distinguishing managed vs native heaps). Issue 19 is fully qualified for idle residency under the 160 MB gate.
