# DropCove 0.0.4 — Resident access and settings delivery map

Version: 0.0.4
Implementation approval: approved by user on 2026-10-13
Parent: [Specification](spec.md)
Progress source: [Roadmap](../../docs/ROADMAP.md), version 0.0.4
Research: [Windows Resident Integration](research/windows-resident-integration.md)

## Notes

- [01: Profile-scoped Resident Instance and Activation](issues/01-resident-instance-activation.md) — `resolved`; native behavior verified and interface staging approved.
- [02: Notification-area Resident Access and Exit](issues/02-notification-area-residency.md) — `resolved`; native tray/Exit/foreground repair and installed warning delivery verified.
- [03: Global Hotkey and Settings](issues/03-global-hotkey-settings.md) — `resolved`; user physical retest passed, 55 tests and remaining native Enter/Space/Save-disabled checks passed.
- [04: Start with Windows and Hidden Start](issues/04-start-with-windows-hidden-start.md) — `resolved`; target release build, 66 tests and native startup/rollback/retry smoke passed.
- [05: Release 0.0.4](issues/05-release-0.0.4.md) — `resolved`; version-consistent cargo-xwin NSIS, 66 tests, installed native settings/startup/warning/rollback and exact cleanup verified.

## Decisions-so-far

- Use one Resident Application Coordinator as the permanent high testing seam and resident lifecycle owner; Core Shelf State remains the durable item/sizing owner.
- Scope instance identity by resolved DropCove profile. Secondary launch sends an Activation Request and exits without opening another profile database or creating tray/hotkey/startup resources.
- Version 0.0.4 temporarily maps non-empty Shelf dismissal to `Hidden`; all references remain durable. Version 0.0.5 replaces this staging behavior with the canonical Edge Rail transition.
- Normal launch, secondary activation and tray actions always show/activate on the cursor monitor. The hotkey alone toggles `Hidden ↔ Visible`.
- Settings UI and persisted schema contain only Global hotkey and Start with Windows in 0.0.4. Rail and Shake settings arrive through clean cutovers with their owning runtime versions.
- Keep the old hotkey active while editing/recording. Suppress its activation only while the recorder listens; Save tests the candidate before replacing active integration.
- Missing settings load defaults silently. Malformed/unreadable settings remain untouched, load defaults in memory and produce a warning. Explicit successful Save performs atomic replacement.
- Failed default startup registration does not end residency or rewrite desired Start with Windows Off; warning + later Save retry is the retained behavior.
- Permanent coordinator tests use deterministic native adapters plus a real temporary settings file. Single-instance, tray, RegisterHotKey, Run registration, monitor/DPI, foreground/topmost and installer behavior require actual Windows smoke.
- Ticket chain is intentionally linear because each slice exposes the entry point required by the next: resident activation → tray → Settings/hotkey → startup transaction → release.
- Existing version 0.0.3 implementation/release history and commits `0cee503`, `e2fefcc`, `4eed9bb` remain closed. Ticket 05 advanced release metadata to 0.0.4.
- No ADR is needed: the approved Hidden behavior is temporary staging and active-settings-only persistence is an additive version cutover, not a hard-to-reverse system choice.
- Ticket 01 resolved after the user approved interface staging: launch/show/hide in 01, warning/Exit in 02 and Settings in 03. Evidence: 44 integrated tests pass and owned native activation/hide/retention/bounds/failure smoke. See [ticket 01 answer](issues/01-resident-instance-activation.md#answer).
- Ticket 02 resolved: 47 integrated tests; actual Shell left/double/right-click, clean Exit, foreground focus preservation, exact exit/restart retention, installed secondary activation and Windows toast history delivery. See [ticket 02 answer and visual limits](issues/02-notification-area-residency.md#answer). Tauri owns tray/menu lifetime; maintained notification plugin supplies the missing warning capability.
- Recorder bug was a real installed-app defect, not a harness-only limitation. Removed the global capture hook/thread; focused control input plus existing WM_HOTKEY identity captures chords under authoritative backend validation. See [updated ticket 03 evidence](issues/03-global-hotkey-settings.md#evidence-and-manual-retest). The repaired 0.0.3-metadata test build remains the user's installation; the separate 0.0.4 release was verified in ticket 05.
- Ticket 03 resolved after the user confirmed installed Step 5 passed. The remaining native Enter/Space initiation and Save-disabled checks also passed on an isolated profile. See [ticket 03 answer](issues/03-global-hotkey-settings.md#answer); subsequent startup and release tickets 04–05 are also resolved.
- Ticket 04 resolved: full settings schema, application-owned HKCU Run, hidden launch and transactional rollback/retry. Final-source cargo-xwin release build succeeded and 66 freshly compiled native tests passed. See [ticket 04 evidence and limits](issues/04-start-with-windows-hidden-start.md#answer).

## Fog

- Tickets 01–05 are resolved. Release 0.0.4 is complete; 0.0.5+ remains unstarted pending user review.
- Exact use of maintained Tauri plugins versus small native adapters is an implementation choice constrained by profile-scoped identity, rollback, existing dependencies and the cited primary-source behavior. No plugin default may weaken the profile-specific contract.
- Actual mixed-DPI/multi-monitor, notification UI and occupied-hotkey coverage depend on available Windows hardware/system state; release evidence must name unobserved limits rather than infer them.
- Startup smoke must preserve/restore any pre-existing owned current-user Run value and use known process/registry handles; ambient settings/profile data remain out of bounds.
