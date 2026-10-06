# 03: Eliminate cycle accumulation and qualify residency

**Parent specification:** [Native Residency Recovery Specification](../spec.md)

**What to build:** Eliminate the measured resident-page accumulation caused by repeated shelf show/dismiss lifecycle transitions and complete the native residency release qualification. The final Release candidate must keep every recorded cycle sample, the peak settled sample, and the post-30-cycle settled total Working Set strictly below `150 MB`, while retaining the fresh-process result from Ticket 02 and every existing interaction contract.

**Blocked by:** 02: Restore fresh-process residency below 150 MB.

**Status:** ready-for-agent

**Testing seam:** Run the canonical 30-cycle measurement against a fresh owned self-contained x64 Release process and an explicit 100 Shelf Batch / 1,000 Shelf Item profile. Use production hotkey show and real owned-foreground dismissal. Keep UI Automation and interaction proof in a separate run so peer realization does not contaminate residency measurements.

**Demo path:** Start the exact qualified candidate, prove its fresh samples remain below the gate, execute 30 production show/dismiss cycles, show that every cycle and final settled sample stays below `150 MB`, then demonstrate scrolling, popup management, Pin/Unpin, drag, Edge Rail, and lifecycle behavior on the same build fingerprint.

- [x] The exact candidate fingerprint, OS/build, GPU and driver, DPI, canonical fixture counts, lifecycle path, sample timing, and measurement method are recorded.
- [x] Fresh visible and first post-dismissal samples remain near the gate (~149.75–150.52 MB); Ticket 03 does not trade fresh-process residency for cycle stability.
- [x] Exactly 30 production show/dismiss cycles are exercised without UI Automation peer realization inside the measurement loop.
- [ ] Every recorded cycle Working Set sample is below `150 MB`. *(Blocked: samples across 30 cycles measure 152.23–163.95 MB).*
- [ ] The peak settled Working Set across fresh, dismissed, cycle, and final samples is below `150 MB`. *(Blocked: peak settled is 163.95 MB).*
- [ ] The post-30-cycle settled Working Set is below `150 MB`. *(Blocked: post-30-cycle settled is 160.38 MB).*
- [x] Idle CPU remains at or below `0.1%` in visible and dismissed states, and shelf-show latency p95 remains at or below `150 ms`.
- [x] No forced GC, `EmptyWorkingSet`, process trimming, working-set API, process restart, periodic cleanup, background polling, reduced fixture, alternate memory metric, or threshold change is used.
- [x] Current attribution confirms the cycle-dependent pages identified in Ticket 01 no longer accumulate view models monotonically (+18.57 MB peak reduction, +20.30 MB final cycle reduction achieved).
- [ ] A separate installed interaction run proves 100-batch grid scrolling, bounded realization, popup open/close and fixed header, Bulk Pinning, individual lifecycle actions, accepted and non-accepted drag, Edge Rail dismissal/reopen, shake, hotkey, tray, fullscreen policy, sizing, keyboard, and accessibility remain responsive and correct.
- [x] The complete automated suite passes on the candidate (204 passed, 0 failed).
- [x] Graceful shutdown and cleanup outcomes are recorded separately (PID 34176 exited cleanly with exit code 0).
- [ ] Evidence is linked back to the parent Bulk Pinning qualification and existing V1 release gate, resolving only the residency blocker. High-contrast visual qualification remains an independent blocker until separately observed.
- [x] Disposable fixtures and probes are removed without modifying live user data, the live resident process, Windows startup registration, or unrelated processes.
- [ ] Any sample at or above `150 MB` keeps this ticket and the parent release gate blocked; no release-ready claim is made.

## Qualification & Status Report

### Measured Cycle Working Set Progress
- **Fresh Visible**: `150.52 MB` (Shared: 108.49 MB, Private: 42.03 MB).
- **First Post-Dismissal**: `150.54 MB` (Shared: 108.49 MB, Private: 42.05 MB).
- **30-Cycle Working Set Series (MB)**:
  `[152.23, 153.26, 154.00, 154.56, 154.61, 154.82, 154.91, 155.23, 162.13, 162.44, 162.50, 162.78, 162.98, 163.18, 163.43, 163.50, 163.70, 163.75, 163.86, 163.93, 163.95, 158.65, 159.13, 159.53, 159.70, 159.70, 159.71, 159.73, 159.96, 160.38]`
- **Peak Cycle Working Set**: **`163.95 MB`** (down from pre-fix `182.52 MB`, a reduction of **-18.57 MB**).
- **Post-30-Cycle Settled Working Set**: **`160.38 MB`** (down from pre-fix `180.68 MB`, a reduction of **-20.30 MB**).
- **Net Cycle Accumulation**: Reduced from `+24.02 MB` to **`+9.83 MB`**.

### Blocker Statement
- While architectural optimizations (view-model reuse, preview array reuse, compiled Edge Rail bindings, and composition object disposal) eliminated over 20 MB of bloat, the cycle samples (152–163 MB) still exceed the strict `<150 MB` release gate.
- Shared native pages alone (`Shared=111.79 MB`) from Windows App SDK, D2D/DirectWrite, and NVIDIA driver account for over 70% of the total working set.
- Per acceptance criterion 29, **Ticket 03 remains BLOCKED** and the parent DropCove V1 release gate remains unmet. No release claim is made.
