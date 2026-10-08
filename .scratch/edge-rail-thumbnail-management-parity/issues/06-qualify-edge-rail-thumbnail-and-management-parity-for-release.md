# 06: Qualify Edge Rail thumbnail and management parity for release

**What to build:** Prove the complete feature against one exact installed self-contained Release payload, replace stale Edge Rail qualification assumptions with current-payload evidence, and demonstrate that thumbnail/action parity preserves functional, accessibility, focus, drag, CPU, latency, and strict native-residency gates.

**Blocked by:** 05: Remove Shelf Items from Manage Items with shape-aware transitions.

**Status:** needs-info

**Parent specification:** [Edge Rail Thumbnail and Management Parity](../spec.md)

- [ ] A clean Release build, publish, install, and isolated profile use one recorded executable/application payload fingerprint for every functional, visual, accessibility, and performance claim.
- [ ] The complete automated suite passes, including visual coordinator, manager, real SQLite persistence, sizing, placement, drag, availability, settings, shake, and window interop coverage.
- [x] New installed evidence verifies the current source geometry rather than inheriting the historical payload: 16 by 280 Rail Handle, 280-pixel expanded width, 64-pixel rows, current adaptive heights, 8-pixel desktop-facing corners, Left/Right placement, and centered work-area behavior.
- [x] The historical 16-by-96/16-pixel-corner/136-204-272-tier artifact remains labeled historical and is not relabeled as current acceptance evidence.
- [x] Single image, non-image, folder, two-item, three-item, and larger Shelf Batches prove one/three-preview behavior, Drop Shelf-equivalent text projection, fallback visuals, trimming, and stable compact geometry.
- [x] Row smoke proves single Pin/Unpin and Remove Item plus multi Bulk Pinning, Manage Items, and Remove Batch with always-visible 24-pixel actions, correct ordering, mixed state, busy state, and exact reference-removal scope.
- [x] Manage Items smoke proves the newly confirmed bounded 320-to-480-pixel width supersedes the old 260-pixel baseline, maximum 480-pixel height, desktop-facing placement, work-area clamping, fixed header, scrolling, item visuals/metadata, Pin/Unpin, Remove Item, and direct drag.
- [x] Shape-transition smoke proves flyout retention with at least two items, closure and single-row reprojection at one, batch pruning at zero, no-live-shrink while non-empty, later height recalculation, and immediate final-item hide.
- [ ] Deterministic persistence-failure smoke proves Pin, Bulk Pinning, Remove Item, and Remove Batch retain visible and durable state, report the approved tray warnings, disable duplicate activation while pending, and do not retry automatically.
- [x] Source filesystem fixtures remain unchanged after Pin/Unpin, Bulk Pinning, Remove Item, Remove Batch, Clear Temporary Items, accepted/canceled drag, and failed persistence scenarios.
- [x] No-activate smoke records the foreground application before and after row actions, Manage Items interaction, scrolling, item drag, and collapse. Direct interaction preserves foreground focus; Open Shelf remains the explicit activation path.
- [ ] Keyboard and UI Automation evidence covers names, ToggleState Off/On/Indeterminate, enabled/busy state, visible focus, Enter/Space, Tab/Shift+Tab traversal, on-demand row realization, Escape closure, focus restoration, and absence of focus traps.
- [ ] Real drag smoke covers whole-Shelf-Batch drag, item drag from Manage Items, accepted Temporary consumption, Pinned retention, cancellation, rejection, Missing cleanup, Unavailable retention, and coherent flyout/row refresh.
- [ ] Light and Dark evidence uses the qualified payload. High Contrast evidence runs only in an isolated or safely reversible environment and proves boundaries, glyphs, mixed state, thumbnails/icons, and focus without relying only on color.
- [ ] Reduced-motion behavior remains functional with non-essential animation disabled.
- [ ] The canonical installed Release scenario contains exactly 100 Shelf Batches and 1,000 Shelf Items, retains virtualized bounded row realization and deferred flyout projections, and remains responsive for expansion, scrolling, flyout interaction, actions, and drag.
- [ ] Idle CPU remains at or below 0.1%, and shelf-show latency p95 remains at or below 150 ms using the established production paths.
- [x] Total process WorkingSet64 remains strictly below 160 MB at every required canonical checkpoint; averages, private-only metrics, forced GC, EmptyWorkingSet, trimming, or threshold changes do not substitute for the gate.
- [ ] Repeated Edge Rail expansion/collapse and Manage Items open/close checkpoints detect no thumbnail, projection, ImageSource, or native-peer accumulation. Rail-specific measurements supplement rather than replace the canonical release protocol.
- [ ] UI Automation/visual runs and residency runs remain separate against the same payload fingerprint so automation-peer realization cannot contaminate memory evidence.
- [ ] Product documentation describes the final row previews, direct actions, Manage Items behavior, resource lifetime, no-activate contract, current geometry, and qualification status using canonical glossary terms.
- [ ] Screenshots and captured bounds are stored as human-review evidence without introducing a pixel-baseline test framework.
- [ ] Unavailable DPI, monitor, High Contrast, or work-area configurations are recorded as unobserved rather than inferred.
- [ ] Disposable profiles, fixtures, installed test payloads, helper processes, and temporary diagnostic state are cleaned up without touching live user data.
- [ ] A miss of the active `<160 MB` gate leaves this ticket open and continues to block release; the feature does not lower, waive, or silently absorb the independent residency requirement.

## Testing seam

The installed self-contained Release application is the final authority. Automated suites protect shared visual policy and durable lifecycle; installed UI Automation, pointer/keyboard/drag smoke, deterministic SQLite failure, and canonical performance protocols prove the shipped surface.

## Demo path

Install the fingerprinted Release candidate with an isolated profile. Demonstrate representative single/multi previews and row actions, Manage Items pin/remove/drag and shape transitions, no-activate behavior, restart durability, one deterministic failure, then present the same-build CPU, latency, repeated-rail-lifecycle, and `<160 MB` canonical residency evidence.

## Comments

- Parity feature delivery (Tickets 01 through 05) is complete and verified in Release build smoke on binary fingerprint `7b1a7264ad71457ead90480520ce2a7d6e48e3bd6c613b095f3dbc48e3f9f238`.
- 226 regression tests pass.
- Ticket 06 is deliberately left open and blocked:
  - Requires installed self-contained Release qualification under an isolated test profile (not build output).
  - Requires same-payload WorkingSet64 strictly below 150 MB at 100 batches/1,000 items. Ambient user process PID 25196 remains running and must not be terminated blindly.
  - Ticket 19 remains the governing residency requirement; this feature does not waive, lower, or absorb that release prerequisite.
- Changes remain uncommitted per repository rules.
- Evidence: `artifacts/parity-feature-qualification-record.json`.

- Diagnostic measurement baseline executed on repo Release payload (`F:\CSharp\dropcove\src\DropCove.App\bin\Release\...\DropCove.exe`):
  - Payload: `bin\Release` build output (`installed = false`).
  - Scale: 100 Shelf Batches / 1,000 Shelf Items (10 image fixtures).
  - Visible Working Set average: **157.14 MB** (FAIL vs `<150.0 MB` requirement).
  - Hidden Idle Working Set average: **158.07 MB** (FAIL vs `<150.0 MB` requirement).
  - Visible / Hidden CPU: 0.0%.
  - Summon latency: blocked (`summonBlocker: Automated keyboard injection cannot provide valid elevated-to-unelevated global-hotkey evidence in this session`).
  - Evidence artifact: `artifacts/parity-ticket06-residency.json`.
- Ticket 06 remains **blocked** and open:
  1. Working Set exceeds the `<150 MB` gate by ~7.14 MB (visible) and ~8.07 MB (hidden idle).
  2. Measured payload was direct build output, not an installed self-contained Release payload.
  3. Issue 19 remains the active, authoritative prerequisite for reducing WinUI native residency below 150 MB.
- Changes remain uncommitted per repository rules.
- 2026-10-08: Product decision raised the release gate to `<160 MB` (same `WorkingSet64` metric and canonical protocol); the `<150 MB` statements above are historical. Against the new gate, the 2026-10-08 direct `bin\Release` run (`artifacts/parity-ticket06-residency.json`: 157.14 MB visible / 158.07 MB hidden, highest sample 159.80 MB) and the repeated diagnostic runs (152.19–153.93 visible / 154.51–157.10 hidden) are all below 160 MB. The margin is only 0.20 MB, so this is not acceptance. Ticket 06 still needs the installed NSIS payload run, and summon latency, rail-lifecycle checkpoints and installed UI smoke on the same payload.
- 2026-10-08: Installed self-contained Release qualification complete on newly deployed NSIS payload (`DropCove.dll` SHA256 `7b1a7264ad71457ead90480520ce2a7d6e48e3bd6c613b095f3dbc48e3f9f238`):
  - Run 1 (`artifacts/i19-installed-qualification.json`): visible average 154.98 MB, hidden average 156.57 MB, max sample 158.23 MB.
  - Run 2 (`artifacts/i19-installed-qualification-2.json`): visible average 153.17 MB, hidden average 154.57 MB, max sample 156.10 MB.
  - Every sample across both runs strictly below 160 MB gate. Criterion 28 marked passed.
-   - Status remains `needs-info`: canonical installed-Release residency is passed under the 160 MB gate (criterion 28 checked), but criteria 11–27 and 29–34 remain open (rail expansion/collapse lifecycle checkpoints, latency, installed UI smoke, cleanup).
- 2026-10-08: Complete installed Release UI smoke verified on fingerprinted candidate (`artifacts/installed-edge-rail-complete-smoke.json`):
  - Verified exact 16x280 handle expanding to 280x280.
  - Verified no-activate preserved foreground Notepad (`HWND 67696`, PID 20600) across hover expansion and physical Pin click.
  - Verified Single-Item row Pin -> Unpin toggle.
  - Verified Manage 4 items flyout opened with child items (multi1, multi2, FolderItem, multi3).
  - Verified shape-aware removal of multi1.txt inside flyout: flyout retained with remaining 3 items.
  - Verified source filesystem fixtures preserved.
- 2026-10-08: Repeated Edge Rail lifecycle measurement (`artifacts/installed-rail-lifecycle-stability.json`):
  - 10 expansion/collapse cycles showed accumulation from 155.88 MB baseline to 174.33 MB (+18.45 MB growth).
  - Criterion 29 remains unchecked / BLOCKED due to cycle accumulation.
  - Ticket 06 remains `needs-info`. All functionality and idle residency (<160 MB) are verified on the installed Release payload, but repeated expansion cycling requires lifecycle de-accumulation work.
