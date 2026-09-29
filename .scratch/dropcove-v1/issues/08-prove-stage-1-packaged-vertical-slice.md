# 08: Prove the Stage 1 packaged vertical slice

**What to build:** Deliver a verified first usable DropCove slice that carries real files from Explorer/Desktop into a persisted shelf and back into representative Windows destinations.

**Blocked by:** 06: Restore the shelf through SQLite; 07: Render native visuals at shelf scale.

**Status:** ready-for-agent

- [ ] A development-signed packaged build completes Explorer/Desktop → DropCove → Explorer for one file, one folder, and a multi-item Shelf Batch.
- [ ] Drag-out is also exercised with an Edge/Chrome file input, VS Code, and at least one common chat application.
- [ ] Accepted, canceled, rejected, unsupported, Missing-cleanup, Unavailable-partial, and same-integrity failure outcomes match the application-seam contract.
- [ ] Valid temporary references survive canceled/failed operations; confirmed-Missing references removed before drag are not restored on cancellation.
- [ ] Restart restore, hotkey, single-instance, tray, current-monitor placement, DPI behavior, topmost behavior, and hidden auto-start pass packaged smoke scenarios.
- [ ] The application-seam test suite uses a real temporary SQLite database and passes without per-service mock-only assertions.
- [ ] The 100 Shelf Batch / 1,000 Shelf Item scenario remains interactive and thumbnail/database work does not block the UI thread.
- [ ] Baseline release-build measurements are recorded for idle CPU, summon latency, and working set, with any misses fixed or explicitly blocking completion.
- [ ] Stage 1 contains no Edge Rail, shake-to-open, Compact/Expanded split, telemetry, updater, or file-launch feature.