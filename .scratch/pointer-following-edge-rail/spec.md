# DropCove 0.0.18 — Pointer-following Edge Rail placement

Status: resolved
Version: 0.0.18
Design confirmation: approved by user on 2026-10-21 through `grill-with-docs`
Testing seam: deterministic native monitor-selection and Resident presentation regressions, plus actual installed Windows Resident/WebView2 smoke on two monitors observing native window bounds and the persisted latest dismissal monitor; explicitly approved by the user on 2026-10-21
Delivery prerequisite: completed — DropCove 0.0.17 is committed and accepted
Ticket decomposition: three slices approved by the user on 2026-10-21 — fresh dismissal placement, docked placement reconciliation, and release
Implementation approval: approved by the user on 2026-10-21 for Tickets 01 through 03; final release acceptance and Git commit authorization remain separate gates
Delivery status: completed — all three approved tickets are resolved; the user confirmed final 0.0.18 acceptance on 2026-10-21 and authorized the Git commit
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), especially Dismiss Drop Shelf and Edge Rail placement
Placement ownership: [ADR-0002](../../docs/adr/0002-edge-rail-placement-ownership.md)

## Problem Statement

On a multi-monitor system, dismissing a non-empty Drop Shelf can make the Edge Rail appear on a different display from the user's active pointer. The current automatic monitor mode gives the latest persisted dismissal monitor precedence over the pointer and the current Drop Shelf monitor. Once a secondary display has been remembered, later dismissals can keep returning the Rail Handle to that display even when the user invokes Dismiss Drop Shelf from another display.

This makes the Drop Shelf appear to disappear without leaving an Edge Rail in the user's current work area. The observed failure had the Drop Shelf on the primary display while the persisted monitor identifier selected the secondary display; the Rail Handle was correctly created but placed at the secondary display's far-right edge.

The current Settings label `Remembered` also describes persistence rather than the behavior the user wants. In automatic mode, the user expects each fresh Dismiss Drop Shelf action to choose the display containing the pointer, while persistence should exist only to restore an already-docked Edge Rail when no new dismissal has occurred.

## Solution

Replace the automatic monitor option label with **Follow pointer** and give a fresh non-empty Dismiss Drop Shelf action a dismissal-specific monitor policy:

1. A connected explicit Target monitor selected in Settings remains authoritative.
2. Otherwise, use the monitor containing the pointer at the instant Dismiss Drop Shelf begins.
3. If pointer-monitor resolution fails, use the Drop Shelf monitor, then the latest persisted dismissal monitor, then the Windows primary display.
4. Persist the actual monitor selected for the successful dismissal.

The Edge Rail does not continuously follow the pointer after docking. Fullscreen restoration, Shake-to-open cancellation, and other restoration without a fresh dismissal reuse the current docked placement. Selecting **Follow pointer** while the Edge Rail is already docked does not move it until the next dismissal. Selecting a fixed monitor moves a docked Edge Rail immediately.

If an explicit monitor is disconnected, DropCove preserves the explicit Settings value but temporarily applies the same pointer-first fallback. When that fixed monitor reconnects, a docked Edge Rail returns to it. In **Follow pointer** mode, disconnecting the monitor that currently owns the Edge Rail moves it to the monitor containing the pointer and records the actual fallback placement.

## User Stories

1. As a multi-monitor user, I want a newly dismissed non-empty Drop Shelf to leave its Rail Handle on the display where my pointer is, so that I do not have to search another display.
2. As a user dismissing with `Escape`, I want the pointer monitor sampled for that dismissal, so that keyboard dismissal matches my active display context.
3. As a user dismissing with the Drop Shelf `X`, I want the pointer monitor sampled for that dismissal, so that pointer dismissal and keyboard dismissal have the same placement outcome.
4. As a user dismissing with the global hotkey, I want the pointer monitor sampled for that dismissal, so that the hotkey does not send the Edge Rail back to a stale display.
5. As a user dismissing with the native Drop Shelf close action, I want the same Resident-owned policy, so that native close cannot diverge from the other dismissal paths.
6. As a user with an explicit Target monitor, I want that connected monitor to override the pointer, so that a deliberate fixed placement remains stable.
7. As a user whose explicit Target monitor is temporarily disconnected, I want DropCove to preserve my Settings choice while placing the Edge Rail on the pointer monitor, so that the Shelf remains reachable without losing my preference.
8. As a user whose fixed monitor reconnects, I want a docked Edge Rail to return to it automatically, so that my explicit preference becomes effective again.
9. As a user in automatic mode, I want the Settings option named `Follow pointer`, so that its behavior is clear before I save.
10. As a user switching from a fixed monitor to `Follow pointer`, I want the currently docked Edge Rail to stay where it is until the next dismissal, so that saving Settings does not cause an unexpected jump.
11. As a user selecting a fixed monitor while the Edge Rail is docked, I want the Edge Rail to move there immediately, so that the visible result confirms my explicit choice.
12. As a user changing the docking edge while selecting `Follow pointer`, I want the new edge applied on the current docked monitor without changing displays, so that monitor and edge decisions remain independent.
13. As a user leaving fullscreen mode, I want the Rail Handle restored on the monitor where it was already docked, so that restoration does not resample an unrelated pointer position.
14. As a user canceling Shake-to-open, I want the previous Edge Rail placement restored, so that a temporary Shelf presentation does not move the Rail Handle.
15. As a user moving the pointer after the Edge Rail docks, I want the Edge Rail to remain stationary, so that ordinary pointer movement does not make the surface jump between displays.
16. As a user whose current Follow pointer monitor disconnects, I want the Edge Rail moved to the available monitor containing my pointer, so that it remains reachable.
17. As a user restarting or restoring DropCove without a fresh dismissal, I want the latest actual dismissal monitor retained, so that the existing Edge Rail context is recoverable.
18. As a user dismissing again on another display, I want the new pointer monitor to replace the prior remembered dismissal monitor, so that persistence cannot override a newer action.
19. As a user whose pointer monitor cannot be resolved, I want DropCove to fall back to the Drop Shelf monitor, then the latest dismissal monitor, then the primary display, so that dismissal remains reliable.
20. As a user with an empty Drop Shelf, I want Dismiss Drop Shelf to keep all Shelf surfaces hidden, so that pointer-following placement does not create an empty Edge Rail.
21. As a user with retained Shelf Items, I want every dismissal path to preserve those references while changing only presentation placement, so that this fix cannot alter Shelf state.
22. As a keyboard-only user, I want the global hotkey and `Escape` path to produce the same Edge Rail placement as pointer dismissal, so that input method does not change the result.
23. As a user with fullscreen hiding disabled or enabled, I want the existing fullscreen policy preserved, so that monitor selection does not change visibility policy.
24. As a user updating from 0.0.17, I want the existing empty `RailMonitorId` value to become `Follow pointer` without a settings migration or reset, so that my other preferences remain unchanged.
25. As a user installing 0.0.18, I want the packaged application to reproduce the verified two-monitor behavior from embedded production assets, so that the fix is not limited to a development build.

## Implementation Decisions

- Keep Settings as the owner of the explicit `RailMonitorId` preference. A non-empty connected identifier remains authoritative.
- Keep Shelf state as the owner of the latest actual monitor selected by a successful non-empty dismissal. This persisted value is restoration state, not the first choice for a new automatic dismissal.
- Introduce a dismissal-specific monitor-selection operation rather than globally reordering the existing restore/settings monitor selector. A fresh dismissal and a restore are different domain events and require different precedence.
- For a fresh non-empty dismissal, use this order: connected explicit Target monitor; pointer monitor; Drop Shelf monitor; latest persisted dismissal monitor; Windows primary display.
- When an explicit Target monitor is disconnected, preserve its identifier in Settings and temporarily continue from the pointer-monitor step. Do not rewrite or clear the explicit preference.
- Sample the pointer monitor once per dismissal operation. The successful selected monitor flows through the existing Resident docking transaction and is persisted only as the actual latest dismissal monitor.
- All Dismiss Drop Shelf entry points continue converging on the same Resident coordinator. No frontend path may choose a monitor, move the Edge Rail directly, or reproduce the precedence rules.
- Retain the existing behavior for an empty Drop Shelf: hide the Drop Shelf and Edge Rail without selecting or persisting a Rail monitor.
- Non-dismissal restoration reuses the live docked placement. This includes fullscreen recovery, Shake cancellation, Rail expansion/collapse, content updates, and transient cleanup.
- Saving `Follow pointer` while the Edge Rail is docked preserves the current monitor. A simultaneous docking-edge change applies on that current monitor.
- Saving a connected fixed monitor while the Edge Rail is docked moves it immediately to that monitor.
- Display topology changes reconcile placement as follows: a connected explicit Target monitor wins and a reconnect returns the Rail there; otherwise, if the current Rail monitor disappeared, use the current pointer monitor and normal fallbacks, then retain the actual fallback placement.
- Change the empty Target monitor option label from `Remembered` to the exact text `Follow pointer`.
- Reuse the existing monitor enumeration, native cursor position, monitor identity, Edge Rail geometry, Resident presentation state, Settings transaction, and Shelf monitor persistence. Add no dependency or generalized window manager.
- Keep the existing settings schema. Empty `RailMonitorId` changes semantics and presentation label but requires no migration, compatibility layer, or duplicate state field.
- Keep `RailEdge`, fullscreen policy, Shake-to-open behavior, Shelf references, Profile Data, and source files unchanged.
- Synchronize all canonical version sources to `0.0.18` only in the release slice after the behavior slice is accepted.
- No glossary change is required. **Dismiss Drop Shelf**, **Drop Shelf**, **Edge Rail**, and **Rail Handle** already express the domain accurately.

## Testing Decisions

- The primary deterministic seam is the native monitor-selection and Resident presentation boundary. Tests assert externally meaningful placement outcomes and persisted selected monitor identifiers, not source routing or Win32 call counts.
- Add a regression for the reported pattern: two connected monitors, automatic mode, latest persisted dismissal monitor on display 2, pointer and Drop Shelf on display 1. A fresh dismissal must dock on display 1 and replace the persisted latest dismissal monitor.
- Prove connected explicit Target monitor precedence over pointer, Drop Shelf, and persisted dismissal monitors.
- Prove disconnected explicit Target monitor preserves the Settings identifier but temporarily chooses the pointer monitor.
- Prove pointer-resolution failure falls back through Drop Shelf monitor, persisted dismissal monitor, then primary display.
- Prove saving `Follow pointer` preserves the currently docked monitor while applying any docking-edge change.
- Prove saving a connected fixed monitor moves a docked Edge Rail immediately.
- Prove explicit-monitor reconnect returns a docked Edge Rail to the configured monitor.
- Prove Follow pointer hotplug moves an orphaned docked Edge Rail to the pointer monitor and retains the actual fallback placement.
- Preserve and extend existing monitor-selection, Edge Rail geometry, dismissal rollback, fullscreen, Shake restore, Settings transaction rollback, display reflow, and Profile Data safety regressions.
- Actual Windows smoke uses the real Resident Instance, WebView2 Drop Shelf, Edge Rail HWND, two connected monitors, and the installed production candidate. Observe exact HWND visibility and physical bounds after `X`, `Escape`, global hotkey, and native close dismissal.
- Actual smoke verifies that moving the pointer after docking does not move the Edge Rail, switching to `Follow pointer` does not move an already-docked Rail, selecting a fixed monitor does move it, and fixed-monitor disconnect/reconnect behavior is exercised where practical.
- Inspect the persisted latest dismissal monitor after successful dismissals without modifying ambient Profile Data. Use owned test Profile Data for mutation and cleanup.
- Run the complete native Windows regression executable directly, frontend typecheck/production build, cargo-xwin release and test compilation, NSIS packaging, embedded-assets smoke with the development server unavailable, install/reinstall/restart/Cooperative Shutdown/uninstall safety, and final human acceptance.
- External SDK missing-PDB linker warnings remain nonfatal and unsuppressed if no actual link failure occurs.

## Out of Scope

- Continuously moving the Edge Rail whenever the pointer crosses a monitor boundary.
- Moving the Edge Rail on ordinary fullscreen restore, Shake cancellation, expansion, collapse, or content updates when its current monitor remains connected.
- Persisting pointer coordinates or Drop Shelf coordinates.
- Adding a new Settings field, database column, migration, telemetry, dependency, monitor-management framework, or generalized placement policy.
- Changing the selected docking edge, Edge Rail size, topmost behavior, no-activate behavior, taskbar behavior, fullscreen policy, animation, appearance, or hit regions.
- Changing Drop Shelf initial placement, movement, sizing, drag-and-drop, item lifecycle, or dismissal semantics for empty content.
- Reworking Settings Save/Cancel transactions, global-hotkey registration, Shake-to-open detection, or display enumeration beyond the placement policy required here.
- Retrofitting the completed 0.0.17 specification or release evidence.
- Creating a Git commit without separate explicit user authorization after implementation, verification, release acceptance, and review.

## Further Notes

- The diagnosed application behavior was not a missing or failed Edge Rail HWND. The Rail Handle was visible at physical bounds `x=3824..3840` on the secondary display while the user was operating on the primary display.
- The current automatic option and ADR make the latest persisted dismissal monitor win over the current interaction context. This specification deliberately separates a fresh dismissal decision from restoration of an already-docked placement.
- `Follow pointer` describes selection at dismissal and disconnected-monitor recovery. It does not mean continuous tracking.
- ADR-0002 must be amended because the decision about automatic dismissal precedence changes while ownership remains the same.
- The Product Specification Edge Rail placement section must distinguish fresh dismissal selection from non-dismissal restoration.
- No new ADR is warranted: ADR-0002 already owns this decision and should record the revised tradeoff.
- Ticket decomposition and implementation approval remain separate from design confirmation and testing-seam approval.
- Git commit authorization remains separate from design, specification, tickets, implementation, verification, version synchronization, and release acceptance.

## Final human acceptance and commit authorization

The user confirmed `tất cả đã đạt, hãy commit các thay đổi` on 2026-10-21, accepting the exact candidate and authorizing the commit. Historical observation limits, physical coverage boundaries, and ambient installation accounting remain documented in the release ticket and roadmap.
