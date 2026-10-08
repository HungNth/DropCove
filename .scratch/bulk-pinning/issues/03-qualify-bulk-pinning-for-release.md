# 03: Qualify Bulk Pinning for release

**Parent specification:** [Bulk Pinning for Shelf Batches Specification](../spec.md)

**What to build:** Complete the narrow post-implementation qualification gate for Bulk Pinning. Produce clean installed Release evidence that the card and popup slices work together across lifecycle, accessibility, keyboard, persistence-failure, drag, availability, theme, Edge Rail, and performance contracts without shipping test scaffolding or weakening existing requirements.

**Blocked by:** 02: Manage Bulk Pinning inside the anchored popup.

**Status:** needs-info

**Testing seam:** Run the complete automated suite, then exercise the installed self-contained Release through an isolated test profile and UI Automation. Use real temporary SQLite state and a deterministic abort trigger for failure evidence. Installed observations are authoritative for XAML visuals, automation, focus, popup behavior, real drag interaction, and performance.

**Demo path:** In the clean installed Release, drive an all-Temporary, Mixed, and all-Pinned Shelf Batch through card and popup Bulk Pinning; restart to prove durability; trigger a deterministic persistence failure; exercise keyboard and assistive-technology states; perform accepted and non-successful drag-out; then record theme, Edge Rail, responsiveness, and established performance-gate results.

- [x] A clean self-contained Release build and publish completes without new warnings or errors and is the artifact used for installed qualification.
- [x] The complete automated suite passes, including manager-plus-real-SQLite coverage for atomic Bulk Pinning, restart, availability, deterministic transaction abort, and unchanged drag lifecycle.
- [x] Installed card and popup evidence covers Off → On, Mixed → On, and On → Off, with every participating Shelf Item verified rather than inferring success from aggregate text alone.
- [x] Restart evidence proves bulk-pinned and bulk-unpinned states are durable and that no batch-owned pin state or schema migration exists.
- [x] An isolated test profile with a deterministic SQLite abort proves the visible failure message, unchanged card/header/item states, retained popup focus when applicable, unchanged persisted state after reopen, and no automatic retry. Live user data is not modified.
- [x] UI Automation records Off, On, and Indeterminate toggle states and the count-aware accessible names for both card and popup controls.
- [x] Keyboard evidence proves keyboard-open focus starts on the bulk header, `Tab` and `Shift+Tab` traverse all header and virtualized row actions in both directions, `Enter` and `Space` activate, wrapping is correct, and `Esc` restores chevron focus.
- [x] Long-list evidence proves the popup header remains fixed, item rows remain virtualized, the list scrolls vertically only, popup bounds remain within existing limits, and the Drop Shelf bounds do not change.
- [ ] Light, dark, and high-contrast evidence proves Mixed is distinguishable through shape rather than color or opacity alone.
- [x] Availability evidence proves Available, Unavailable, and confirmed-Missing references still present in a Shelf Batch participate in Bulk Pinning without reclassification or cleanup.
- [x] Real drag-out evidence proves accepted operations consume only participating Temporary Items and retain participating Pinned Items, while cancellation, rejection, failure, and unaccepted outcomes preserve valid references; source filesystem objects remain unchanged.
- [ ] Single-item card behavior, individual Pin/Unpin, Remove, Clear Temporary Items, popup owner closure, shelf sizing, Automatic Shelf Growth, Manual Height Override, shake, hotkey, tray, fullscreen policy, and Edge Rail behavior remain unchanged.
- [ ] Deferred popup projections and interaction remain responsive for a long Shelf Batch. The established `100` Shelf Batch / `1,000` Shelf Item gate remains satisfied: visible and post-dismissal working set below `160 MB`, idle CPU at or below `0.1%`, responsive pointer/scroll/drag interaction, and shelf-show latency p95 at or below `150 ms`.
- [x] Qualification records only exercised configurations and outcomes, preserves explicit blockers for anything required but unverified, and does not substitute automated tests or earlier evidence for failed installed scenarios.
- [x] Production output contains no disposable trigger, diagnostic, smoke harness, fallback path, new telemetry, polling, persistence abstraction, UI test project, or unused package introduced for qualification.

## Comments

### 2026-10-06 — Qualification remains blocked

- Current installed-directory Release builds/publishes without warnings/errors and has an isolated profile, separate from the live resident application. This is a side-by-side self-contained publish, not an NSIS replacement or a final installer qualification claim.
- Actual [card evidence](../../../artifacts/bulk-pinning-card-smoke.json) and [partial popup evidence](../../../artifacts/bulk-pinning-popup-smoke.json) record tri-state, lifecycle, restart, atomic failure, count/name, and exercised focus behavior.
- Ticket 02 is not fully accepted: real keyboard traversal stopped immediately when the user's terminal took foreground. Complete forward/reverse wrap, fixed-header end-of-list scrolling, theme proof, and real integrated drag/scale scenarios require an exclusive foreground interval.
- Historical scale evidence is not reused as a current pass. The existing `100`-batch / `1,000`-item gate and post-cycle residency requirement remain unchanged, including the previously recorded `165.61 MB` post-cycle failure in `artifacts/automatic-growth-scale.json`.
- Two-axis code review: Standards reported no documented-standard breach and one P3 heuristic about imperative header projection copying; retained because it follows the existing popup lifecycle convention without adding an abstraction. Spec reported zero source findings and explicitly left installed qualification incomplete.
- Integration review added an owner-identity guard around asynchronous bulk completion: a recycled card/shared popup control must not receive the previous owner's state or focus after SQLite yields. The race scenario still needs the exclusive-foreground smoke; compilation alone is not runtime proof.
- Final reviewed Release publish completed without warnings/errors. Full suite run once after source stabilization: `dotnet test tests/DropCove.Tests/DropCove.Tests.csproj -c Release --no-restore` — 204 passed, 0 failed/skipped, 645 ms. Earlier desktop evidence predates the final owner guard; final installed replay and race proof remain required.
- No commit has been made or separately authorized.

### 2026-10-06 — Final source review and OS recovery blocker

- Exclusive-foreground replay completed Ticket 02's installed interaction proof, including the explicit keyboard-source fix and both delayed SQLite owner races. Latest parallel Standards and Spec reviews reported zero actionable findings. The prior optional projection-copy heuristic follows the repository's existing convention.
- Light and dark Mixed visuals were captured and inspected: [light](../../../artifacts/bulk-pinning-light.png), [dark](../../../artifacts/bulk-pinning-dark.png). High-contrast activation killed the Python probe kernel (exit 116) before its screenshot/restoration completed; that probe is not a pass.
- [OS recovery evidence](../../../artifacts/bulk-pinning-theme-recovery.json): original SPI flags `126`, scheme empty, AppsUseLightTheme `0`; independent recovery found active contrast (`127`, High Contrast Black). Contrast is now off (`126`), AppsUseLightTheme `0`, and registry High Contrast Scheme empty. SPI still returns High Contrast Black. Full restoration remains unverified despite three approaches, including a NULL scheme pointer; all further smoke stopped.
- Current blocker is exact OS-state recovery/user decision, not foreground permission. No further theme, drag, or scale probe is performed while SPI differs from its recorded baseline. The disposable real drag oracle source and isolated scale fixtures are prepared but have not been exercised.
- Manager-plus-real-SQLite availability and drag lifecycle cases passed in the automated suite. This is not substituted for the still-unverified real installed drag and current performance gate. No release-ready claim or commit.

### Post-session continuation

- Read-only SPI and registry query now exactly match captured baseline: flags `126`, native scheme empty, registry scheme empty, AppsUseLightTheme `0`. Host theme/contrast is not changed again; remaining contrast proof uses an owned Windows Sandbox guest with read-only payload/fixture mappings.
- Expected persistence errors are caught specifically as `Microsoft.Data.Sqlite.SqliteException`; cancellation and unexpected runtime faults are not silently mislabeled as “Nothing changed.” Clean Release publish and actual abort/focus/rollback/manual-retry smoke passed with this boundary.
- [Real drag evidence](../../../artifacts/bulk-pinning-real-drag.json) records correlated destination receipt, actual copied-file integrity and reference lifecycle for seven accepted/non-accepted cases. Invalid driver attempts are excluded rather than reinterpreted as product success.
- Current `<150 MB` / CPU / p95 scale qualification remains independent and is not inferred from source review, the suite, or prior historical samples.



### Final source checks after expected-error narrowing

- Final complete suite on current source: `dotnet test tests/DropCove.Tests/DropCove.Tests.csproj -c Release --no-restore` — 204 passed, 0 failed/skipped, 663 ms. Source, contract tests and actual expected SQLite-error surface remain consistent.
- Final parallel code review: Standards 0 actionable findings; Spec 0 source findings. Both explicitly retain unresolved high-contrast/current-scale qualification rather than treating code correctness as release readiness.

### Isolated high-contrast outcome — unverified

- Existing Windows Sandbox feature was enabled; no tool/SDK/feature installation or host theme mutation was performed. First actual guest run failed on a PowerShell `$PID` parameter collision; corrected to `$appProcessId`.
- Corrected guest attempt timed out after 481.37 seconds without a completion marker or guest events/screenshot at its new output path. The owned client was closed by the host runner. [Unverified contrast record](../../../artifacts/bulk-pinning-high-contrast-unverified.json). No further retries of this harness; no contrast pass inferred.
- High-contrast evidence and the current scale residency gate remain separate acceptance blockers. Actual native drag, source integrity, core tests and code review do not satisfy either gate.

### Current scale gate — failed, not waived

- [Current scale evidence](../../../artifacts/bulk-pinning-scale.json): actual installed-directory Release, isolated fixture exactly 100 Shelf Batches/1,000 Shelf Items, 96 DPI, 30 production hotkey show/real Escape dismiss cycles; no UIA peer realization during the measurement, no forced GC or working-set trimming.
- Visible working set `153.2461 MB`; post-dismissal samples `156.2695–156.6836 MB`; peak settled sample `182.5195 MB`; post-30-cycle settled `178.8672 MB`. Approved `<150 MB` gate fails. CPU is `0.0%`; queue-to-visible show latency p95 `14.3475 ms` passes `<=150 ms`. Neither passing metric cancels the residency failure.
- [Separate cleanup record](../../../artifacts/bulk-pinning-scale-cleanup.json): the owned test process PID 26908 produced a graceful-shutdown timeout warning and the harness invoked its owned force-stop branch. Normal shutdown is not claimed.
- [Separate large-fixture interaction proof](../../../artifacts/bulk-pinning-scale-surface.json) passes actual grid scrolling, popup, atomic exact-batch pin/unpin, closure release and hide/hotkey restore. [Lifecycle regression](../../../artifacts/bulk-pinning-lifecycle-regression.json) confirms the binary single-item action and actual confirmed Clear Temporary retention/source integrity. Shake/tray/fullscreen engine code is intentionally unchanged; unexercised native configurations remain unclaimed.
- Ticket 03 stays `needs-info`: residency fails and high-contrast visual proof is unverified. No repeated qualification attempt, speculative optimization, artificial memory reduction, lowered threshold, or release-ready claim. A planning decision is required before widening implementation scope to native residency work.

### Delivery state and cleanup

- [Final launch](../../../artifacts/bulk-pinning-final-launch.json): final published Release is running as an explicit isolated test profile with actual `Pin all 3 items, 1 currently pinned` / Indeterminate state. Application DLL fingerprint and owned PID 11024 are recorded. Normal resident executable, live profile and Windows startup registration are not replaced.
- [Cleanup](../../../artifacts/bulk-pinning-cleanup.json): disposable consumers/oracles/OS-restore and Sandbox scripts, builds, copies and old fixture files are removed. Own SQLite handles were explicitly closed, without forced collection. Three externally locked empty mapped directories remain recorded; no files or harness code remain there, and no further deletion retry is performed. The separate running verified demo profile and installed payload are intentionally preserved.
- Source work remains uncommitted. Ticket 01's remaining combined theme acceptance and Ticket 03's contrast/performance gates are not checked. Core behavior, SQLite/UIA/native-drag evidence, full suite and two-axis code review are complete, not equivalent to release qualification.

### Post-review stale-owner correction

- A delayed SQLite failure from an old popup owner previously called the shared failure status before checking whether the shared header had been rebound. The identity guard now runs before any status, toggle or focus mutation.
- [Installed owner-race evidence](../../../artifacts/bulk-pinning-owner-guard.json): a blocked 2-item Mixed mutation aborts after the UI switches to a 3-item Mixed popup. Both persisted batches remain unchanged; the new popup keeps its name, Indeterminate state and Manage focus; no stale “Nothing changed” status appears.
- Corrected Release publish succeeded. Final current-source suite: 204 passed, 0 failed/skipped, 651 ms. This correction does not alter the independent residency or high-contrast blockers.

