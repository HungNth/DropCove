# DropCove 0.0.6 — Shake-to-open

Status: ready-for-agent
Version: 0.0.6
Implementation approval: approved by user on 2026-10-14
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.6
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), section 7.2
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), section 13

## Problem Statement

DropCove 0.0.5 can be opened through launch, tray, global hotkey, or the Edge Rail, but it cannot appear beside an in-progress storage drag. The documented Shake-to-open gesture, its sensitivity controls, transient restoration rules, and hook-failure behavior have no runtime owner.

## Solution

Add one Windows low-level mouse observer that captures ordered pointer/button events with minimal callback work and posts them through a bounded queue to a deterministic axis-reversal detector. Low, Normal, and High sensitivity use the canonical movement and reversal thresholds, a 600 ms detection window, and a 500 ms cooldown.

When a qualifying gesture occurs while the left button is held and the Shelf is Hidden or EdgeDocked, the Resident Application Coordinator saves the prior presentation, shows the Drop Shelf near the pointer without taking the active drag, and starts one transient Shake session. A newly accepted Shelf Batch promotes that presentation to normal Visible state. Release, rejection, unsupported input, duplicate-only input, cancellation, false positive, or processing failure restores the saved Hidden or EdgeDocked presentation.

Extend the strict settings contract with `ShakeEnabled` and `ShakeSensitivity`. Save applies hotkey, startup, Edge Rail, Shake runtime, and the atomic JSON file as one rollback-safe transaction. Hook installation failure warns but never disables tray, hotkey, Drop Shelf, Edge Rail, or existing storage workflows.

## User Stories

1. As a user dragging files, I want rapid back-and-forth pointer movement to reveal the Drop Shelf near the pointer.
2. As a user, I want the gesture to require the left mouse button so normal pointer movement cannot open DropCove.
3. As a user, I want Low, Normal, and High sensitivity to use predictable documented thresholds.
4. As a user, I want slow movement outside the 600 ms window and retriggers inside the 500 ms cooldown rejected.
5. As a user whose Shelf was Hidden, I want a false positive or canceled drag to return it to Hidden.
6. As a user whose Shelf was EdgeDocked, I want a false positive or canceled drag to restore the same rail placement and presentation.
7. As a user dropping a new supported batch, I want the Drop Shelf to remain visible near the pointer after acceptance.
8. As a user dropping only duplicates or unsupported input, I want no Shelf mutation and restoration of the prior presentation.
9. As a settings user, I want Shake-to-open enabled by default with Normal sensitivity and draft-only Enable/Sensitivity controls.
10. As a settings user, I want failed hook enable/reconfiguration or later Save failure to restore the prior runtime and exact settings bytes where possible.
11. As a user whose pointer hook cannot be installed, I want a warning while every non-Shake workflow remains usable.
12. As a Windows user, I want the installed 0.0.6 artifact to exhibit the same behavior as the verified development build.

## Implementation Decisions

- Add one `windows_shake` adapter. It owns `WH_MOUSE_LL`, teardown, a preallocated bounded ordered queue, native message dispatch, and the pure detector on a dedicated event-driven hook thread. Detector results marshal to the existing Tauri UI thread for all Resident, Window, and Settings operations; no polling or new dependency is introduced.
- The hook callback records only pointer coordinates, event kind, flags, and event time; it queues work and always continues the hook chain.
- Detector samples choose the dominant movement axis. A reversal counts only when the accepted sample remains on that axis and changes sign after meeting the selected minimum movement. Changing dominant axis starts a new reversal sequence.
- Profiles are fixed by the canonical specification: Low = 24 px / 4 reversals, Normal = 10 px / 2 reversals, High = 8 px / 2 reversals. The detection window is 600 ms and cooldown is 500 ms.
- The Resident Application Coordinator owns the transient Shake session. No fourth durable display state is added.
- A Shake session records prior `Hidden` or `EdgeDocked` state, exact rail placement, and rail presentation. Native showing during an active drag is no-activate; successful intake promotes it to normal `Visible` presentation after OLE drop completion.
- Existing Core Shelf State remains authoritative for supported paths, mixed input, duplicate-only input, persistence, ordering, and mutation. Shake code consumes `DropResult`; it never copies intake rules.
- Main-window OLE callbacks mark Shake drag entry/leave/drop boundaries before asynchronous persistence so button release and drop completion resolve one session deterministically.
- Extend the strict settings object to exactly seven required fields: `HotKey`, `StartWithWindows`, `RailEdge`, `RailMonitorId`, `ShowRailOverFullscreen`, `ShakeEnabled`, and `ShakeSensitivity`. Defaults are enabled and Normal.
- A valid 0.0.5 settings file is malformed under the 0.0.6 clean cutover: retain original bytes, load complete defaults, warn, and replace only after explicit successful Save. No migration or compatibility alias is added.
- Extend the existing settings application transaction with Shake snapshot/apply/restore/description operations. Do not add a second settings coordinator.
- Startup hook failure is nonfatal. Desired settings remain visible for retry; tray, hotkey, Shelf, Rail, and drag operations continue.
- No ADR is added: the Windows adapter follows the already-approved architecture and introduces no hard-to-reverse product ownership decision.

- Equal absolute X/Y displacement selects the X axis. Sub-threshold samples do not advance the accepted-point baseline.
- The 600 ms window starts on the first accepted direction sample after left-button down or sequence reset. Expiry discards reversals, uses the current point as the new baseline, and waits for the next threshold-crossing movement to start a new window. Left-button up resets the sequence.
- Queue overflow discards the incomplete movement sequence while preserving the latest observed button state, so missing interior samples cannot manufacture a reversal.

## Vue Component Map

- `SettingsView.vue`: settings-window composition and authoritative rollback of control values after rejected draft operations.
- `ShakeSettingsFields.vue`: one focused section exposing enabled and sensitivity props and typed change events.
- `useSettings.ts`: typed Settings snapshot and Tauri command calls; no gesture logic in the frontend.

## Testing Decisions

- Pure detector tests use synthetic timestamps and points; no sleeps. Cover each profile threshold, left-button gating, dominant-axis reversals, 600 ms expiration, 500 ms cooldown, and bounded queue overflow reset.
- Resident coordinator tests use deterministic window/rail adapters. Cover Hidden restore, exact EdgeDocked restore, accepted-batch promotion, duplicate/rejected restoration, and native failure rollback.
- Settings tests use the real temporary atomic JSON store and a deterministic integration adapter. Cover exact seven-field schema, 0.0.5 byte preservation, Save/Cancel, hook apply failure, file failure after hook effect, complete rollback, and retained-state warning.
- Actual Windows smoke is required for physical/injected mouse-hook delivery, no-activate presentation during a drag, near-pointer placement, OLE accepted/duplicate/unsupported outcomes, hook enable/disable, actual Settings controls, clean shutdown, and installed NSIS behavior.
- Release verification uses the newly built 0.0.6 artifact and an owned profile/install location. Ambient installation, profile, registry, shortcuts, and unrelated processes must be restored exactly.

## Out of Scope

- Proving that a detected left-button gesture belongs to a file drag before the Drop Shelf receives OLE data.
- Raw Input replacement, polling, telemetry, adaptive sensitivity, custom thresholds, per-device profiles, or top/bottom Edge Rail placement.
- Final shell visuals, thumbnails, virtualization, complete accessibility conformance, recovery, updater, signing, or installer lifecycle hardening planned for later versions.
- macOS or Linux support.
