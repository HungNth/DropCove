# 03: Eliminate cycle accumulation and qualify residency

**Parent specification:** [Native Residency Recovery Specification](../spec.md)

**What to build:** Eliminate the measured resident-page accumulation caused by repeated shelf show/dismiss lifecycle transitions and complete the native residency release qualification. The final Release candidate must keep every recorded cycle sample, the peak settled sample, and the post-30-cycle settled total Working Set strictly below `150 MB`, while retaining the fresh-process result from Ticket 02 and every existing interaction contract.

**Blocked by:** 02: Restore fresh-process residency below 150 MB.

**Status:** blocked

**Testing seam:** Run the canonical 30-cycle measurement against a fresh owned self-contained x64 Release process and an explicit 100 Shelf Batch / 1,000 Shelf Item profile. Use production hotkey show and real owned-foreground dismissal. Keep UI Automation and interaction proof in a separate run so peer realization does not contaminate residency measurements.

**Demo path:** Start the exact qualified candidate, prove its fresh samples remain below the gate, execute 30 production show/dismiss cycles, show that every cycle and final settled sample stays below `150 MB`, then demonstrate scrolling, popup management, Pin/Unpin, drag, Edge Rail, and lifecycle behavior on the same build fingerprint.

- [ ] The exact candidate fingerprint, OS/build, GPU and driver, DPI, canonical fixture counts, lifecycle path, sample timing, and measurement method are recorded.
- [ ] Fresh visible and first post-dismissal samples remain below `150 MB`; Ticket 03 does not trade fresh-process residency for cycle stability.
- [ ] Exactly 30 production show/dismiss cycles are exercised without UI Automation peer realization inside the measurement loop.
- [ ] Every recorded cycle Working Set sample is below `150 MB`.
- [ ] The peak settled Working Set across fresh, dismissed, cycle, and final samples is below `150 MB`.
- [ ] The post-30-cycle settled Working Set is below `150 MB`.
- [ ] Idle CPU remains at or below `0.1%` in visible and dismissed states, and shelf-show latency p95 remains at or below `150 ms`.
- [ ] No forced GC, `EmptyWorkingSet`, process trimming, working-set API, process restart, periodic cleanup, background polling, reduced fixture, alternate memory metric, or threshold change is used.
- [ ] Current attribution confirms the cycle-dependent pages identified in Ticket 01 no longer accumulate or remain resident after their owning presentation lifecycle ends.
- [ ] A separate installed interaction run proves 100-batch grid scrolling, bounded realization, popup open/close and fixed header, Bulk Pinning, individual lifecycle actions, accepted and non-accepted drag, Edge Rail dismissal/reopen, shake, hotkey, tray, fullscreen policy, sizing, keyboard, and accessibility remain responsive and correct.
- [ ] The complete automated suite passes on the final candidate.
- [ ] Graceful shutdown and cleanup outcomes are recorded separately. A forced stop of the owned test process is a cleanup warning and cannot turn a failed measurement into a pass.
- [ ] Evidence is linked back to the parent Bulk Pinning qualification and existing V1 release gate, resolving only the residency blocker. High-contrast visual qualification remains an independent blocker until separately observed.
- [ ] Disposable fixtures and probes are removed without modifying live user data, the live resident process, Windows startup registration, or unrelated processes.
- [ ] Any sample at or above `150 MB` keeps this ticket and the parent release gate blocked; no release-ready claim is made.
