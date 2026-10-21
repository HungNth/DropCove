# 02: Preserve and reconcile docked Edge Rail placement

**What to build:** Keep an already-docked Edge Rail stable when no new dismissal occurs, while making explicit monitor Settings and display topology changes reconcile predictably. `Follow pointer` samples the pointer only for a fresh dismissal or when the current monitor disappears; fixed monitor choices apply immediately and resume automatically when reconnected.

**Blocked by:** 01: Follow pointer on Dismiss Drop Shelf.

Status: resolved

- [x] Saving `Follow pointer` while the Edge Rail is docked preserves its current monitor until the next fresh Dismiss Drop Shelf action.
- [x] A simultaneous docking-edge change while saving `Follow pointer` applies on the current docked monitor without selecting a different display.
- [x] Saving a connected explicit Target monitor while the Edge Rail is docked moves it immediately to that monitor and preserves the configured docking edge.
- [x] When an explicit Target monitor disconnects, its Settings identifier remains unchanged and the docked Edge Rail temporarily uses the monitor containing the pointer, with the approved fallback order if pointer resolution fails.
- [x] When that explicit Target monitor reconnects, a docked Edge Rail returns to it automatically without rewriting Settings or Shelf references.
- [x] In `Follow pointer` mode, disconnecting the monitor that currently owns the Edge Rail moves it to the available monitor containing the pointer and retains the actual fallback placement.
- [x] Ordinary pointer movement after docking never moves the Edge Rail between monitors.
- [x] Fullscreen hide/restore, Shake-to-open cancellation, expansion/collapse, content updates, transient cleanup, and Settings rollback reuse the live docked placement rather than resampling the pointer while that monitor remains connected.
- [x] Open Shelf continues to open on the monitor where the Edge Rail is currently docked, including a temporary fallback monitor.
- [x] Native presentation or persistence failures do not leave Resident visibility, live placement, Settings, or latest dismissal monitor claiming a transition that did not complete; existing rollback guarantees remain intact.
- [x] Deterministic regression coverage proves fixed Save, Follow pointer Save without movement, edge-only Save, explicit disconnect/reconnect, Follow pointer hotplug, non-dismissal restore stability, Open Shelf placement, and failure rollback.
- [x] Actual Windows two-monitor smoke proves the docked Edge Rail stays stationary after pointer movement and non-dismissal restoration, moves immediately for a fixed Save, waits for the next dismissal after switching to `Follow pointer`, and reconciles available disconnect/reconnect scenarios.
- [x] Existing Edge Rail geometry, fullscreen policy, no-activate/topmost behavior, hit regions, animation, drag/drop, Rail Handle sizing, and source/Profile Data safety remain unchanged.
- [x] No new schema, migration, dependency, telemetry, persisted pointer coordinates, or continuous pointer-following loop is added.
- [x] No Git commit is created without separate explicit user authorization.

## Comments

- 2026-10-21: Resolved. Actual final-payload Settings Save moved the docked Rail immediately to a fixed secondary monitor, retained its secondary bounds when switching to `Follow pointer` with the pointer on primary, applied a left-edge change without changing monitor, and Open Shelf returned to that secondary monitor. SQLite retained the actual dock monitor separately from the empty Settings target. Ordinary pointer movement left bounds unchanged. Raw assertions: `build/smoke_final_candidate.log`.
- Independent review found and corrected inverted initial-restore precedence and incomplete rollback coverage. The shared production coordinator now reads prior durable state before native mutation and restores native/live/persisted placement after SQL failure or a native move followed by projection failure. The latter regression was observed RED (`monitor_b` versus expected `monitor_a`), then GREEN. Full suite: 118/118; focused database-teardown check: 1/1.
- Physical cable disconnect/reconnect, mixed/fractional DPI, and physical fullscreen/Shake gesture restoration were not re-exercised. Deterministic reconciliation, fullscreen, and Shake regressions passed; those results are not physical surface proof. Existing restore callsites remain unchanged.
