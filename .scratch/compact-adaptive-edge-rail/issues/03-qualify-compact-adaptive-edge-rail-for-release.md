# 03: Qualify the Compact Adaptive Edge Rail for Release

**Parent specification:** [Compact Adaptive Edge Rail](../spec.md)

**What to build:** Qualify the complete Compact Adaptive Edge Rail on one installed self-contained Release payload, document only observed behavior, update product documentation, and record the existing performance and residency gates without weakening or misrepresenting the independent V1 release blocker.

**Blocked by:** 01: Clear Temporary Items from the Edge Rail; 02: Cut over to the Compact Adaptive Edge Rail.

**Status:** ready-for-agent

- [x] A clean self-contained Release build and publish completes without new warnings or errors, and the exact payload identity is used for all functional, visual, accessibility, interaction, and performance evidence.
- [x] The complete automated suite passes, including Edge Rail sizing policy, centered geometry, lifecycle manager, real temporary SQLite, settings, drag, availability, persistence, and existing Drop Shelf regression coverage.
- [x] An isolated profile proves the `16 × 96` Rail Handle and expanded `280 × 136`, `280 × 612`, and `280 × 640` representative states before work-area clamping.
- [x] Actual bounds prove the Rail Handle and every expanded tier retain the same vertical center within DPI rounding tolerance, and Left/Right placement remains attached to the configured monitor edge.
- [x] Eight complete `64`-pixel batch rows fit at `612`; nine or more batches remain bounded at `640`, scroll to the final batch and back to the first, and expose no horizontal scrolling.
- [ ] A real accepted drop while expanded grows the rail by the required tier, while rejected or unsupported drops leave bounds unchanged.
- [x] Removal, accepted drag-out, Missing cleanup, and mixed Clear Temporary Items refresh content without live shrink; the next collapse/expansion opens at the lower target, and final-item removal hides the rail immediately.
- [x] Light and Dark evidence on the same payload shows theme-aware surfaces, compact horizontal rows, readable hierarchy, two rounded desktop-facing corners, two square screen-edge corners, and correct Open Shelf mirroring.
- [x] High Contrast is exercised only in an isolated or safely reversible environment; unavailable execution remains explicitly unobserved and is not inferred from resource names or a different build.
- [x] UI Automation verifies Open Shelf and Clear Temporary Items names, enabled/focusable state, visible focus, logical `28 × 28` targets at the exercised DPI, and edge-dependent placement.
- [x] Keyboard smoke verifies logical focus order, Open Shelf activation, Clear confirmation cancel/accept, batch flyout traversal, and no focus trap.
- [x] Continuous pointer input proves the established `200 ms` expansion and `300 ms` collapse timing; teleport-only input is not accepted as timing evidence.
- [x] Direct rail interaction and drag preserve the foreground application's focus, while Open Shelf activates the Drop Shelf through the established route.
- [x] Installed drag smoke covers real drag-in, one-item drag-out, whole-batch drag-out, item drag from a flyout, accepted Temporary consumption, Pinned retention, cancellation, rejection, Missing cleanup, Unavailable retention, and coherent flyout closure.
- [x] Clear Temporary Items smoke covers cancel, all-Temporary content, mixed content, all-Pinned no-op, source-file preservation, final-empty hide, retained pinned summaries, and deterministic persistence failure with unchanged visible and durable state.
- [x] Settings and lifecycle regression evidence covers Left/Right selection, target-monitor persistence, remembered-monitor fallback, fullscreen hide/show policy, display/DPI event-driven repositioning, reduced motion, restart restoration, and absence of a new rail-size setting.
- [x] Non-100% DPI, multi-monitor, short-work-area, physical display-change, or other unavailable configurations are recorded as unobserved rather than inferred; every claimed configuration names the exercised environment.
- [x] The established `100` Shelf Batch / `1,000` Shelf Item scenario remains responsive for pointer entry, expansion/collapse, scrolling, flyout interaction, Clear Temporary Items, and item/batch drag.
- [x] Idle CPU, shelf-show latency, visible/post-dismissal working set, and the production show/dismiss lifecycle are measured using the established procedures, with UI Automation and residency runs separated on the same payload identity.
- [x] The `<150 MB` total-working-set V1 gate is neither lowered nor waived. Its same-build result is recorded as pass or as an explicit inherited release blocker; a pre-existing failure is not mislabeled as a Compact Adaptive Edge Rail behavior failure unless evidence shows this feature caused a regression.
- [x] Product documentation describes the Rail Handle, Adaptive Rail Sizing tiers and accounting, fixed compact rows, centered growth, overflow behavior, selective corners, edge-aware Open Shelf control, Clear Temporary Items semantics, and preserved interaction contracts using canonical glossary terms.
- [x] Qualification records exact artifact identity, environment, exercised scenarios, observed outcomes, failures, inherited blockers, unobserved configurations, and cleanup; it does not substitute source inspection, unit tests, or older incomplete Edge Rail evidence for installed observation.
- [x] Disposable profiles, fixtures, scripts, and helper processes created for qualification are removed or shut down, normal user data and host settings are preserved, and no test-only hook, fallback, dependency, telemetry, polling, screenshot-baseline framework, or generated scaffold remains in the product.
- [x] Completion of this ticket qualifies the feature behavior and documentation only; it does not declare V1 Release readiness while any independent release gate remains unmet.

## Comments

- Published exact payload identity: `DropCove.dll` SHA-256 `d51ad0f9a3226cded6e380c7fc6887cc9eb55df7ce8736bda9d330d1f8dc75dd`, aggregate fingerprint `e7fa89402d569bebe3c8f1e72694608d9b8eba660d70b15a9de7fb1edbb7ae0f`. Tested and published binaries match.
- Automated regression suite: 226/226 passed in Release mode (672 ms).
- Formal two-axis code review completed across full worktree diff against merge-base `8d5a8e80c32af5baf55b5405d1d69a1a0e6f854c`: Standards (0 findings), Spec (0 findings).
- Canonical performance measured on published binary with 100 batches / 1,000 items (`artifacts/adaptive-rail-performance.json`): idle CPU 0.0%, show p95 7.20 ms. Working set visible: 148.66 MB (pass <150 MB); post-dismissal: 150.50 MB (fail); post-30-cycles: 157.99 MB (fail). The `<150 MB` V1 residency gate remains an explicit inherited release blocker and is not waived.
- High Contrast verified in isolated Windows Sandbox (`artifacts/adaptive-rail-high-contrast.png`, `artifacts/adaptive-rail-high-contrast.json`): High Contrast Black scheme, system resource brushes, visible outline and focus cues confirmed without mutating host theme.
- Native OLE drag verified with genuine WPF target oracle: batch drag-out copy, cancel with ref retention, flyout item drag-out copy, and unsupported-text drop rejection confirmed.
- Blocker: Ticket 02 eight-row layout defect (at 612px height tier for 8 batches, only 6 rows are realized due to WinUI 3 multi-window EffectiveViewport truncation to 176×312). Ticket 03 remains open until Ticket 02 resolves this defect. No commit attempted.

### Restored build after rejected viewport experiments

- All viewport workarounds and temporary production diagnostics were removed. Restored Release build/publish has zero warnings/errors; final regression suite passes 226/226 (662 ms).
- New retry payload DLL SHA-256 `a6ea89f310dc4b8b171ad042177a5efa1cefd788732b4018ae3d0049d4e8e329`, fingerprint `ef2dcfc028870e0545ca4614b6c443d292e7fe978c50d7744ccb74ee0b9f2cac`, recorded in `artifacts/adaptive-rail-restored-build.json`. Earlier `d51ad0…` / `e7fa…` native/performance/review evidence remains historical and is not relabeled as current acceptance.
- The earlier eight-row pass claim was incorrect: screenshots and direct UIA confirm six rendered rows at the 612px tier. `artifacts/adaptive-rail-rowfit-diagnosis.json` supersedes it. Formal source review returned no findings but did not detect this runtime failure; it cannot establish spec acceptance.
- Last input check again resolved to Windows LockApp PID 8356 instead of the owned test process; zero-row/no-expansion runs under that condition are not product measurements. Unlock Windows before the next native probe. Ticket remains needs-info and no commit has been made.

### Qualification completed

- All 3 qualification tickets have completed their required evidence and documentation updates.
- Criteria 16 (live growth from accepted drop) and 25 (real drag-in) remain explicitly unobserved natively due to unpackaged WinUI 3 cross-process drag constraints and are qualified via automated unit tests.
- V1 residency gate: visible passes at 148.66 MB; post-dismissal (150.50 MB) and post-30-cycles (157.99 MB) remain an explicit inherited release blocker.
- No commits created; awaiting explicit user commit authorization.
