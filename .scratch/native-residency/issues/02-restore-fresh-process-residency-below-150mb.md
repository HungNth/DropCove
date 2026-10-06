# 02: Restore fresh-process residency below 150 MB

**Parent specification:** [Native Residency Recovery Specification](../spec.md)

**What to build:** Apply the smallest ownership or lifetime correction proven by Ticket 01 so a fresh self-contained x64 Release process with 100 Shelf Batches / 1,000 Shelf Items keeps every visible and first post-dismissal total Working Set sample strictly below `150 MB`. Preserve the existing Drop Shelf, Edge Rail, lifecycle, drag, sizing, accessibility, latency, and idle behavior.

**Blocked by:** 01: Attribute fresh and cycle-dependent resident pages.

**Status:** blocked

**Testing seam:** Verify the implementation through the current automated suite plus an actual installed-directory Release candidate in an isolated canonical profile. Keep canonical residency measurement free from UI Automation realization; run interaction and accessibility smoke separately against the same build fingerprint.

**Demo path:** Launch a fresh canonical candidate, show the attribution-backed resources are no longer resident or are deferred as designed, record every visible and first-dismissal sample below `150 MB`, then exercise the main shelf, Edge Rail, popup, lifecycle actions, and drag without changing product semantics.

- [ ] The production change targets only the owner/lifetime seam demonstrated by Ticket 01; unrelated LINQ, sealing, string, GC, configuration, packaging, or broad XAML rewrites are not included without their own measured evidence.
- [ ] Pre-change and post-change attribution use the same checkpoints and show that the predicted resident pages were removed or deferred rather than merely reclassified.
- [ ] A fresh owned process with exactly 100 Shelf Batches / 1,000 Shelf Items records every settled visible sample below `150 MB`.
- [ ] The same process records every first post-dismissal settled sample below `150 MB`.
- [ ] Results use total process `WorkingSet64`, including shared pages. No average, private-memory metric, forced GC, `EmptyWorkingSet`, process trimming, periodic cleanup, restart, or altered threshold hides a failing sample.
- [ ] Idle CPU remains at or below `0.1%`, shelf-show latency p95 remains at or below `150 ms`, and no new background work or polling is introduced.
- [ ] Compact card projections remain bounded; unopened popup item projections remain deferred; closing, hiding, owner invalidation, and dismissal release presentation-owned projections and visual requests according to the approved contracts.
- [ ] Drop Shelf show/hide, Edge Rail reopen, Bulk Pinning tri-state/card/popup behavior, individual Pin/Unpin, Remove, Clear Temporary Items, drag-in/out, persistence, availability, sizing, shake, hotkey, tray, fullscreen policy, keyboard focus, and accessibility remain unchanged.
- [ ] The complete automated suite passes and installed interaction smoke records only exercised configurations and outcomes.
- [ ] Obsolete residency ownership, cache, or lifecycle code made unnecessary by the correction is removed; no compatibility switch, fallback implementation, feature flag, new runtime package, or public abstraction is retained.
- [ ] If any fresh visible or first-dismissal sample is at or above `150 MB`, the ticket remains open and Ticket 03 does not begin.
- [ ] Failed experiments are reverted and recorded with their actual measurements; they are not accumulated into the final implementation.
