# 04: Verify Automatic Shelf Growth in the installed Release

**Parent specification:** [Automatic Shelf Growth](../spec.md)

**What to build:** Prove the complete Automatic Shelf Growth contract in the installed self-contained Release application, close any integration defects exposed by real Windows behavior, and update product documentation with the verified hybrid sizing model. Evidence must distinguish observed behavior from unavailable monitor, DPI, input, or shell scenarios.

**Blocked by:** 03: Recalculate automatic height across shelf lifecycles.

**Status:** blocked

- [x] The full automated suite passes with obsolete assertions that every accepted Shelf Batch preserves visible bounds replaced by the approved hybrid contract.
- [ ] A real Explorer/Desktop one-column scenario demonstrates `180/236/292/348` progression, four-row cap, vertical-only overflow, newest-first visibility, and no automatic width change.
- [x] Wider responsive scenarios demonstrate that existing-row capacity does not grow the shelf and that only a newly required row advances the height tier.
- [ ] Pointer smoke verifies left/right resize does not create or clear Manual Height Override, top/bottom and corner resize establishes it when height changes, and later accepted Shelf Batches respect it.
- [ ] Lifecycle smoke verifies pending empty-shelf sizing, first-item persistence, restart before first content returning to `180 × 180`, exact manual restoration, final-item reset, no live shrink, and recalculation on later show/restart.
- [ ] Window smoke verifies top-edge anchoring, upward correction and clamping near work-area limits, popup closure without stranded focus, keyboard usability, and reduced-motion behavior.
- [x] Multi-monitor and non-100% DPI behavior is exercised where the environment permits; unavailable scenarios are recorded as unobserved rather than reported as passing.
- [ ] The installed Release gate with `100` Shelf Batches and `1,000` Shelf Items keeps visible and post-dismissal working set below `150 MB`, idle CPU at or below `0.1%`, responsive pointer/scroll/drag interaction, and shelf-show latency p95 at or below `150 ms`.
- [x] User-facing product documentation describes Automatic Shelf Growth, Manual Height Override, row-based tiers, no live shrink, persistence, reset, and Edge Rail non-impact using the canonical glossary terms.
- [x] The ticket records the exact installed artifact, exercised scenarios, observed results, performance measurements, and any environmental limitations needed to reproduce the acceptance evidence.

## Acceptance evidence and release blocker

- Installed self-contained Release: `%LOCALAPPDATA%\Programs\DropCove\DropCove.exe`, built with the existing CLI publish and NSIS installer; verification uses isolated `--test-profile` directories, not normal user data.
- Actual Explorer one-column drops observed heights `180/236/292/348/348` for batches 1–5 with width `180` and fixed top edge. Width-only native resize to `350` preserved a null override; at that width batches 1–3 observed heights `180/180/236`. Evidence: `artifacts/automatic-growth-installed-drops.json`.
- Installed lifecycle smoke: five batches open at `348`; visible removal to one batch holds `348`; hide/show and restart recalculate `180`. A real vertical pointer resize selects `240`; later drops and restart preserve `240`. Evidence: `artifacts/automatic-growth-lifecycle.json`.
- **Failing gate:** `100` batches / `1,000` real text-file fixtures began near `145 MB`, but 30 hide/show cycles left `165.61 MB` after settling. CPU samples were `0.0%`; native hotkey-message-to-visible p95 was `6.24 ms`, not a physical-keyboard or frame-present measurement. This is the last measured failing artifact, not a passing result for the final reviewed artifact. Evidence: `artifacts/automatic-growth-scale.json`.
- Three diagnosis attempts did not meet `<150 MB`: one opening resize (`171.28 MB`), deterministic composition-definition disposal (`172.26 MB`), and retained presentation on dismissal (`154.64 MB`). All experimental code changes were reverted; normal hide-time cleanup remains intact. Further fixes require a profiling decision under the repository's three-failed-fixes escalation rule.
- Final reviewed build: automated suite `190/190`; Standards review `0` remaining findings; Spec review `0` after fixes. Opening uses fitted monitor width before applying the derived tier and each `WM_SIZING` message retains its direction. Final installed artifact hash and direct `350 × 236` centered opening are recorded in `artifacts/automatic-growth-final-launch.json`; it has not been reprofiled after the three-failed-fixes escalation.
- Actual UI smoke observed popup Tab traversal to `Remove growth.txt`, post-growth focus on `Clear Temporary Items`, upward correction at work-area bottom `1032`, and corner selection of `220 × 240`. Growth-only popup-close causality is not separately established because interacting with Explorer can dismiss it first.
- Both available monitors were exercised at `96 DPI`: primary opening `(785,398,350,236)`, secondary `(2705,398,350,236)`. No non-100% DPI display is available. Reduced-motion, remaining exact lifecycle/rail scenarios, and full scale interaction acceptance are still incomplete; no pass is claimed. Evidence: `artifacts/automatic-growth-ui-smoke.json`.
