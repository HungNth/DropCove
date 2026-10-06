# Native Residency Diagnosis

Date: 2026-10-06

## Decision boundary

No residency implementation is selected yet. Current evidence proves a release failure but does not attribute the current resident pages strongly enough to justify an architecture change. The next step must measure the current build at the same lifecycle checkpoints before changing production ownership or caching.

## Current failing evidence

The current self-contained x64 Release candidate was exercised with an isolated profile containing exactly 100 Shelf Batches and 1,000 Shelf Items at 96 DPI. The measurement used total process `WorkingSet64`, no forced GC, no `EmptyWorkingSet`, no process trimming, no UI Automation peer realization inside the 30-cycle loop, production `WM_HOTKEY` show, and real owned-foreground `Esc` dismissal.

Evidence: [`artifacts/bulk-pinning-scale.json`](../../artifacts/bulk-pinning-scale.json).

- Fresh visible samples: `153.2461 MB` each; gate `<150 MB` fails before cycling.
- First post-dismissal samples: `156.2695–156.6836 MB`; gate fails before cycling.
- Peak settled sample during 30 cycles: `182.5195 MB`.
- Post-30-cycle settled sample: `178.8672 MB`.
- Current visible-to-peak increase: `29.2734 MB`.
- Current visible-to-post-30 increase: `25.6211 MB`.
- Idle CPU: `0.0%`; pass against `<=0.1%`.
- Queue-to-visible show latency p95: `14.3475 ms`; pass against `<=150 ms`.

The cleanup harness timed out waiting for graceful process exit and force-stopped only its owned test process. That warning is recorded separately and is not a passing shutdown claim or an explanation for the already captured residency samples.

## Prior passing and attribution evidence

The earlier accepted candidate recorded `134.57 MB` visible and `141.05 MB` after dismissal on the same canonical scale. Evidence: [`artifacts/ticket19-final-post-collapse-release.json`](../../artifacts/ticket19-final-post-collapse-release.json).

The current fresh visible result is `18.6761 MB` above that earlier visible result. The current first post-dismissal result is at least `15.2195 MB` above the earlier dismissed result. Build, OS and harness differences mean these deltas identify a regression window, not a causal module.

A prior fresh-process `QueryWorkingSet` snapshot reconciled `140.29 MB` exactly:

- native images and DLLs: `88.82 MB`;
- private heap and managed/runtime pages: `34.10 MB`;
- mapped files: `17.37 MB`;
- shared pages: `100.99 MB` of the total.

Evidence: [`artifacts/ticket19-profiler-attribution.json`](../../artifacts/ticket19-profiler-attribution.json). This historical split cannot be assigned to the current `182.5195 MB` peak without a current snapshot.

## Source and lifecycle observations

- `MainPage.ReleasePresentation` clears popup projections, the batch `ItemsSource`, visual requests, card view models, cached storage items, known batch identities and pending animations before hiding the native window.
- The next show rebuilds 100 `BatchCardViewModel` instances and up to 300 bounded preview projections, while `ItemsRepeater` realizes only the visible card range.
- A 100-item popup realized only 10–11 rows. Closed-popup UI Automation exposed no popup rows. Bulk Pinning itself does not eagerly materialize all Shelf Items.
- Code scanning found no critical string-slicing/culture patterns. The relevant LINQ and list allocations are bounded projection/model work and are not evidence for tens of megabytes of native physical residency.
- Prior diagnostic changes were not solutions: one initial resize and composition-definition disposal did not reduce the failure; retaining presentation reduced churn but still measured `154.6445 MB` and was reverted. Evidence: [`artifacts/automatic-growth-scale.json`](../../artifacts/automatic-growth-scale.json).

## Current measured evidence & hypothesis evaluation

Measured against the verified Release candidate (`BulkPinningQualification\DropCove.exe`, SHA-256 `1de244817796...`, DropCove.dll SHA-256 `2663046539cd...`, Core SHA-256 `04b69327...`, Native SHA-256 `77a4f846...`, Services SHA-256 `4afb0078...`) using Win32 `QueryWorkingSet` exact page enumeration reconciled against `WorkingSet64`:

| Checkpoint | Total WS | Shared WS | Private WS | Native Images | Private Heap/Managed | Mapped Files |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Fresh Visible** (100×1,000) | **154.41 MB** | 112.76 MB | 41.65 MB | 98.39 MB | 36.13 MB | 19.88 MB |
| **First Dismissal** (100×1,000) | **156.67 MB** | 114.37 MB | 42.30 MB | 99.22 MB | 36.77 MB | 20.67 MB |
| **Cycle 5** | **165.92 MB** | 115.54 MB | 50.38 MB | 99.52 MB | 44.84 MB | 21.55 MB |
| **Cycle 10** | **173.91 MB** | 116.37 MB | 57.54 MB | 99.98 MB | 52.00 MB | 21.94 MB |
| **Cycle 20** | **177.07 MB** | 116.75 MB | 60.32 MB | 100.05 MB | 54.77 MB | 22.24 MB |
| **Cycle 30** | **180.68 MB** | 117.23 MB | 63.45 MB | 100.11 MB | 57.89 MB | 22.68 MB |
| **Control Fresh Visible** (1×1) | **140.50 MB** | 106.65 MB | 33.85 MB | 93.78 MB | 28.47 MB | 18.25 MB |
| **Control First Dismissal** (1×1) | **143.82 MB** | 108.49 MB | 35.32 MB | — | — | — |
| **Control Cycle 30** (1×1) | **153.91 MB** | 111.67 MB | 42.25 MB | 95.90 MB | 36.82 MB | 21.19 MB |

Evidence: [`artifacts/residency-current-attribution.json`](../../artifacts/residency-current-attribution.json).

### Hypothesis ranking & evaluation

1. **Private heap / managed or WinUI presentation retention (LEADING UNRESOLVED HYPOTHESIS - RANK #1)**:
   - Across 30 show/dismiss cycles in the canonical 100×1,000 scenario, `privateHeapAndManagedMb` (MEM_PRIVATE) grew monotonically from **36.77 MB** to **57.89 MB** (**+21.15 MB**), accounting for **88.1% of the total 24.02 MB cycle accumulation**.
   - By contrast, native image pages grew by only **+0.89 MB** (99.22 MB → 100.11 MB) and mapped files by **+2.01 MB**. Top resident modules (`nvwgf2umx.dll`, `System.Private.CoreLib.dll`, `Microsoft.ui.xaml.dll`, `Microsoft.Windows.SDK.NET.dll`) showed negligible residency deltas (+0.0 MB to +0.2 MB).
   - *Crucial caveat*: `privateHeapAndManagedMb` is an aggregate category comprising both .NET managed GC heap and native C++/WinUI process heap pages. This measurement proves cycle retention is concentrated in private pages, consistent with retained presentation objects, element trees, or visual resources; isolating managed objects vs native XAML peer allocations remains a prerequisite before selecting an architecture fix.
2. **Scale-dependent baseline card realization (OBSERVED SCALE DELTA - RANK #2)**:
   - Comparing the 100×1,000 canonical fresh visible (154.41 MB) against the 1-batch control (140.50 MB) reveals an observed **+13.91 MB scale delta**.
   - The delta is split between **+7.80 MB private heap/managed** and **+6.11 MB shared native pages**, reflecting initial realization of 100 `BatchCardViewModel` instances, `ItemsRepeater` templates, and layout structures.
3. **Visual/icon pipeline churn (RANK #3 - SECONDARY)**:
   - While thumbnail requests contribute initial private allocations, the cycle delta is dominated by private heap growth across repeated transitions, not image module/file bloat.
4. **Environment/build baseline shift (FALSIFIED)**:
   - The 1-batch control fresh visible is **140.50 MB**, strictly below 150 MB and closely tracking the historical 140.29 MB baseline. The OS/driver environment itself is not responsible for pushing DropCove above 150 MB.

### Status of Ticket 02

Ticket 02 remains **blocked**:
- Ticket 01 demonstrates that the failure is split into an observed +13.91 MB fresh-scale delta and a +21.15 MB cycle accumulation in private pages (MEM_PRIVATE).
- However, because MEM_PRIVATE combines managed .NET GC heap and native C++/WinUI heap, the exact allocation owner is not yet isolated.
- Profiling distinguishing managed GC roots/objects from native WinUI/composition allocations (e.g. via managed dump analysis or ETW allocation sampling) is required before production implementation or architecture fixes begin.
## Required next evidence

At fresh visible, first dismissal, cycles 5, 10, 20 and 30:

- reconcile `WorkingSet64` against page enumeration;
- split shared/private/mapped/image pages;
- report resident module and mapped-file deltas;
- compare a one-batch control with the canonical 100-batch/1,000-item profile;
- use built-in Windows profiling facilities if page attribution cannot distinguish managed/native ownership;
- record executable and DLL fingerprints, OS, GPU/driver, DPI and exact lifecycle state;
- do not change production code until the measured owner is specific enough to predict a reduction that reaches `<150 MB`.
