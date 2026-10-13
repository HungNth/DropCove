# DropCove 0.0.4 — Resident access and settings

Status: ready-for-agent
Version: 0.0.4
Implementation approval: approved by user on 2026-10-13
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.4
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md)
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md)

## Problem Statement

DropCove 0.0.3 provides a durable Drop Shelf, but its process lifetime is still tied to the visible window and it has no resident access points or user-configurable global integrations. A second launch can create another process, closing the Shelf can terminate access, users cannot open it from a notification-area icon or hotkey, and sign-in startup/settings are absent.

## Solution

Turn DropCove into one profile-scoped Resident Instance that owns durable Shelf state, transient windows, global integrations and warning notifications. Normal and secondary launches, the notification-area icon and the active hotkey all route through one show/hide coordinator with cursor-monitor placement. Add a Settings window for Global hotkey and Start with Windows, backed by atomic JSON persistence and rollback-safe application of Windows integrations.

During this staged version, hiding a non-empty Shelf uses `Hidden` while retaining every reference. Version 0.0.5 replaces that temporary transition with the canonical Edge Rail workflow. Rail and Shake controls are not exposed before their runtime owners ship.

## User Stories

1. As a Windows user, I want one Resident Instance per DropCove profile, so that global integrations and Shelf state have one owner.
2. As a Windows user, I want a second launch to signal the existing process and exit, so that duplicate residents do not compete for the same profile.
3. As a Windows user, I want a second launch to show and activate the existing Shelf, so that launching DropCove remains a reliable access method.
4. As a user with isolated profiles, I want single-instance identity scoped by the resolved profile, so that independent owned profiles do not block each other.
5. As a Windows user, I want closing or hiding the Shelf to keep DropCove resident, so that tray and hotkey access remain available.
6. As a Windows user, I want hiding a non-empty 0.0.4 Shelf to preserve every reference, so that the deferred Edge Rail does not cause data loss.
7. As a Windows user, I want an explicit Exit DropCove command, so that I can terminate the resident process deliberately.
8. As a Windows user, I want a notification-area icon while DropCove is resident, so that I can reopen or exit it without another executable launch.
9. As a Windows user, I want left-click or double-click on the notification-area icon to show the Shelf, so that common tray interactions produce the same result.
10. As a Windows user, I want a right-click menu with Settings and Exit DropCove, so that resident controls have one predictable home.
11. As a Windows user, I want tray/launch activation always to show rather than toggle, so that an access action never unexpectedly hides the Shelf.
12. As a Windows user, I want the global hotkey to show a hidden Shelf and hide a visible Shelf, so that one shortcut acts as a predictable toggle.
13. As a Windows user, I want the default `Ctrl + Shift + Space` shortcut attempted on first launch, so that keyboard access works without setup.
14. As a Windows user, I want hotkey registration failure to leave DropCove resident and warn me, so that an optional integration cannot prevent access through tray or launch.
15. As a Windows user, I want Settings to display the currently desired shortcut, so that I can understand and change keyboard access.
16. As a Windows user, I want shortcut recording to begin by click, Enter or Space, so that pointer and keyboard workflows are both supported.
17. As a Windows user, I want shortcut recording to require a modifier and supported primary key, so that invalid bare-key global shortcuts are rejected before Save.
18. As a Windows user, I want modifiers displayed in `Ctrl + Alt + Shift + Win + key` order, so that shortcut text is stable.
19. As a Windows user, I want letters, digits, function keys, navigation/edit keys, OEM punctuation and modified Tab/Enter/Space supported, so that useful standard shortcuts are recordable.
20. As a Windows user, I want Escape, bare Tab, focus loss and Settings deactivation to cancel recording, so that recording cannot trap navigation.
21. As a Windows user, I want the existing active hotkey retained while recording and before Save succeeds, so that editing a draft does not remove access.
22. As a Windows user, I want activation from the existing hotkey suppressed while the recorder is listening, so that pressing that combination records the draft instead of toggling the Shelf.
23. As a Windows user, I want Save disabled while the recorder is listening, so that an incomplete shortcut cannot be applied.
24. As a Windows user, I want Windows conflicts reported in Settings while the old shortcut stays active, so that a rejected shortcut cannot break access.
25. As a Windows user, I want Settings changes to remain drafts until Save succeeds, so that Cancel or close discards them completely.
26. As a Windows user, I want only one Settings window, so that multiple drafts cannot compete.
27. As a Windows user, I want Settings reopened from the tray to show/focus the existing Settings window, so that draft ownership stays clear.
28. As a Windows user, I want Start with Windows enabled by default, so that DropCove becomes available after sign-in.
29. As a Windows user, I want Start with Windows to add or remove the current-user registration only after Save, so that drafts have no external effect.
30. As a Windows user, I want a sign-in start to restore state and integrations without showing Shelf surfaces, so that startup remains unobtrusive.
31. As a Windows user, I want tray, hotkey or another launch to show DropCove after hidden sign-in start, so that the resident remains accessible.
32. As a Windows user, I want startup registration failure to keep desired Start with Windows On and show a warning, so that DropCove does not silently rewrite my choice.
33. As a Windows user, I want a later Save to retry a failed startup registration, so that temporary system errors can be corrected.
34. As a Windows user, I want missing settings to load defaults without a warning, so that a first launch is clean.
35. As a Windows user, I want malformed or unreadable settings left untouched while defaults load in memory, so that DropCove does not destroy evidence or silently overwrite configuration.
36. As a Windows user, I want malformed/unreadable settings to produce a resident warning, so that fallback is visible.
37. As a Windows user, I want successful settings saves to atomically replace `settings.json`, so that a crash cannot leave a partially written configuration.
38. As a Windows user, I want a failed settings application to retain the previous active hotkey, startup registration and settings file, so that Save is rollback-safe.
39. As a Windows user, I want incomplete rollback reported with the actual retained safe state, so that the Settings view does not make a false claim.
40. As a Windows user, I want launch, tray and hotkey activation placed on the monitor containing the cursor, so that DropCove appears where I am working.
41. As a Windows user, I want restored logical dimensions converted for the selected monitor DPI and clamped to its work area, so that the Shelf remains usable across displays.
42. As a Windows user, I want foreground changes to reassert topmost ordering without activating DropCove, so that other applications keep keyboard focus.
43. As a Windows user, I want monitor/display changes handled by events rather than polling, so that residency does not consume resources continuously.
44. As a Windows user, I want integration warnings available through the notification area, so that recoverable failures do not require fatal startup dialogs.
45. As a Windows user, I want an installed 0.0.4 artifact to exhibit the same resident/settings behavior, so that development-only success is not treated as a release.

## Implementation Decisions

- Add one Resident Application Coordinator as the resident lifecycle interface. It owns resolved profile identity, primary/secondary launch outcome, display state, show/hide/open-settings/exit actions, current Settings state and warning publication. Presentation and adapters remain behind it rather than calling one another directly.
- Resolve and normalize the DropCove profile before acquiring the profile-scoped single-instance identity. The primary Resident Instance owns durable state and global integrations; a secondary process sends one Activation Request and exits without opening a second database or creating tray/hotkey resources.
- Use established Windows kernel/IPC primitives behind a narrow Single-Instance Adapter. Object/channel names derive from a stable profile identity and current Windows user scope; activation payloads carry intent, not durable Shelf data.
- Keep display state transient. Version 0.0.4 supports `Hidden` and `Visible`; non-empty dismissal temporarily becomes `Hidden` with references untouched. Do not introduce an unfinished `EdgeDocked` state or Rail window.
- Route normal launch, secondary activation and notification-area activation through one `show_shelf` operation. It selects the monitor containing the cursor, converts preferred logical sizing through that monitor's DPI, clamps complete native bounds to the work area, shows, activates and restores the Shelf's topmost state. Repeated show requests are idempotent but may reposition to the current cursor monitor.
- The global hotkey routes through one toggle operation: `Hidden` shows/activates; `Visible` hides. Tray/normal/secondary actions always show. Losing focus does not hide. Shelf close requests hide rather than terminate; only Exit DropCove or a fatal process boundary terminates the primary resident.
- Create the notification-area icon once per primary Resident Instance. Left/double click show the Shelf. The right-click menu contains Settings and Exit DropCove. Recoverable integration/settings failures use warning notifications after the tray adapter is available.
- Keep one Settings window and one draft. Settings is a transient projection of the coordinator's desired/active state. Reopening focuses the current window. Cancel, close or deactivation during shortcut recording discards the appropriate draft/listening state without applying integrations.
- Persist only `HotKey` and `StartWithWindows` in 0.0.4. Rail and Shake fields are added through clean cutovers with their owning runtime versions; no inactive/no-op controls or speculative compatibility fields are introduced.
- Represent a hotkey as ordered modifier flags plus a numeric Windows virtual-key. The recorder allowlist is letters, digits, F1–F24, navigation/edit keys, OEM punctuation, and modified Tab/Enter/Space. Escape, bare modifiers, mouse keys and media/system keys are not valid primary keys in this version.
- Keep the active hotkey registered while editing. During recording, suppress handling of the old registered combination while still allowing its key sequence into the recorder. Save validates the draft and tests candidate registration before replacing the active registration.
- Apply Settings as one coordinator operation. The candidate hotkey is validated/registered first when changed; the desired current-user startup registration is then applied; `settings.json` is atomically replaced only after integration steps succeed; old hotkey/startup state is retired last. Failure unwinds applied steps and leaves the Settings window open with an actionable error. If rollback cannot fully restore, query adapters for the retained actual state and warn rather than reporting the draft as active.
- Missing `settings.json` yields defaults silently. Malformed or unreadable content stays untouched, defaults load in memory and a warning is queued. Only explicit successful Save replaces it.
- The 0.0.4 current-user startup command launches the same executable/profile identity with hidden-start intent. Hidden start restores persistence/settings, initializes tray/hotkey/startup state and remains `Hidden`. Registration failure does not abort residency; desired Start with Windows remains On and Save can retry.
- Keep Windows integration behind focused adapters: single instance/activation, tray, hotkey, startup registration, monitor/window placement, and foreground/topmost repair. Reuse the already-installed Tauri/Windows dependencies when they expose the required capability; add no plugin or package solely to wrap a few Win32 calls.
- Foreground and display handling is event-driven. Topmost repair must not activate or steal keyboard focus. Monitor placement uses current cursor-monitor work area and Per-Monitor V2 logical/physical conversion; temporary work-area/DPI clamping never rewrites durable preferred Shelf sizing.
- Extend the application version to 0.0.4 only in the release ticket after runtime tickets pass. Do not start 0.0.5, create an Edge Rail, or add Shake-to-open behavior in this effort.

## Testing Decisions

- The permanent high seam is the Resident Application Coordinator interface. Tests drive launch intent, visibility, Settings drafts, Save/Cancel, adapter failures and rollback through the same operations used by the real entry points.
- Use the real Settings Adapter against an explicitly owned temporary profile. Exercise missing, valid, malformed, unreadable and failed atomic replacement behavior through loaded/retained settings, not source text or helper wiring.
- Native integration adapters have deterministic substitutes in coordinator tests. They model registration conflicts, startup failures, tray warnings, partial rollback and activation events; tests assert consumer-visible coordinator state and adapter-observable contracts, not method forwarding.
- Keep Core Shelf State tests at the existing real-SQLite seam. Resident work must not duplicate item lifecycle tests already proved by 0.0.3; integration smoke verifies that the same state survives hiding, secondary activation, hotkey/tray access and hidden start.
- Add permanent behavior tests for profile-scoped primary/secondary outcomes, idempotent activation, hotkey toggle versus always-show entry points, recorder allowlist/cancellation/formatting, Settings atomic Save/Cancel and retained-state reporting after rollback failure.
- Do not permanently test Tauri menu construction, event forwarding, copied labels, registry statement text, process counts without an activation assertion, or mock echoes. Those are wiring/source-shape tests.
- Prove native behavior through actual Windows processes and surfaces: two executable launches sharing an owned profile, notification-area actions/menu/notifications, `RegisterHotKey`, current-user Run registration, `--autostart`, cursor-monitor placement, work-area/DPI conversion and no-activate topmost repair.
- Native smoke must use owned profiles, owned registry values/kernel object names and known process handles. Preserve and restore any pre-existing current-user Run value; never terminate processes by executable name or mutate the ambient DropCove profile.
- Release verification installs 0.0.4 and exercises normal launch, secondary activation, tray show/settings/exit, hotkey toggle/record/Save conflict, startup toggle, hidden sign-in launch, settings fallback and restart persistence from the packaged executable.

## Out of Scope

- Edge Rail, Rail Handle, non-empty dismissal docking, rail monitor preference UI/runtime, rail fullscreen behavior and Open Shelf from Rail.
- Shake-to-open, low-level pointer gesture hook and its settings.
- Final icon/thumbnail resources, full visual-conformance polish, recovery hardening, automatic updater, signing, installer shutdown coordination and complete profile deletion.
- Multiple user-selectable production profiles, accounts, sync, network activation or cross-session/global machine singleton behavior.
- Automatic version advancement past 0.0.4 or implementation before this spec/ticket set receives explicit approval.

## Further Notes

- Confirmed staging decisions: non-empty dismissal is temporarily Hidden; Settings UI contains only Hotkey and Start with Windows; hotkey is the only toggle entry point; missing settings are silent defaults while malformed/unreadable settings warn and remain untouched; failed default startup registration keeps desired On.
- The Resident Application Coordinator seam and native smoke split were explicitly approved before publication.
- Existing 0.0.3 implementation/release history remains closed. This effort gets its own spec, tickets, map and research notes.
- No ADR is added: the Hidden staging transition is deliberately temporary, and the active-settings-only cutover is straightforward to reverse by adding the owning version's fields rather than a hard-to-reverse architectural tradeoff.
- Primary-source Windows/Tauri API findings are recorded in [Windows Resident Integration Research](research/windows-resident-integration.md). Delivery must re-check exact dependency types; the report informs adapters but does not override this product contract.
- The user approved the five-ticket linear breakdown and implementation of 0.0.4. Delivery proceeds ticket by ticket; version metadata changes only in ticket 05.
- User explicitly approved coordinator interface staging: launch/show/hide in ticket 01, warning/Exit in ticket 02 and open Settings in ticket 03. Version-level requirements are unchanged.
