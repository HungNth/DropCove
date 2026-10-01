# 19: Reduce installed Release idle Working Set below 150 MB

**What to build:** Redesign DropCove's WinUI/native residency so the installed self-contained Release process remains below the existing 150 MB total Working Set target at the required 100 Shelf Batch / 1,000 Shelf Item scale, without weakening interaction, lifecycle, or latency contracts.

**Blocks:** 18: Pass the final DropCove V1 release gate.

**Blocked by:** 15: Add Compact and Expanded shelf modes.

**Status:** ready-for-agent

- [ ] The acceptance metric remains total process `WorkingSet64`; it is not replaced by private committed memory, private working set, installer size, or an explicitly trimmed value.
- [ ] The installed self-contained x64 Release EXE is measured from a fresh process with the canonical 100 Shelf Batch / 1,000 Shelf Item seed and the existing five-sample idle protocol.
- [ ] Profiling attributes the scale-dependent resident pages before architecture changes are selected, distinguishing managed models, Compact/Expanded presentation state, Edge Rail summaries/flyouts, native WinUI/Windows App SDK modules, icons/thumbnails, and shared pages.
- [ ] The chosen architecture removes or defers measured residency rather than hiding it behind `EmptyWorkingSet`, forced GC, GC configuration tuning without proof, manual process trimming, or a changed acceptance threshold.
- [ ] Compact, Expanded, Edge Rail, shake summon, drag-in/out, pin/remove/clear, persistence, and current-monitor behavior remain unchanged.
- [ ] Idle CPU remains at or below 0.1%, shelf-show latency remains p95 at or below 150 ms, and no periodic cursor polling or new idle work is introduced.
- [ ] The default installed-Release performance run records an average idle Working Set below 150 MB with 100 Shelf Batches / 1,000 Shelf Items; any miss continues to block Ticket 18.
- [ ] Automated tests remain presentation-independent where possible, while installed-EXE smoke proves shelf show, dismissal into Edge Rail, rail reopen, and management behavior after the residency redesign.

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
