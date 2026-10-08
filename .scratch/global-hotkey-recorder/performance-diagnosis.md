# Native Residency Diagnostic Report: Scale vs Lifecycle Attribution

Date: 2026-10-08
Target payload: `223BF7EB587F18B99B9DC4C42DD5CBEB7EF40764F55BEDA64A7B72E24D859DBB` (Installed Release self-contained x64 `DropCove.exe`).

## Measured Matrix Across Controls

All runs used the identical executable, safe no-existing-process abort guard, explicit isolated `--test-profile`, and verified window visibility.

| Configuration | Batches × Items | Image Fixture | Cycles | Initial / Fresh Visible WS | Peak Sampled WS | Gate `< 160 MB` | Artifact |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **0-Cycle Baseline** | 100 × 10 (1,000) | 10 `Preview.png` | 0 | **153.68 MB** | **153.68 MB** | **PASSED** (< 160 MB) | `artifacts/diagnose-residency-baseline-0cycle.json` |
| **Canonical Stress** | 100 × 10 (1,000) | 10 `Preview.png` | 30 | **153.56 MB** | **167.27 MB** | **FAILED** (exceeds 160 MB) | `artifacts/diagnose-residency-authoritative.json` |
| **1×10 Control** | 1 × 10 (10) | 1 `Preview.png` | 30 | **147.00 MB** | **160.52 MB** | **FAILED** (exceeds 160 MB) | `artifacts/diagnose-residency-control-1x10.json` |
| *1×1 Exploratory* | 1 × 1 (1) | None | 30 | 140.96 MB | 152.58 MB | *Passed (exploratory shape)* | `artifacts/diagnose-residency-control-1x1.json` |
## Verified Empirical Breakdown

1. **Zero-Cycle Fresh Visible Baseline**:
   - At fresh startup with 100 batches (1,000 items), the working set is **153.68 MB**.
   - This proves that **fresh startup alone strictly satisfies the `< 160 MB` gate**. There is no initial static failure.

2. **Scale Delta (1×10 vs 100×10 fresh)**:
   - Initial 1-batch (10 items): **147.00 MB**
   - Initial 100-batch (1,000 items): **153.68 MB**
   - **Initial Scale Delta = +6.68 MB**.

3. **Lifecycle Churn Delta (30 Cycles)**:
   - In 1×10 run: Working set climbs from 147.00 MB to **160.52 MB** (**+13.52 MB** accumulation).
## Ranked Falsifiable Hypotheses for Lifecycle Churn

*Note: The measurements localize the failure pattern to show/dismiss lifecycle accumulation independent of scale. Distinguishing the exact memory owner (managed heap vs WinUI composition surfaces vs native interop) remains an open investigation.*

### Hypothesis 1: Lifecycle view-model/collection churn and CLR committed heap retention (Rank 1 - Leading Hypothesis)
- **Statement**: Ephemeral allocations (projections, delegates, strings, models) generated during show/dismiss transitions remain in committed CLR segments without triggering Gen 2 GC pressure.
- **Prediction**: Reusing presentation models across transitions flattens cycle accumulation.

### Hypothesis 2: WinUI Composition / DirectComposition surface retention (Rank 2)
- **Statement**: Unmanaged composition visual trees allocate per-transition surfaces that settle into working set without prompt trimming.

### Hypothesis 3: Window Message Hook / native interop allocations per activation (Rank 3)
- **Statement**: Repeated activation / message dispatch retains small unmanaged handles or structures per show event.
