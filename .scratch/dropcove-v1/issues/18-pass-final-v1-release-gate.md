# 18: Pass the final DropCove V1 release gate

**What to build:** Produce a production-signable DropCove V1 release candidate whose complete reference, drag/drop, persistence, windowing, Edge Rail, shake, UI, performance, privacy, and packaging contracts have been exercised end to end.

**Blocked by:** 16: Add motion, accessibility, and full multi-monitor polish; 17: Prepare self-contained MSIX for Microsoft Store; 19: Reduce installed Release idle Working Set below 150 MB.

**Status:** blocked

- [x] The full automated application-seam suite passes with real temporary SQLite databases and no test-only service architecture.
- [ ] The installed-EXE compatibility matrix passes for Explorer/Desktop input and Explorer, Edge/Chrome file input, VS Code, and at least one common chat destination; folder drag-out passes with Explorer.
- [ ] Accepted, canceled, rejected, unsupported, Missing-cleanup, Unavailable-partial, at-least-once crash, and same-/cross-integrity behaviors match the specification.
- [ ] Single-instance, hotkey conflict, hidden auto-start, tray, current-monitor/DPI, Edge Rail, fullscreen, shake, Compact/Expanded, reduced-motion, and accessibility smoke scenarios pass.
- [ ] The 100 Shelf Batch / 1,000 Shelf Item scenario remains responsive with lazy thumbnails and virtualization.
- [ ] Installed-EXE release measurements meet idle CPU average at or below 0.1%, zero periodic cursor polling, shelf-show latency p95 at or below 150 ms, and idle working-set target below 150 MB.
- [ ] EXE installation, upgrade, restart, database restoration, corruption backup, and uninstall are verified on a supported Windows 11 environment.
- [ ] Later Microsoft Store submission has verified Partner Center identity association and meets Store requirements.
- [ ] No telemetry, background update checker, application-owned network traffic, source-file mutation, or out-of-scope feature is present.
- [ ] Any unmet acceptance criterion blocks V1 release rather than being converted into a silent follow-up.

## Comments

- 2026-10-01: Ticket 19's final default installed Release measurement after the collapse-release fix records `WorkingSet64` averages of 134.57 MB visible and 141.05 MB after dismissal into Edge Rail for 100 Shelf Batches / 1,000 Shelf Items, with 0.00% idle CPU. The full MSTest suite passes 72/72 and the Release build has 0 errors.
- 2026-10-01: V1 remains blocked by the explicit unmet gates: native Explorer/Desktop and destination compatibility matrix coverage, Edge Rail/shake/current-monitor/reduced-motion/accessibility smoke gaps after the residency change, shelf-show latency p95 evidence, EXE upgrade/database/uninstall lifecycle verification, Store/Partner Center validation, and the open Ticket 19 native profiler-attribution criterion. The interactive supplemental smoke verified projection expansion, pin/remove, collapse/reopen, preview, pinned drag-out, and persistence, but did not verify Explorer drag-in or Edge Rail UIA dismissal/reopen.
