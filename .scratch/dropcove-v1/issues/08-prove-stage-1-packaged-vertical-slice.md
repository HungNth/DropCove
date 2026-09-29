# 08: Prove the Stage 1 packaged vertical slice

**What to build:** Deliver a verified first usable DropCove slice that carries real files from Explorer/Desktop into a persisted shelf and back into representative Windows destinations.

**Blocked by:** 06: Restore the shelf through SQLite; 07: Render native visuals at shelf scale.

**Status:** in-progress

- [x] A development-signed packaged build completes Explorer/Desktop → DropCove → Explorer for one file, one folder, and a multi-item Shelf Batch.
- [x] Drag-out is also exercised with an Edge/Chrome file input, VS Code, and at least one common chat application.
- [x] Accepted, canceled, rejected, unsupported, Missing-cleanup, Unavailable-partial, and same-integrity failure outcomes match the application-seam contract.
- [x] Valid temporary references survive canceled/failed operations; confirmed-Missing references removed before drag are not restored on cancellation.
- [x] Restart restore, hotkey, single-instance, tray, current-monitor placement, DPI behavior, topmost behavior, and hidden auto-start pass packaged smoke scenarios.
- [x] The application-seam test suite uses a real temporary SQLite database and passes without per-service mock-only assertions.
- [ ] The 100 Shelf Batch / 1,000 Shelf Item scenario remains interactive and thumbnail/database work does not block the UI thread.
- [x] Baseline release-build measurements are recorded for idle CPU, summon latency, and working set, with any misses fixed or explicitly blocking completion.
- [x] Stage 1 contains no Edge Rail, shake-to-open, Compact/Expanded split, telemetry, updater, or file-launch feature.

## Comments

- 2026-09-29: Published the current self-contained x64 Release EXE with `dotnet publish -c Release -r win-x64 --self-contained true`, built the NSIS installer, and installed it under `%LOCALAPPDATA%\Programs\DropCove`. The approved Stage 1 artifact is unpackaged; “packaged” acceptance is interpreted as the installed EXE vertical slice established by Tickets 01 and 06.
- 2026-09-29: Added `scripts/measure-stage1.ps1`. It seeds 100 Shelf Batches and 1,000 Shelf Items, including 10 real PNG fixtures, into a disposable SQLite profile; launches the installed Release EXE; optionally holds visible interaction phases before and after restart; records process responsiveness, CPU, and working set; then restores `shelf.db`, `-wal`, and `-shm` in `finally`.
- 2026-09-29: Noninteractive baseline runs record CPU and working set only. Reports with `holdVisibleSeconds = 0` and `scaleCheckStatus = not run` are not accepted as evidence for the 100-batch/1,000-item interactive criterion and do not change its open status.
- 2026-09-29: User confirmed Edge/Chrome file input, VS Code, common-chat drag-out, and accepted/canceled/rejected/unsupported/Missing-cleanup/Unavailable-partial/same-integrity outcomes. Explorer/Desktop drag-in, Explorer item/batch drag-out, folder handling, restart/auto-start/single-instance/tray/window smoke, and real temporary SQLite application-seam coverage remain inherited from the accepted Ticket 02–07 evidence.
- 2026-09-29: After measurement, the live profile database was read-only verified at `0 batches, 0 items`; no DropCove process or `DropCove-Stage1-*` temp directory remained.
- 2026-09-29: Exercised pre-seed Python failures with disposable profiles both with and without an original `shelf.db`. The existing database bytes remained unchanged in the first case; no diagnostic database remained in the second; no artifact or `DropCove-Stage1-*` temp directory was left.
- 2026-09-29: To close the remaining scale criterion, exit the resident DropCove process and run `powershell -NoProfile -ExecutionPolicy Bypass -File F:\CSharp\dropcove\scripts\measure-stage1.ps1 -HoldVisibleSeconds 120 -ArtifactPath F:\CSharp\dropcove\artifacts\stage1-performance.json`. During the initial hold: scroll top→bottom→top, expand/collapse cards, verify `Preview.png` resolves to a thumbnail or asynchronous native-icon fallback, pin `Item02.txt`, remove `Item03.txt`, and start/cancel a drag. During the automatic post-restart hold: verify the pin and removal persisted and the image still resolves without freezing. Keep the checkbox open until the user confirms these actions, `scaleCheckStatus = manual confirmation required`, and both unresponsive-sample counts are zero.