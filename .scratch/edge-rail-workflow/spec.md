# DropCove 0.0.5 — Edge Rail workflow

Status: ready-for-agent
Version: 0.0.5
Implementation approval: approved by user on 2026-10-13
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.5
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)
Placement decision: [ADR-0002](../../docs/adr/0002-edge-rail-placement-ownership.md)

## Problem Statement

DropCove 0.0.4 keeps a non-empty Shelf resident when the floating Drop Shelf is dismissed, but hides every shelf surface. The user must remember and invoke a global access path to see that retained content. Rail placement preferences, edge interaction, fullscreen policy and rail-native item actions described by the canonical product specification have no runtime owner.

## Solution

Add the Edge Rail as the non-activating presentation of a non-empty dismissed Shelf. A collapsed Rail Handle indicates retained content at the selected monitor edge. Hover or storage-drag entry expands a bounded, scrollable rail that reuses the existing Shelf operations for intake, pinning, removal, clearing, management and Copy drag-out. Open Shelf returns to the floating Drop Shelf on the monitor where the rail is currently docked.

Extend Settings through a clean 0.0.5 schema cutover with docking edge, target monitor and fullscreen policy. The Resident Application Coordinator owns `Hidden`, `Visible` and `EdgeDocked` transitions; Core Shelf State remains the only durable item/lifecycle owner. Monitor fallback, fullscreen hiding and all rail interaction remain event-driven and do not steal foreground focus.

## User Stories

1. As a user with retained Shelf Items, I want dismissing the Drop Shelf to show a Rail Handle, so that retained content remains discoverable.
2. As a user with an empty Shelf, I want dismissal to hide all shelf surfaces, so that an empty rail is never shown.
3. As a user, I want removing the final Shelf Item to hide every shelf surface immediately, so that stale UI is not left behind.
4. As a user, I want the rail on my configured left or right edge, so that placement matches my workspace.
5. As a user, I want Right to remain the default edge, so that first use has deterministic placement.
6. As a multi-monitor user, I want a connected configured monitor to take precedence, so that the rail stays where I explicitly chose.
7. As a user without an explicit monitor, I want the latest non-empty dismissal monitor remembered, so that the rail follows my recent workspace.
8. As a user whose saved monitor is disconnected, I want a temporary primary-display fallback, so that the rail remains accessible.
9. As a user reconnecting a saved monitor, I want my preference preserved, so that a temporary fallback does not rewrite intent.
10. As a user opening the Shelf from the rail, I want it on the monitor where the rail is currently docked, so that it does not jump to stale remembered placement.
11. As a user, I want the collapsed Rail Handle centered in the selected work area, so that it is predictable and avoids taskbar-reserved space.
12. As a user, I want the rail to overlay rather than reserve desktop work area, so that it does not rearrange other windows.
13. As a user, I want rail interaction not to activate DropCove, so that my current application keeps keyboard focus.
14. As a user, I want hovering over the handle to expand after 200 ms, so that accidental crossings do not constantly open it.
15. As a user, I want storage-drag entry to use the same 200 ms expansion delay, so that drop targets become available predictably.
16. As a user, I want pointer or drag leave to collapse after 300 ms, so that brief boundary crossings do not close the rail.
17. As a user, I want re-entry before the collapse deadline to cancel collapse, so that interaction is stable.
18. As a user, I want one through three batches to fit the minimum rail height, so that small shelves do not waste space.
19. As a user with more batches, I want bounded tiered growth and scrolling, so that the rail never exceeds 640 logical pixels.
20. As a user, I want new batches to grow an expanded rail, so that newly accepted content remains visible.
21. As a user, I want removal not to shrink an already expanded rail live, so that controls do not move under the pointer.
22. As a user, I want the next expansion to recalculate height, so that unused space is reclaimed later.
23. As a user, I want Manage Items or Clear confirmation to keep the rail expanded, so that an active operation is not collapsed underneath me.
24. As a user, I want to inspect batch summaries in the expanded rail, so that I can identify retained content quickly.
25. As a user, I want to drop files and folders into the rail, so that intake does not require opening the full Shelf.
26. As a user, I want duplicate and mixed-input behavior to match the Drop Shelf, so that intake semantics do not depend on surface.
27. As a user, I want to drag an entire batch from the rail with Windows Copy semantics, so that rail access is fully useful.
28. As a user, I want to drag an item from rail Manage Items, so that I can use one reference without opening the Shelf.
29. As a user, I want successful Temporary Item drag-out to consume only participating references, so that existing lifecycle rules remain consistent.
30. As a user, I want Pinned Items retained after successful drag-out, so that pinning means the same thing on every surface.
31. As a user, I want unavailable or canceled drag participants retained, so that rail use cannot lose references.
32. As a user, I want Pin/Unpin, Bulk Pinning, Remove Item and Remove Batch in the rail workflow, so that normal management remains available.
33. As a user, I want Clear Temporary Items with confirmation, so that bulk removal is deliberate and Pinned Items survive.
34. As a user, I want Open Shelf to show and activate the full Drop Shelf, so that detailed keyboard management remains available.
35. As a keyboard-only user, I want the existing global hotkey and Drop Shelf workflow to remain the keyboard entry point, so that the no-activate rail does not require a second global key hook.
36. As a fullscreen user, I want the requested rail hidden by default while its selected monitor is fully covered, so that the rail does not overlay fullscreen content unexpectedly.
37. As a user who enables Show over fullscreen applications, I want the rail to remain available over fullscreen windows, so that the policy follows my choice.
38. As a user leaving fullscreen, I want only the collapsed Rail Handle restored, so that stale expanded state does not reappear.
39. As a user entering fullscreen during Manage Items or Clear confirmation, I want those transient surfaces closed and unconfirmed work discarded, so that hidden controls cannot retain an operation.
40. As a user with an active native drag, I want fullscreen presentation changes not to cancel the drag, so that OLE owns drag completion safely.
41. As a user, I want display and DPI changes to reposition/reflow the rail without polling, so that residency remains efficient.
42. As a settings user, I want Docking edge, Target monitor and Show over fullscreen changes to remain drafts until Save, so that Cancel is side-effect free.
43. As a settings user, I want a failed Save to restore prior hotkey, startup, rail settings, settings file and active rail presentation where possible, so that configuration remains coherent.
44. As a user upgrading the development profile from 0.0.4, I want the old settings bytes preserved when the required Rail fields are absent, so that the clean-cutover warning path does not destroy evidence.
45. As a Windows user, I want an installed 0.0.5 artifact to exhibit the same Edge Rail behavior, so that development-only success is not treated as a release.

## Implementation Decisions

- Extend the Resident Application Coordinator display state to `Hidden`, `Visible` and `EdgeDocked`. Ordinary non-empty dismissal transitions to `EdgeDocked`; sign-in startup may still retain content while `Hidden`.
- Keep Core Shelf State as the only owner of Shelf Batches, Shelf Items, lifecycle, duplicate handling, availability, drag preparation/completion and serialized mutation. The rail consumes projections and requests existing mutations; it does not duplicate lifecycle logic.
- Use a separate Edge Rail window with no-activate native behavior. Rail interaction, hover, storage intake, drag-out and management must not take foreground focus from another application.
- The collapsed Rail Handle is 16 × 280 logical pixels. Expanded width is 280 logical pixels. Expanded height is `max(280, min(640, 68 × batch count + 68))`; row height is 64 and row gap is 4 logical pixels.
- Expansion waits 200 ms after hover or valid storage-drag entry. Collapse waits 300 ms after pointer/drag leave. Re-entry cancels the pending collapse. These timers are transient presentation state, not durable state.
- An open rail Manage Items flyout or Clear confirmation holds expansion. Removing the final Shelf Item closes rail transients and transitions to `Hidden` immediately.
- Reuse the established Shelf commands and Windows Copy drag seam for rail actions. Surface-specific code owns only projection, anchor geometry, expansion/collapse and no-activate window behavior.
- Rail Manage Items uses the same content and lifecycle as Drop Shelf Manage Items, constrained to rail width with a bounded vertical scroller. Pointer opening remains no-activate. Keyboard users open the Drop Shelf through the global hotkey rather than navigating the rail through a new global hook.
- Register the rail as a Windows storage drop target. Valid external storage drag follows the rail expansion delay; application-owned drag-out is not accepted as intake.
- Persist the actual monitor selected for every non-empty Drop Shelf dismissal in shelf state. A display-change fallback while already docked does not overwrite remembered placement.
- Resolve placement in this order: connected non-empty `RailMonitorId`; connected remembered dismissal monitor when `RailMonitorId` is empty; Drop Shelf monitor when no remembered monitor exists during dismissal; otherwise Windows primary display as temporary fallback. `RailEdge` selects Left or Right.
- Open Shelf uses the monitor where the rail is currently docked, including a temporary primary-display fallback. It then transitions to `Visible`, hides rail surfaces and activates the Drop Shelf.
- Rail bounds use logical dimensions converted through selected-monitor DPI and remain inside the work area. The rail is an overlay and never registers an application desktop work area.
- Extend the settings contract to exactly `HotKey`, `StartWithWindows`, `RailEdge`, `RailMonitorId` and `ShowRailOverFullscreen`. Defaults are `Right`, empty monitor identifier and `false` for fullscreen overlay.
- `RailEdge` accepts only `Left` or `Right`. `RailMonitorId` is either empty or a Windows display identifier. An unavailable saved display remains a valid preference and is not silently cleared.
- Apply a clean schema cutover. Missing any required 0.0.5 field, including a valid 0.0.4 file, follows the existing malformed-settings behavior: preserve bytes, load complete 0.0.5 defaults in memory, warn and replace only on successful explicit Save. No compatibility alias or migration is added.
- Extend the existing settings application transaction rather than adding a second settings coordinator. Candidate hotkey, startup and rail presentation/settings effects are committed together; failure restores the prior JSON/native/presentation state where possible and reports actual retained state when complete rollback is impossible.
- When Show over fullscreen applications is Off, the Edge Rail foreground tracker uses geometric full-monitor coverage on the selected monitor. It hides requested rail presentation while covered and restores only the collapsed handle afterward.
- Fullscreen hiding closes rail Manage Items and discards any unconfirmed Clear prompt. It does not cancel an already-running native drag.
- Display, device, DPI and foreground changes remain event-driven. No polling loop is introduced.
- Build only the 0.0.5 runtime and settings capability. Do not start Shake-to-open, recovery hardening or later visual-resource work.

## Testing Decisions

- The highest permanent seam remains the Resident Application Coordinator with real temporary Shelf and Settings stores. It exercises display transitions, selected/remembered monitor ownership, expansion state, fullscreen requests, final-item hiding and settings rollback as one product workflow.
- Native rail/window, monitor, clock, foreground/fullscreen and drag adapters are deterministic in coordinator tests. Timer tests use an injected/manual clock rather than sleeping.
- Core Shelf tests remain authoritative for duplicate intake, availability boundaries, Temporary/Pinned lifecycle, Clear, pinning, item/batch removal and drag preparation/completion. Rail tests assert that the rail routes through those outcomes, not copies them.
- Settings tests use the real temporary atomic JSON store plus deterministic hotkey/startup/rail integrations. Cover defaults, exact 0.0.5 serialization, unknown/missing fields, valid 0.0.4 input as clean-cutover malformed data, Save, Cancel and failures/rollback at each applied integration.
- Actual Windows smoke is required for no-activate focus preservation, OLE drop-in/drag-out, hover/leave timing, native monitor identifiers, DPI/work-area bounds, display changes, geometric fullscreen coverage, foreground tracking and installed NSIS behavior.
- UI verification exercises the actual Edge Rail surface: collapsed/expanded geometry, scrolling, flyout/confirmation hold-open, item/batch actions, drag feedback, Open Shelf placement and Settings controls. DOM-only mutation or process existence is not completion evidence.
- Permanent tests cover consumer-visible transitions and invariants. Do not add tests for command forwarding, copied projections, Tauri builder wiring, CSS source text or implementation wording.
- Release verification builds with cargo-xwin for `x86_64-pc-windows-msvc`, runs the newly compiled permanent suite directly on Windows, installs the current-user NSIS artifact into an owned location/profile and restores ambient installation/registry/shortcut state exactly.

## Out of Scope

- Shake-to-open, low-level pointer hooks, gesture sensitivity and Shake restoration sessions.
- Top or bottom rail docking.
- A setting to disable standard non-empty dismissal docking.
- Global keyboard traversal of the no-activate rail.
- Arbitrary multi-selection across Shelf Batches.
- Move/link drag effects, materializing virtual shell content or application-owned file copies.
- Automatic update, signing, database corruption recovery, profile deletion and installer shutdown hardening.
- Final icon/thumbnail resources, complete High Contrast polish and final accessibility-conformance audit beyond the existing required names/states/workflows.
- macOS or Linux support.

## Further Notes

- Approved clarification: Open Shelf follows the currently docked rail monitor, not stale remembered placement.
- Approved clarification: fullscreen hiding closes rail flyouts/unconfirmed Clear, restores only the handle and does not cancel active native drag.
- Approved clarification: keyboard access remains global hotkey → Drop Shelf; no rail-specific global keyboard hook.
- Approved testing seam: Resident Application Coordinator plus real temporary stores; native Windows smoke for focus, OLE, fullscreen, monitors and installed behavior.
- Approved delivery is five vertical slices with dependency graph `01 → 02 → 03 → 04 → 05`: display/handle/settings placement; interaction/drop-in; management/drag-out; fullscreen/transaction completion; installed 0.0.5 release. Read-only research or disposable prototypes may run in parallel, but shared-source implementation follows the graph.
