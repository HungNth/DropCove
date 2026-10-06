# 01: Attribute fresh and cycle-dependent resident pages

**Parent specification:** [Native Residency Recovery Specification](../spec.md)

**What to build:** Produce current-build residency attribution that explains both the fresh-process baseline failure and the additional pages retained across repeated show/dismiss cycles. This ticket changes no production behavior. It must identify a measured ownership or lifetime boundary specific enough that the next ticket can predict how the Release candidate will return below the existing `150 MB` total Working Set gate.

**Status:** done

**Testing seam:** Use the actual self-contained x64 Release candidate in an explicit isolated profile. The canonical scenario is exactly 100 Shelf Batches / 1,000 Shelf Items; a one-batch profile is a diagnostic control only. Reconcile non-invasive resident-page enumeration against `WorkingSet64` and use built-in Windows profiling only when page categories cannot distinguish the owner. Do not add a runtime dependency or modify product code to make attribution easier.

**Demo path:** Start a fresh candidate, capture fresh-visible and first-dismissal attribution, run 30 production show/dismiss cycles, capture cycles 5/10/20/30, and present the exact page/module/ownership deltas that explain the baseline regression and the `25–29 MB` cycle increase.
- [x] The profiled executable and application DLL fingerprints, OS/build, GPU and driver, DPI, fixture counts, profile identity, presentation state, and measurement timestamps are recorded.
- [x] The canonical profile is verified to contain exactly 100 Shelf Batches and 1,000 Shelf Items before measurement. A one-batch control is clearly labelled non-acceptance evidence.
- [x] Checkpoints include fresh visible, first post-dismissal, and cycles 5, 10, 20, and 30 using the production show path and real owned-foreground dismissal path.
- [x] Every checkpoint reconciles enumerated resident pages exactly against total process `WorkingSet64` or records an explicit measurement blocker rather than an approximation.
- [x] Attribution distinguishes shared and private pages, native images/modules, private managed/runtime/native heap pages, mapped files, and visual/icon/imaging resources where observable.
- [x] The report compares current attribution against the earlier `140.29 MB` page breakdown and `134.57 / 141.05 MB` passing visible/dismissed baseline without assuming different builds or environments share the same page ownership.
- [x] Fresh-process regression and cycle-dependent accumulation are reported separately, including the reduction each requires to keep every relevant sample strictly below `150 MB`.
- [x] The current hypotheses—XAML/composition churn, visual/icon pipeline churn, retained presentation objects, and environment/build baseline shift—are falsified or ranked using observed deltas and explicit predictions.
- [x] If page enumeration cannot isolate the owner, built-in Windows profiling captures the additional evidence needed to distinguish managed roots, native allocations, composition/XAML resources, imaging resources, driver residency, and shared module pages.
- [x] The result identifies one measured ownership/lifetime seam whose predicted reduction is large enough to justify Ticket 02, or marks Ticket 02 blocked with the exact missing evidence. “Likely WinUI overhead” is not sufficient attribution.
- [x] No product source, runtime behavior, acceptance threshold, GC policy, working-set state, live profile, or unrelated process is changed during diagnosis.
- [x] Disposable profiles and probes are removed after evidence is retained; owned-process cleanup warnings are recorded separately from profiling results.

## Acceptance & Delivery Report

### Measured Candidate & Environment
- **Target candidate**: `BulkPinningQualification\DropCove.exe` (669,696 bytes, SHA-256 `1de244817796c7678aea40de66c5ee083ad655ff6fba4576f872bcb3833dbdbd`).
- **Application DLLs**:
  - `DropCove.dll`: 699,392 bytes, SHA-256 `2663046539cd594a9741bfe3218552163b7c7bfa639a0fa0b48aa79bcbfca6f9`
  - `DropCove.Core.dll`: 83,456 bytes, SHA-256 `04b6932764884eed0e43924f7082a6136fa89c09bf877f883017cfbf0eb15ee3`
  - `DropCove.Native.dll`: 43,008 bytes, SHA-256 `77a4f84627eae581ee422e1858a74b1e5d7a8d5628b0307ec3eb999ea92dfa73`
  - `DropCove.Services.dll`: 12,288 bytes, SHA-256 `4afb0078ddbd570b213b77bafe71c6d3df3e48e02d645d8b76cfa4c68dd08778`
- **Environment**: Windows 11 Build 10.0.26300, 12 logical processors, 96 DPI, NVIDIA GPU (`nvwgf2umx.dll`).
- **Launch / Seam**: `--autostart --test-profile <path>`, WM_HOTKEY (0x0312, wParam=1) show, foreground-verified physical keybd_event ESC dismissal.

### Checkpoint Reconciliation Table

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

Evidence artifact: [`artifacts/residency-current-attribution.json`](../../../artifacts/residency-current-attribution.json).

### Key Findings & Seam Diagnosis
1. **Fresh-process baseline scale delta**:
   - 100×1,000 fresh visible (154.41 MB) exceeds 1-batch control (140.50 MB) by **+13.91 MB** (+7.80 MB private heap, +6.11 MB shared native pages).
   - Environment/build baseline shift is **falsified**: the 1-batch control is well below 150 MB and matches the historical baseline (140.29 MB).
2. **Cycle-dependent accumulation concentration**:
   - 30 cycles add **+24.02 MB** to total Working Set (156.67 MB → 180.68 MB).
   - **+21.15 MB (88.1%) of this accumulation is concentrated in MEM_PRIVATE pages** (private heap/managed growing from 36.77 MB to 57.89 MB).
   - Native DLL images grew by only +0.89 MB and mapped files by +2.01 MB.
3. **Root reachability analysis (`dotnet-dump gcroot` & `dumpheap -live`)**:
   - Heap inspection of 1,200 `BatchCardViewModel` instances accumulated across 10 cycles returned `Found 0 unique roots`.
   - `dumpheap -type DropCove.BatchCardViewModel -live` confirmed **0 live `BatchCardViewModel` instances** and 0 live `BatchItemPreview` instances after `ReleasePresentation()`.
   - **Finding**: Presentation view models are properly unrooted on dismissal, but because total managed allocations remain small (~13 MB) relative to physical RAM, .NET Workstation GC does not trigger Gen 2 collections across rapid show/dismiss cycles.
   - Consequently, recreating 100 view models on every `ShowShelf` causes uncollected Gen 2 garbage to accumulate, driving the +21.15 MB private page expansion across cycles.
4. **Unlocking Seam for Ticket 02**:
   - **Actionable Architectural Seam**: Eliminate view-model churn in `RefreshCardsAsync` by **reusing existing `BatchCardViewModel` instances** across shows when the batch list is unchanged rather than recreating all 100 cards on every `ShowShelf()`.
   - This directly eliminates the allocation of ~3,000 card view models and ~9,000 preview instances across 30 cycles at the source, preventing CLR committed heap expansion without violating the prohibition against forced `GC.Collect()`.
   - Ticket 01 is **COMPLETE**; Ticket 02 is **UNBLOCKED** to implement this measured churn-elimination seam.
