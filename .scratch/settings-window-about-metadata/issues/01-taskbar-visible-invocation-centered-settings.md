# 01: Taskbar-visible, invocation-centered Settings

**What to build:** Make Settings behave as a normal Windows configuration surface: one decorated top-level singleton with a taskbar button, centered on the invoking monitor when first created and reused without repositioning afterward. Preserve the complete Settings draft, Save/Cancel, shortcut recording, Shelf presentation, and Resident Instance shutdown contracts.

**Parent specification:** [DropCove 0.0.17 — Settings Window and About Metadata Polish](../spec.md)

**Testing seam:** Actual Windows Resident Instance with real WebView2 Settings, Drop Shelf Utility Menu, notification-area menu, taskbar, native window frame, and available monitors; deterministic shared placement checks support diagnosis.

**Demo path:** After the 0.0.16 human gate closes, open new Settings windows from the Drop Shelf and notification area on available monitors, navigate through the Windows taskbar, move/minimize/reopen the singleton, exercise Save/Cancel and shortcut recording, then exit DropCove with Settings open.

**Blocked by:** [DropCove 0.0.16 Ticket 04 — Release DropCove 0.0.16](../../drag-placement-shelf-fit/issues/04-release-dropcove-0.0.16.md), specifically its final human acceptance gate.

**Status:** resolved

- [x] The blocking 0.0.16 final human acceptance gate is closed before source implementation begins.
- [x] One narrow invocation-context contract distinguishes Drop Shelf requests from notification-area requests; no user-controlled monitor identifier or configurable placement policy is added.
- [x] The already-proven About Surface monitor selection, complete-outer-frame measurement, centered work-area calculation, and physical positioning behavior is reused through one shared auxiliary-window placement seam rather than copied or exposed through About-specific internals.
- [x] A new Settings window opened from the Drop Shelf is centered and clamped within the work area of the monitor containing the Drop Shelf, with Windows primary-monitor fallback when that monitor cannot be resolved.
- [x] A new Settings window opened from the notification area is centered and clamped within the work area of the monitor containing the pointer, with Windows primary-monitor fallback when that monitor cannot be resolved.
- [x] Settings is built hidden, positioned from its measured decorated outer frame in physical coordinates, and only then shown, so the user does not see the default upper-left placement.
- [x] Settings retains its established `420 × 520` logical inner size, fixed-size decorations, title, normal z-order, label-based frontend composition, and non-modal behavior.
- [x] Settings is a top-level native window without Drop Shelf ownership, remains non-topmost, and has a normal Windows taskbar button.
- [x] Requests from either entry point reuse one Settings window and one draft. An existing minimized or hidden Settings window is restored, shown, and focused without changing its current position.
- [x] Manually moving an open Settings window is respected for the rest of that window lifetime; no position is persisted across close or restart.
- [x] Opening Settings from the notification area does not show, move, activate, or otherwise change the Drop Shelf or Edge Rail. Opening it from the Drop Shelf Utility Menu closes the transient menu before focus transfers.
- [x] Save still applies the established transactional settings operation and closes on success. Cancel or native close still discards unsaved changes. Settings deactivation still cancels shortcut recording without discarding unrelated draft values.
- [x] Global hotkey, Start with Windows, Rail placement, fullscreen, Shake-to-open, warning, rollback, and settings persistence behavior remains unchanged.
- [x] Explicit Exit DropCove and Cooperative Shutdown close the top-level Settings window and complete existing hotkey/Shake teardown without a stranded Resident Instance.
- [x] Permanent deterministic tests cover centered/clamped complete-frame geometry, negative monitor coordinates, Settings frame dimensions, and constrained work areas through the shared placement seam without duplicating lower-level forwarding tests.
- [x] Actual Windows smoke proves both invocation contexts, taskbar presence, singleton reuse, minimize/restore, no-reposition reuse, independent Shelf visibility, Save/Cancel, recorder focus loss, and Resident shutdown.
- [x] Relevant native regressions, frontend production build, and cargo-xwin Windows build pass; unavailable physical mixed-DPI or monitor configurations are recorded as unobserved rather than inferred.
- [x] No database field, settings field, migration, dependency, generalized window manager, compatibility layer, version bump, or Git commit is introduced by this ticket.

## Answer — 2026-10-20

- Baseline actual Windows Settings opened at outer `[0, 0, 436, 559]` with the hidden Drop Shelf as owner. The implemented top-level window has no owner and centers at `[742, 236, 1178, 795]` in primary work area `[0, 0, 1920, 1032]`, and `[2662, 236, 3098, 795]` on the second monitor.
- Shared complete-frame placement regressions passed 2/2. Cargo-xwin compiled the focused test executable and embedded release application; only external SDK missing-PDB linker warnings remain.
- Actual WebView2/native smoke proved taskbar registration (`DropCove - 1 running window` while Shelf is hidden), native tray Settings routing, no-owner/non-tool/non-topmost flags, singleton/minimize restore at user-moved origin `[230,170]`, complete-frame centering, Save persistence, Cancel byte retention, recorder cancellation on About focus, and Cooperative Shutdown exit 0 with Settings/About open.
- The Settings surface was visually inspected at `420 × 520`. Pointer movement is restricted to the primary monitor by the host, so tray-to-secondary and physical mixed-DPI behavior remain unobserved; Drop Shelf invocation was exercised on both 96-DPI displays. Initial pointer-menu automation hit a Desktop menu, which was dismissed without selecting an action; successful tray verification used the owned native tray window/menu command.
- Full-suite and exact installed-candidate release proof belong to Ticket 03. No version bump or commit was created by this slice.
