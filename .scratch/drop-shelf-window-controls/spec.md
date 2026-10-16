# DropCove 0.0.14 — Drop Shelf Window Controls

Status: resolved
Version: 0.0.14
Design confirmation: approved by user on 2026-10-16
Testing seam: actual Windows Resident Instance, WebView2 Drop Shelf, and native window behavior, supported by the existing native regression suite; approved by user on 2026-10-16
Delivery prerequisite: the 0.0.13 final human acceptance gate must complete before 0.0.14 implementation starts
Ticket decomposition: three approved slices — [01 Dismiss Drop Shelf control strip](issues/01-dismiss-drop-shelf-control-strip.md) → [02 Native Drop Shelf movement](issues/02-native-drop-shelf-movement.md) → [03 Release DropCove 0.0.14](issues/03-release-0.0.14.md)
Implementation approval: approved by user on 2026-10-16; the 0.0.13 final acceptance gate and Git commit authorization remain separate prerequisites
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.14
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 5–8, 10, and 17–19
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 2, 5, 11, and 15
Placement ownership: [ADR-0002](../../docs/adr/0002-edge-rail-placement-ownership.md)

## Problem Statement

The Drop Shelf is a borderless resident window with no visible control for dismissing it. A user who wants the Drop Shelf out of the way must remember and invoke the configured global hotkey even while the surface is directly in front of them. Closing the Drop Shelf must remain distinct from exiting DropCove: the Resident Instance, Shelf references, notification-area access, and non-empty Edge Rail workflow must continue.

The Drop Shelf is also placed automatically but cannot be moved by the user. Cursor-monitor, Edge Rail, work-area, restart, and Shake-to-open rules choose where it appears, yet the user cannot make a temporary adjustment when it covers another application or an inconvenient part of the current display. The absence of native movement is especially limiting because the undecorated window has no Windows title bar.

Adding generic close or drag behavior at the presentation layer would be unsafe. Directly hiding or destroying the window could bypass the Resident Application Coordinator, leave the display state or Edge Rail inconsistent, or skip Manage Items cleanup. Treating cards or the whole surface as a window-drag region would conflict with Copy drag-out, scrolling, action controls, confirmation dialogs, and the existing eight-direction resize contract.

## Solution

Add a compact, titleless control strip to the top of the Drop Shelf. The strip is 28 logical pixels high, contains a subtle movement affordance, and exposes a 24-by-24 logical-pixel top-right `X` action named **Hide Drop Shelf**. The `X` requests the existing **Dismiss Drop Shelf** behavior rather than directly hiding, destroying, or terminating anything. An empty Shelf transitions from `Visible` to `Hidden`; a non-empty Shelf transitions from `Visible` to `EdgeDocked`; Manage Items closes; Shelf references remain intact; and the Resident Instance keeps running.

Use the remainder of the strip as the only window-drag region. A primary-pointer action starts native Windows movement after Manage Items has closed successfully. Cards, actions, collection content, scrollbars, dialogs, status controls, and resize handles remain outside the drag region. Movement does not start during resize, modal confirmation, or a transactional Shelf interaction.

Movement is session-only presentation state. It does not add a setting, persisted coordinate, display state, or placement owner. While the Drop Shelf remains visible, the user-selected position is retained subject to existing work-area and display/DPI reflow rules. After dismissal, activation, Open Shelf, restart, or another established presentation transition, the existing cursor-monitor or Edge Rail monitor placement rules remain authoritative.

The primary acceptance seam is the actual Windows Resident Instance with the real WebView2 Drop Shelf and native window. Existing coordinator/native regression tests, frontend production build, cargo-xwin compilation, and the directly executed Windows test binary support that seam; no new frontend component-test framework is introduced.

## User Stories

1. As a Drop Shelf user, I want a visible `X` action, so that I can dismiss the surface without remembering a global hotkey.
2. As a user with an empty Shelf, I want the `X` action to hide every Shelf surface, so that an empty DropCove surface is not left on screen.
3. As a user with retained Shelf Items, I want the `X` action to dismiss the Drop Shelf into the Edge Rail, so that my content remains discoverable.
4. As a DropCove user, I want dismissing the Drop Shelf to keep the Resident Instance running, so that hotkey, notification-area, startup, and activation workflows remain available.
5. As a user with Temporary or Pinned Items, I want dismissal to preserve every Shelf reference, so that hiding the presentation cannot discard my workspace.
6. As a user with Manage Items open, I want Drop Shelf dismissal to close that popup through its established lifecycle, so that no orphaned management surface remains.
7. As a user, I want the visible `X`, the native close request, `Esc`, and the global hotkey to converge on the same display-state rules, so that dismissal is predictable.
8. As a user, I want **Exit DropCove** to remain a separate notification-area command, so that the `X` cannot accidentally terminate the resident application.
9. As a pointer user, I want the `X` to use the established compact action size and hover treatment, so that it is consistent with the rest of DropCove.
10. As a keyboard user, I want the `X` reachable in the Drop Shelf Tab order and activatable with Enter or Space, so that dismissal does not require a pointer.
11. As a screen-reader user, I want the action named **Hide Drop Shelf**, so that the icon's effect is explicit.
12. As a pointer user, I want a native tooltip for the `X`, so that its meaning is discoverable before activation.
13. As a Dark, Light, or Windows Contrast user, I want the close action and movement affordance to use theme-aware colors, so that they remain legible in my active presentation.
14. As a user who values the compact design, I want window controls without restoring the removed `DropCove` title or subtitle, so that the Shelf remains concise.
15. As a pointer user, I want a dedicated movement affordance, so that I can discover how to reposition the undecorated Drop Shelf.
16. As a user with an empty Drop Shelf, I want to drag it from the control strip, so that I can move the drop target out of the way.
17. As a user with a populated Drop Shelf, I want the movement strip to remain reachable, so that card density does not remove window movement.
18. As a multi-monitor user, I want native movement between connected displays, so that I can place the visible Shelf on the display I am working on.
19. As a mixed-DPI user, I want movement and subsequent reflow to honor the destination display's scale and work area, so that the Shelf remains usable.
20. As a user, I want the moved Drop Shelf to remain inside the applicable monitor work area, so that its controls cannot become inaccessible.
21. As a user dragging a Shelf Item or Shelf Batch, I want card drag-out to remain Copy-only and unchanged, so that moving the window cannot replace the file-transfer gesture.
22. As a user clicking Pin, Manage Items, Remove, Clear Temporary Items, or status controls, I want those controls to remain normal actions, so that they never start window movement.
23. As a user scrolling many Shelf Batches, I want the collection and scrollbar to remain dedicated to scrolling, so that scrolling cannot move the window.
24. As a user resizing the Drop Shelf, I want every edge and corner resize handle to retain its existing cursor, hover, and sizing behavior, so that movement cannot steal resize input.
25. As a user with Manage Items open, I want it closed before movement begins, so that the popup is not left detached at its old screen position.
26. As a user, I want movement to stay blocked if Manage Items cannot close cleanly, so that DropCove does not enter a partially moved transient state.
27. As a user viewing a confirmation dialog, I want the modal interaction to retain pointer ownership, so that I cannot accidentally move the Shelf while deciding.
28. As a user during a serialized Shelf mutation, I want window movement blocked until the interaction settles, so that presentation input does not compete with an in-flight operation.
29. As a user during native resize, I want movement blocked, so that one pointer gesture cannot start two window operations.
30. As a user who temporarily moved the Shelf, I want it to stay where I placed it while that visible session continues, so that the adjustment is useful.
31. As a user reopening the Shelf with the hotkey, tray, or normal activation, I want cursor-monitor placement to remain authoritative, so that established access behavior does not change.
32. As a user opening the Shelf from the Edge Rail, I want the rail's current monitor to remain authoritative, so that a temporary drag does not override Edge Rail placement ownership.
33. As a user restarting DropCove, I do not want an old manually dragged coordinate restored, so that disconnected displays and stale desktop layouts cannot strand the Shelf.
34. As a user, I do not want a new placement setting solely for temporary movement, so that Settings remains focused on durable preferences.
35. As a user with Automatic Shelf Growth, I want the new strip included in the presentation chrome budget, so that ordinary batch counts do not gain premature scrolling.
36. As a user with a Manual Height Override, I want the existing outer-window sizing intent and work-area clamping preserved, so that movement controls do not create a second sizing model.
37. As a user, I want the empty downward icon, `Drop here` prompt, card density, footer, status presentation, and rounded window treatment preserved around the new controls.
38. As a user, I want dismissal or movement failures reported through existing status or warning behavior while the prior coherent state remains visible, so that failures do not silently lose access.
39. As a release owner, I want the feature exercised through the actual Windows Resident Instance and WebView2 surface, so that native movement and display-state integration are proven together.
40. As a maintainer, I want the close action to reuse the established Resident coordinator and the movement action to use the platform window API, so that no second lifecycle or geometry system is introduced.
41. As a security-conscious maintainer, I want only native movement permission scoped to the main Drop Shelf window and dismissal owned by the Resident coordinator, so that presentation code cannot destroy or raw-hide windows.
42. As a maintainer, I want no new database field, settings field, migration, dependency, polling loop, or compatibility layer, so that the feature remains the smallest complete vertical slice.
43. As a user, I want source files and folders to remain untouched by every close and movement path, so that presentation changes cannot imply deletion or Move behavior.
44. As a release owner, I want one exact Windows x64 installer candidate verified after implementation acceptance, so that the delivered 0.0.14 bytes match the behavior that was approved.

## Implementation Decisions

- Treat 0.0.14 as one Drop Shelf interaction slice containing both visible dismissal and native movement. Do not fold it into the accepted 0.0.13 spacing scope or split the shared control-strip work across multiple releases.
- Do not start implementation until the 0.0.13 final human acceptance gate is complete. A `ready-for-agent` specification means the requirements are complete; it does not override version sequencing or grant implementation approval.
- Add one 28-logical-pixel titleless control strip at the top of the Drop Shelf. Do not restore a product title, subtitle, native decoration, minimize control, maximize control, or standard title bar.
- Keep the Drop Shelf minimum preferred outer size at `191 × 180` logical pixels. Account for the new strip in Automatic Shelf Growth and collection chrome calculations so normal row counts do not gain premature overflow. Preserve Manual Height Override as existing outer-window intent and add no persisted sizing field.
- Use the established 24-by-24 logical-pixel icon-action contract for the top-right close action and the existing close glyph. The icon inherits `currentColor`, uses semantic theme tokens and Windows system colors, exposes a native tooltip, has a visible hover and `focus-visible` state, and is named **Hide Drop Shelf**.
- Put the close action before Shelf Batch and footer actions in the natural Drop Shelf Tab order. Standard Enter and Space activation apply. The decorative movement affordance is not a button and is hidden from the accessibility tree.
- Define the close action as **Dismiss Drop Shelf**, not application close or process exit. An empty Shelf produces `Hidden`; a non-empty Shelf produces `EdgeDocked`; Manage Items closes; visual resources follow existing surface-lifetime cleanup; the Resident Instance and Shelf references remain.
- Add one frontend-invokable backend command named `dismiss_drop_shelf`. It receives the calling window context, rejects callers other than the main Drop Shelf window, and then delegates to the existing Resident coordinator dismissal operation. The `X` invokes this command; native `CloseRequested`, `Esc`, and hotkey paths remain separate callers of the same coordinator behavior. Frontend code must not call window close or hide APIs or reproduce display-state logic.
- If `dismiss_drop_shelf` fails, leave the Drop Shelf visible in the last coherent display state and publish the error through existing Shelf status or resident warning behavior. Do not fall back to direct hiding.
- Grant only the main Drop Shelf window `core:window:allow-start-dragging`. Dismissal uses the resident-owned backend command and requires no additional window capability.
- Use the platform's native window-drag API for movement. Do not implement movement with document-level pointer tracking, repeated coordinate updates, polling, or custom Windows message loops.
- Start movement only from the control strip's non-action drag region and only for the primary pointer. The close action stops drag-region activation. Shelf Batch drag areas, every action control, collection content, scrollbars, dialogs, status UI, footer, and resize handles remain excluded.
- Close Manage Items through its established lifecycle before invoking native movement. Start movement only after close succeeds. Preserve existing focus restoration when valid, but do not let popup focus handling replace or cancel the initiating native drag gesture.
- Block movement while Shelf mutation state is busy, while native resize is active, while a batch-drag or Clear Temporary Items confirmation is open, or while another modal/transient interaction owns the surface. Do not silently cancel a confirmation or mutation to move the window.
- Preserve existing item and whole-batch Copy drag-out semantics, movement threshold, availability preparation, Temporary/Pinned completion behavior, cancellation behavior, and source-file safety. The control strip never becomes a storage drag source or drop target.
- Keep movement session-only and disposable. Do not persist coordinates, add a placement setting, add another display state, or change Edge Rail placement ownership recorded in ADR-0002.
- While the Drop Shelf remains visible, retain its manually moved position through ordinary Shelf refreshes and content changes unless work-area, display, or DPI constraints require reflow. Dismissal followed by hotkey, tray, normal activation, restart, Open Shelf, or Shake-to-open continues to use the canonical placement rule for that entry point.
- Use native Windows movement across monitors, then enforce the existing work-area and Per-Monitor V2 DPI invariants through event-driven reflow. Do not add custom snapping, magnetic edges, or continuous position polling.
- Preserve topmost repair without activation, rounded transparent-frame ownership, eight-direction native resizing, collection virtualization, compact scrollbar behavior, empty-state prompt, status dismissal, Clear Temporary Items, and existing motion policy.
- Introduce no dependency, schema migration, settings field, database field, telemetry, general title-bar component, generalized window manager, or compatibility path.
- Update the canonical product behavior documentation during delivery to describe the control strip, Dismiss Drop Shelf action, movement boundaries, and non-persisted placement. No ADR is required because the feature follows existing presentation, coordinator, and placement ownership.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Use one primary behavioral seam: the actual Windows Resident Instance with the real WebView2 Drop Shelf and native window. This is the highest existing seam that proves the icon control, coordinator transition, Edge Rail result, native movement, pointer routing, popup lifecycle, resize integration, theme, focus, and monitor geometry together.
- Use the existing Resident coordinator/native regression suite as supporting coverage for `Visible` to `Hidden`, `Visible` to `EdgeDocked`, retained residency, popup cleanup, placement failure safety, and existing activation placement. Add permanent cases only where a plausible user-visible regression is not already covered; do not test that one function merely forwards to another.
- Run the complete frontend production build for Vue single-file-component typechecking and bundling. Do not add a frontend component-test framework for this feature.
- Compile with cargo-xwin for `x86_64-pc-windows-msvc`, then run the complete produced Windows native test executable directly. Wine runner failure or absence is not a passing runtime result.
- Verify the capability and caller boundary in the running application: the main Drop Shelf can start native movement, unrelated windows cannot, `dismiss_drop_shelf` accepts the main caller, and it rejects every non-main caller without changing presentation state.
- Exercise an empty Drop Shelf through the `X`: the surface disappears, no Edge Rail appears, the process remains resident, and a later activation restores the Drop Shelf through the canonical entry-point placement.
- Exercise a non-empty Drop Shelf through the `X`: Manage Items closes, the Drop Shelf disappears, the Rail Handle appears at canonical placement, every Shelf reference remains, and Open Shelf restores the content.
- Compare the `X`, native main-window close request, `Esc`, and global hotkey paths for the same empty and non-empty state outcomes. Test observable states and surfaces, not source routing.
- Exercise close-action failure handling with an existing realistic failure seam where practical. Prove the main window is not destroyed or raw-hidden and the last coherent state remains accessible.
- Exercise keyboard traversal to the top-right close action, visible focus, Enter, Space, tooltip, and the exact accessible name **Hide Drop Shelf**. Confirm the decorative movement affordance is absent from the accessibility tree.
- Exercise pointer movement from the drag region with empty, populated, scrollable, automatically grown, manually resized, and status-bearing Drop Shelves.
- Exercise movement with Manage Items open. Confirm the popup closes first through its normal lifecycle and native movement begins only after successful closure.
- Exercise blocked movement while resizing, while each in-Shelf confirmation is open, and while the Shelf reports a transactional busy state. Confirm the owned interaction remains intact and no native movement begins.
- Exercise cards, Pin/Bulk Pinning, Manage Items, Remove, Clear Temporary Items, status dismissal, collection scrolling, scrollbar dragging, and all eight resize directions. None may start native window movement.
- Exercise actual single-item and whole-batch Windows Copy drag-out after the control strip is present. Confirm accepted Temporary consumption, Pinned retention, cancellation/rejection retention, and unchanged source bytes.
- Move the Drop Shelf within one monitor and across every available monitor. Observe destination DPI, outer bounds, work-area containment, topmost behavior, and absence of focus theft beyond normal native movement.
- After manual movement, refresh Shelf content and resize without an unnecessary jump. Then dismiss and reopen through hotkey, tray/activation, and Edge Rail Open Shelf to prove that canonical placement supersedes the prior dragged coordinate.
- Inspect empty and populated control-strip geometry at minimum width and representative responsive widths. Verify the 28-pixel strip, 24-pixel close target, unchanged rounded transparent frame, no restored title text, no card overlap, and no premature scrolling caused solely by the new chrome.
- Inspect Dark, Light, and actual Windows Contrast Theme where available. Record unavailable theme, fractional-DPI, mixed-DPI, or monitor configurations as unobserved rather than inferring them from emulation or 100% DPI.
- Preserve the existing reduced-motion behavior; no movement animation is added. Confirm disabling client-area animations does not affect close or native drag behavior.
- Keep version metadata at 0.0.13 until implementation and actual-surface acceptance pass. Then synchronize all canonical version sources to 0.0.14, build one exact current-user Windows x64 NSIS candidate, and record installer and installed-executable sizes and SHA-256 hashes.
- Exercise the exact installed candidate with embedded frontend assets and the development server stopped. Cover dismissal, movement, resident activation, Edge Rail return, Cooperative Shutdown, same-candidate reinstall/update, uninstall, and Profile Data preservation.
- Add no permanent screenshot snapshots, CSS-class assertions, source-text assertions, mock-echo tests, or tests that only prove a button calls a command.

## Out of Scope

- Persisting the manually dragged Drop Shelf position across dismissal, activation, restart, update, or monitor reconnection.
- Adding a window-position setting, reset-position action, monitor picker for the Drop Shelf, or migration for prior placement data.
- Changing cursor-monitor placement, Open Shelf placement, Shake-to-open placement, remembered Edge Rail dismissal monitor ownership, or configured Edge Rail placement.
- Custom snapping, magnetic screen edges, desktop docking, top/bottom Edge Rail support, or a second rail-placement mode.
- Moving Manage Items with the Drop Shelf, preserving it at its old coordinates, or redesigning its placement and focus-restoration lifecycle.
- Drag-to-move behavior for Manage Items, Settings, the Edge Rail, Rail Handle, confirmation dialogs, or any surface other than the Drop Shelf.
- Using the `X` to terminate DropCove, adding Exit to the Drop Shelf, or changing Cooperative Shutdown and installer lifecycle behavior.
- Restoring a title, subtitle, standard Windows title bar, minimize button, maximize button, taskbar entry, shadow, or native window decorations.
- Redesigning Shelf Batches, card actions, Clear Temporary Items, status messages, empty-state content, scrollbars, virtualization, resize handles, or visual tokens beyond the approved control strip.
- Changing Shelf Item persistence, Temporary/Pinned lifecycle, duplicate behavior, availability handling, Copy-only drag semantics, or source-file safety.
- Adding touch-specific movement gestures, keyboard window-position commands, telemetry, analytics, a component framework, a frontend test framework, or a new icon dependency.
- Changing 0.0.13 scope, status, evidence, artifacts, or acceptance requirements.
- TypeScript 7, automatic updates, signing, macOS, Linux, x86, or ARM64 support.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- The user approved the close semantics, dedicated control strip, Manage Items closure-before-move rule, session-only placement, 0.0.14 version grouping, and actual Windows Resident/WebView2 testing seam on 2026-10-16.
- **Dismiss Drop Shelf** is the canonical domain term. It must not be described as exiting DropCove or always hiding every Shelf surface.
- ADR-0002 remains authoritative: Settings owns configured Edge Rail placement, Shelf state owns the latest non-empty dismissal monitor, and Open Shelf uses the currently docked rail monitor. Temporary Drop Shelf movement creates no competing placement owner.
- No new ADR is warranted. The feature is a reversible presentation addition within established Resident, Presentation, and Window/Monitor adapter ownership.
- Ticket decomposition is published and approved. The implementation frontier is Ticket 01 after the 0.0.13 final human acceptance gate; Git commit authorization remains separate.

## Delivery Evidence — 2026-10-16

- **Ticket 01 (Dismiss Drop Shelf control strip):** Resolved. Added titleless 28px control strip with top-right 24×24 close action and backend `dismiss_drop_shelf` command validating caller window. Real WebView2 smoke exercised empty/non-empty dismissal, Manage Items closure, and reference retention.
- **Ticket 02 (Native Drop Shelf movement):** Resolved. Scoped `core:window:allow-start-dragging` exclusively to `main`. Added `prepare_shelf_move` closing Manage Items and validating drag/resize state. Exercised native cross-monitor movement, modal preemption, OLE Copy drag-out, and session-only placement.
- **Ticket 03 (Release DropCove 0.0.14):** Exact candidate produced and verified.
  - NSIS Installer: `build/DropCove-Setup-0.0.14.exe` (3,244,674 bytes, SHA-256 `66d7df7efbc4c2abc42abc017ccb12ef7c511a5ad19798732d9e4d4fd7b55d5a`).
  - Installed binary: `dropcove.exe` (13,457,408 bytes, SHA-256 `181962e9245c449641aa4bb0bf0349f8131b90288d19080f8a88c86fe5ea37b4`).
  - Unbundled target release binary: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe` (13,457,408 bytes, SHA-256 `d88df95cb88c288c6ee96b22524739aefc4783bf0cd5f4ad95a3881c14c395aa`).
  - 106/106 native tests passed.
  - Embedded production assets verified with Vite dev server stopped.
  - Silent install, secondary activation, Cooperative Shutdown (exit 0), uninstall removing binary, and profile `shelf.db` retention verified.
  - Same-candidate reinstall/update and Start Menu shortcut creation verified.
  - Ambient registry baseline restored.
  - Unobserved limits: physical high-DPI scaling, physical hardware hotkey chord detection, and physical Windows Contrast Theme monitor presentation.
- User confirmed final 0.0.14 human acceptance on 2026-10-16 and authorized the 0.0.15 ticket implementation. This records user-reported acceptance, not additional agent-observed verification. Git commit authorization remains separate.
