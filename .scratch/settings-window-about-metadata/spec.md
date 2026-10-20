# DropCove 0.0.17 — Settings Window and About Metadata Polish

Status: resolved
Version: 0.0.17
Design confirmation: approved by user on 2026-10-20 through `grill-with-docs`
Testing seam: actual Windows Resident Instance with the real WebView2 Settings and About Surface, Drop Shelf Utility Menu, notification-area menu, and Windows taskbar; supported by existing native window-placement and metadata boundaries; explicitly approved by the user on 2026-10-20 during `to-spec`
Delivery prerequisite: completed — the user reported final human acceptance and commit of 0.0.16 on 2026-10-20
Ticket decomposition: all three approved tickets resolved; the user confirmed final 0.0.17 acceptance
Implementation approval: implementation, final release acceptance, and Git commit explicitly authorized by the user
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.17
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), especially notification-area behavior, About Surface, global-hotkey Settings behavior, Drop Shelf Utility Menu, and Settings
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), especially Resident Process, Presentation, Settings, and window/monitor/DPI ownership
Placement ownership: [ADR-0002](../../docs/adr/0002-edge-rail-placement-ownership.md) remains unchanged

## Problem Statement

Settings currently opens at the upper-left of the desktop instead of appearing in the work area where the user invoked it. This is especially disruptive on multi-monitor systems: a Settings request from the Drop Shelf or notification area can make the user search another display or move a window that should have appeared in the active context.

Settings also does not present a Windows taskbar button even though its builder requests taskbar visibility. The window is currently owned by the hidden Drop Shelf window. On Windows, that ownership keeps Settings out of the normal top-level taskbar navigation model. Users can see About Surface in the taskbar but cannot use the same navigation pattern for Settings.

The package identity is also inaccurate. Cargo metadata still attributes the application to `DropCove Contributors` and describes it as `DropCove Resident Workspace`, which does not clearly explain the product. About Surface projects author and version from authoritative runtime/build metadata, but it does not project the package description. Users therefore cannot see the improved product description in the application, and hard-coding it in the frontend would create a second source of truth.

DropCove needs one small presentation and metadata release: Settings must become a normal singleton top-level window, appear on the Windows taskbar, and center on the invoking monitor only when first created. Cargo must identify `HungNth` as author and provide one concise product description. About Surface must display that exact Cargo description immediately below the synchronized runtime version without changing established Settings state, Shelf behavior, or Resident Instance lifetime.

## Solution

Create Settings as a decorated, fixed-size, singleton top-level window rather than an owned Drop Shelf window. Keep it taskbar-visible and independent of whether the Drop Shelf is visible, hidden, or EdgeDocked. The Resident Instance remains responsible for Settings creation, singleton reuse, close handling, draft cleanup, and shutdown.

Give Settings the same invocation-aware initial placement contract already established for About Surface. A new Settings window opened from the Drop Shelf is centered within the work area of the monitor containing the Drop Shelf. A new Settings window opened from the notification area is centered within the work area of the monitor containing the pointer. If the preferred monitor cannot be resolved, use the Windows primary monitor. Measure and position the complete outer native frame in physical coordinates so decorations remain within the work area. Repeated requests show, restore, and focus the existing Settings window without moving it.

Change the Cargo package author to `HungNth` and the Cargo package description to **A resident Windows utility providing a temporary drag-and-drop workspace for file and folder references.** Extend the existing read-only About metadata projection with that authoritative package description. Display it directly below the runtime version in About Surface, allowing restrained centered wrapping within the existing fixed-size layout. Keep product name and version runtime-owned, and keep author, description, and Source Cargo-owned.

Use the running Windows Resident Instance as the primary acceptance seam. Exercise the real WebView2 Settings and About Surface, native window frames, notification-area and Drop Shelf entry points, Windows taskbar, multi-monitor placement, singleton reuse, Settings Save/Cancel behavior, metadata projection, installed assets, and exact NSIS candidate together. Deterministic native placement and metadata checks support this seam but do not replace actual Windows observation.

## User Stories

1. As a Drop Shelf user, I want a newly created Settings window centered on the Drop Shelf monitor, so that configuration appears where I am already working.
2. As a notification-area user, I want a newly created Settings window centered on the pointer monitor, so that tray navigation follows my active display.
3. As a multi-monitor user, I want invocation source to determine the target monitor, so that Settings does not appear on an unrelated display.
4. As a user whose preferred invocation monitor cannot be resolved, I want Settings to fall back to the Windows primary monitor, so that the window remains reachable.
5. As a user with the Windows taskbar occupying part of a display, I want Settings centered within the monitor work area rather than the complete monitor bounds, so that the taskbar does not cover controls.
6. As a user on a monitor with non-zero or negative desktop coordinates, I want Settings placed correctly within that monitor, so that secondary-display layouts work predictably.
7. As a user on a scaled display, I want DropCove to measure and position the complete native frame in physical coordinates, so that DPI scaling does not move decorations outside the work area.
8. As a user on a work area smaller than the normal Settings frame, I want the frame clamped to the available work area, so that its top-left controls remain reachable.
9. As a user opening Settings for the first time in a process, I want deliberate centering rather than the Windows default upper-left position, so that the surface feels intentional.
10. As a user reopening an existing Settings window, I want DropCove to show and focus that window, so that I return to the current Settings session.
11. As a user who manually moved Settings, I want repeated requests to preserve its current position, so that DropCove does not fight my placement.
12. As a user who minimized Settings, I want another Settings request to restore and focus it, so that the request reliably brings configuration back.
13. As a user opening Settings repeatedly, I want only one Settings window, so that multiple drafts cannot compete.
14. As a user opening Settings from both the Drop Shelf and notification area, I want both entry points to reuse the same singleton, so that configuration state stays coherent.
15. As a Windows user, I want Settings to have a taskbar button, so that I can navigate back to it using the standard Windows shell.
16. As a Windows user with several application windows open, I want Settings represented independently in the taskbar, so that it is not hidden behind the resident Drop Shelf lifecycle.
17. As a user with the Drop Shelf hidden, I want notification-area Settings to remain visible and taskbar-accessible, so that configuration does not depend on Shelf presentation.
18. As a user with the Shelf EdgeDocked, I want opening Settings to leave the Edge Rail state unchanged, so that configuration does not alter Shelf availability.
19. As a user opening Settings from the notification area, I do not want the Drop Shelf shown or moved, so that utility access does not change presentation state.
20. As a user opening Settings from the Drop Shelf Utility Menu, I want Settings to receive focus after the menu closes, so that the handoff remains clear.
21. As a user editing Settings, I want existing Save behavior preserved, so that successful changes still apply atomically and close the window.
22. As a user canceling Settings, I want unsaved changes discarded, so that top-level taskbar behavior does not alter draft semantics.
23. As a user closing Settings with the native close action, I want unsaved changes discarded, so that standard Windows close behavior remains equivalent to Cancel.
24. As a user recording a global shortcut, I want Settings deactivation to cancel recording as before, so that taskbar navigation cannot leave the recorder active unexpectedly.
25. As a user whose shortcut registration fails, I want Settings to remain open with the existing active shortcut preserved, so that presentation changes do not weaken rollback safety.
26. As a user changing Start with Windows, Edge Rail, fullscreen, or Shake-to-open preferences, I want all established controls and Save/Cancel behavior unchanged, so that this release does not redesign Settings.
27. As a user exiting DropCove while Settings is open, I want Resident Instance shutdown to close Settings cleanly, so that removing native window ownership does not strand the process.
28. As a user undergoing Cooperative Shutdown, I want Settings cleanup to complete normally, so that update and uninstall remain reliable.
29. As a user who closes the Drop Shelf while Settings is open, I want Settings to remain usable, so that a top-level utility window is independent of Shelf visibility.
30. As a user, I want the Settings title, fixed size, decorations, and existing visual design preserved, so that only placement and taskbar navigation change.
31. As a user, I want Settings to remain non-topmost, so that it behaves like a normal Windows configuration window rather than covering unrelated applications.
32. As a user, I want Cargo metadata to identify the author as `HungNth`, so that application attribution is accurate.
33. As a user, I want Cargo metadata to describe DropCove as a resident Windows drag-and-drop workspace for file and folder references, so that the product purpose is clear.
34. As a user viewing About Surface, I want to see the package description immediately below the version, so that I can understand what DropCove does.
35. As a user viewing About Surface, I want the exact Cargo description rather than similar frontend copy, so that product identity does not drift.
36. As a user viewing About Surface, I want the runtime version to remain above the description, so that release identity and product purpose have a predictable reading order.
37. As a user viewing About Surface, I want the description to wrap cleanly within the existing fixed-size surface, so that the complete text remains readable.
38. As a user viewing About Surface, I want author `HungNth` shown through the existing metadata row, so that About matches Cargo metadata.
39. As a user viewing About Surface, I want the Source link and repository behavior unchanged, so that the metadata addition does not disturb existing navigation.
40. As a user viewing About Surface, I want the synchronized runtime version unchanged by frontend literals, so that development and release builds report their actual version.
41. As a user viewing About Surface in Dark presentation, I want the description legible with existing tokens, so that the added text matches the established design.
42. As a user viewing About Surface in Light presentation, I want the description legible without changing the layout hierarchy, so that the surface remains consistent.
43. As a Windows Contrast user, I want the description and Settings taskbar behavior to remain usable with system presentation, so that the release does not depend on custom colors.
44. As a screen-reader user, I want the About reading order to announce product name, version, description, then metadata, so that the added information has clear context.
45. As a keyboard user, I want Settings and About controls to retain visible focus and standard operation, so that presentation polish does not regress accessibility.
46. As a reduced-motion user, I want no new animation required for Settings placement or About metadata, so that the behavior remains direct and predictable.
47. As a maintainer, I want one invocation-context contract shared by Settings and About Surface, so that Drop Shelf and notification-area monitor selection cannot diverge.
48. As a maintainer, I want one complete-outer-frame centering calculation for auxiliary top-level windows, so that DPI and work-area fixes apply consistently.
49. As a maintainer, I want Settings to depend on a narrow shared placement seam rather than About-specific internals, so that domain modules remain separated.
50. As a maintainer, I want the existing Resident-owned Settings singleton and draft coordinator preserved, so that a taskbar fix does not create a second state owner.
51. As a maintainer, I want About description added to the existing read-only metadata projection, so that no second command or manifest reader is introduced.
52. As a maintainer, I want Cargo package metadata to remain the sole owner of author, description, and repository, so that packaging and About share authoritative values.
53. As a maintainer, I want product name and runtime version to remain owned by the running Tauri package, so that release synchronization continues to work automatically.
54. As a maintainer, I want no new database field, settings field, migration, dependency, capability, or generalized window manager, so that 0.0.17 stays small.
55. As a maintainer, I want no persisted Settings position, so that first-creation centering does not become a new settings contract.
56. As a release owner, I want actual Windows taskbar presence observed rather than inferred from builder flags, so that shipped behavior matches the user request.
57. As a release owner, I want both Settings invocation paths exercised on available monitors, so that source-aware centering is proven end to end.
58. As a release owner, I want the exact installed candidate to report author `HungNth` and the exact Cargo description, so that package and About metadata are synchronized in shipped bytes.
59. As a release owner, I want version metadata kept at `0.0.16` until implementation acceptance, so that a planned milestone is not presented as delivered.
60. As a release owner, I want one exact Windows x64 NSIS candidate produced after implementation acceptance, so that `0.0.17` identifies verified behavior.
61. As a user, I want source files, folders, Shelf state, Profile Data, and saved Settings untouched by window placement and About metadata changes, so that this polish release cannot cause data loss.
62. As a user, I want taskbar behavior for Drop Shelf, Edge Rail, Manage Items, and confirmation surfaces unchanged, so that only Settings receives the requested navigation change.
63. As a Windows user, I want this behavior optimized for Windows x64 rather than diluted by unsupported platform abstractions, so that native taskbar and monitor semantics are reliable.

## Implementation Decisions

- Deliver the approved work as one `0.0.17` release with three slices: Settings presentation, package/About metadata, and release integration.
- The `0.0.16` prerequisite is closed by user-reported final acceptance. The user approved the three-ticket breakdown and requested implementation of all `0.0.17` tickets. Release acceptance and Git commit authorization remain separate gates.
- Preserve the Resident Instance as the lifecycle owner for Settings and About Surface. Callers request the existing singleton openers; callers do not construct windows or own drafts.
- Introduce one small invocation-context contract with exactly `Drop Shelf` and `Notification Area` cases. The Drop Shelf utility command passes the former; the native notification-area menu passes the latter.
- Reuse one narrow Windows auxiliary-window placement seam for invocation-context monitor selection, complete-outer-frame measurement, centered work-area calculation, and physical positioning. Move or expose the already-proven About placement behavior through that seam rather than duplicating the formula or making Settings depend on About-specific internals.
- Do not create a generalized window manager, placement service, configurable placement policy, or abstraction for surfaces that do not share this contract.
- For `Drop Shelf` invocation, select the monitor containing the current Drop Shelf window. If it cannot be resolved, use the Windows primary monitor.
- For `Notification Area` invocation, select the monitor containing the current pointer. If the pointer monitor cannot be resolved, use the Windows primary monitor.
- Build a new Settings window hidden, measure its complete decorated outer frame in physical pixels, center and clamp that frame within the selected monitor work area, then show and focus it. Do not expose a top-left unpositioned frame during setup.
- Keep Settings at its established `420 × 520` logical inner size, fixed-size behavior, decorated title, normal z-order, and existing label-based frontend composition.
- Create Settings as a normal top-level native window. Do not assign the hidden Drop Shelf as its native owner. Keep taskbar skipping disabled so Windows supplies the normal taskbar button.
- Keep Settings non-modal and non-topmost. Removing native ownership must not disable the Drop Shelf or block unrelated DropCove interaction.
- If Settings already exists, show it, restore it if minimized, and focus it without changing its current position. Do not create a duplicate or replace its current draft.
- Preserve current Settings close-event handling. Save still applies the existing transactional settings operation and closes on success; Cancel or native close discards the draft; focus loss still cancels shortcut recording only.
- Preserve explicit Resident exit and Cooperative Shutdown authority. The existing Settings shutdown path closes the top-level window and tears down hotkey and Shake resources in the established order.
- Opening Settings from the notification area must not show, move, activate, or change the Drop Shelf or Edge Rail. Opening it from the Drop Shelf Utility Menu closes the transient menu before focus transfers.
- Keep current Settings command authorization and label-based frontend composition. The invocation context is supplied by trusted resident callers; no user-controlled arbitrary monitor identifier is added.
- Change Cargo package metadata exactly to `authors = ["HungNth"]` and `description = "A resident Windows utility providing a temporary drag-and-drop workspace for file and folder references."`.
- Extend the existing read-only About metadata projection with one display-ready `description` field sourced from the Cargo package description available to the compiled application.
- Keep product name and version sourced from the running package. Keep author, description, and Source repository sourced from Cargo package metadata. Do not duplicate any of those values in Vue, Tauri configuration, or another manifest.
- Render the description immediately below the runtime version in the About Surface product-identity block. Preserve the order product name, version, description, metadata rows, update placeholder.
- Keep the existing fixed `400 × 460` logical About Surface. The description uses the established restrained text treatment, centered alignment, and bounded wrapping; it must remain fully readable without adding scrolling.
- Preserve About Surface singleton, taskbar, positioning, Source opener, error handling, `Escape`, close, disabled Check for updates, and Coming soon behavior.
- Adding the description to the existing metadata response requires no second IPC command, new capability, filesystem access, network access, dependency, or updater behavior.
- Preserve Settings persistence, hotkey registration, Start with Windows, Edge Rail placement, fullscreen, Shake-to-open, warning, rollback, and Profile Data contracts. This release changes no serialized setting or database schema.
- Preserve ADR-0002 placement ownership. Settings initial centering does not own or rewrite configured Rail monitor preference, remembered rail monitor, Drop Shelf coordinates, or any durable position.
- Update canonical product and architecture documentation during delivery to describe Settings as a taskbar-visible top-level auxiliary surface, the shared invocation-context placement contract, and Cargo description projection.
- No new ADR is required. Top-level versus owned Settings is a reversible presentation choice localized behind the existing Resident/Presentation ownership, introduces no durable state or public data contract, and is fully captured by this specification.
- Keep all canonical version sources at `0.0.16` until the two implementation slices pass behavioral acceptance. Synchronize them to `0.0.17` only in the release slice.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Use one primary behavioral seam: the actual Windows Resident Instance with real WebView2 Settings and About Surface, the Drop Shelf Utility Menu, native notification-area menu, and Windows taskbar. This highest seam proves invocation routing, native ownership, taskbar presence, window lifecycle, physical placement, frontend metadata, Settings state, and resident shutdown together.
- The user explicitly approved this primary seam during `to-spec`. Supporting deterministic checks improve diagnosis but do not replace actual Windows observation.
- A good permanent test catches a plausible consumer-visible failure: wrong monitor, off-work-area placement, duplicate Settings windows, unexpected repositioning, missing taskbar presence in the running application, lost draft, broken Save/Cancel, stranded shutdown, incorrect Cargo metadata, duplicated frontend copy, or clipped About description.
- Reuse the existing deterministic centered-and-clamped outer-frame calculation seam. Extend it only as needed to cover Settings frame dimensions, negative monitor coordinates, primary fallback inputs, and work areas smaller than the frame.
- Prefer one shared placement calculation test surface used by both Settings and About behavior rather than copying the same expected geometry into separate module-private suites.
- Keep target-monitor routing behavior at the resident/native window seam. Do not permanently test that one command merely forwards an enum value; prove the selected monitor in the running application.
- Reuse the existing Settings coordinator and real settings persistence regressions for Save, Cancel, recording cancellation, rollback, and shutdown. Do not duplicate established settings transaction tests solely because native ownership changes.
- Reuse the existing About metadata projection seam for product name, runtime version, author, description, and Source. Assert returned display metadata against authoritative compiled package values, not source-file text.
- Run the complete existing Windows native regression suite to catch Settings, About, Resident Instance, tray, hotkey, startup, Edge Rail, Shake-to-open, Cooperative Shutdown, persistence, and source-safety regressions.
- Run the complete frontend production build for Vue single-file-component typechecking and bundling. Do not add a frontend component-test framework or screenshot snapshot system for one description line.
- Compile with cargo-xwin for `x86_64-pc-windows-msvc`, then run the produced Windows native regression executable directly. Compile-only output or an unavailable runner is not runtime proof.
- Open a new Settings window from the Drop Shelf on each available monitor. Verify the complete decorated outer frame is centered and contained in that monitor work area and a normal Settings taskbar button is visible.
- Open a new Settings window from the notification-area menu with the pointer on each available monitor. Verify pointer-monitor centering, work-area containment, taskbar presence, and unchanged Drop Shelf/Edge Rail state.
- Exercise monitor layouts with non-zero or negative coordinates where available. Record unavailable high-DPI or mixed-DPI hardware as unobserved rather than inferring physical coverage from formula tests.
- Move the existing Settings window manually, invoke Settings again from both entry points, and verify one window is shown/restored/focused without moving from the user-selected position.
- Minimize Settings through the Windows shell and request it again. Verify the existing window restores and receives focus without creating a duplicate.
- Verify taskbar navigation while the Drop Shelf is visible, hidden, and EdgeDocked. Settings remains independently reachable and the Shelf display state remains unchanged.
- Exercise Settings Save, Cancel, native close, shortcut recording, recorder focus loss, startup preference, Rail placement, fullscreen preference, and Shake preference after the ownership change. Verify established draft, transaction, rollback, warning, and close behavior.
- Exercise explicit Exit DropCove and Cooperative Shutdown with Settings open. Verify the top-level Settings window closes, teardown completes, and no Resident Instance remains stranded.
- Open Settings from the Drop Shelf Utility Menu and verify the menu closes before focus transfers. Open it from the notification area and verify no Shelf surface is shown or moved.
- Verify the running About metadata reports author exactly `HungNth` and description exactly `A resident Windows utility providing a temporary drag-and-drop workspace for file and folder references.` from compiled Cargo metadata.
- Inspect About Surface with the exact description below the synchronized runtime version. Verify complete text, centered bounded wrapping, no clipping, no scrollbar, and unchanged Author, Source, Check for updates, Coming soon, close, and `Escape` behavior.
- Inspect About and Settings in Dark, Light, actual Windows Contrast Theme where available, and reduced-motion presentation. Verify text legibility, native taskbar behavior, focus indicators, disabled state, and reading order.
- Inspect accessibility with available Windows tooling: Settings and About window titles, one Settings taskbar representation, product-name/version/description reading order, link/button names, visible focus, and duplicate-window absence.
- Before the release bump, verify the running application and every canonical manifest still report `0.0.16` while About already obtains author and description from the compiled candidate metadata as appropriate for the implementation build.
- After implementation acceptance, synchronize all canonical version sources to `0.0.17`, build one exact current-user Windows x64 NSIS candidate, and record installer and installed-executable byte sizes and SHA-256 hashes.
- Exercise the exact installed candidate with embedded frontend assets and the development server unavailable. Cover both Settings entry points, taskbar navigation, initial centering, singleton reuse, Save/Cancel, About metadata, Resident activation, Cooperative Shutdown, same-candidate reinstall/update, uninstall, and Profile Data preservation.
- Add no permanent tests that assert builder-call text, CSS utility strings, Vue property names, manifest source text, command forwarding, mock echoes, non-empty markup, incidental coordinates unrelated to a work-area invariant, or bare non-throw behavior.
- Record unavailable physical multi-monitor, high/mixed-DPI, Windows Contrast, screen-reader, or Shell automation coverage explicitly. Do not reclassify deterministic calculation tests or media emulation as physical observation.

## Out of Scope

- Remembering, persisting, or restoring a user-moved Settings position across close, restart, update, or sign-in.
- Adding a Settings placement preference, monitor selector, centering toggle, snapping policy, or user-configurable taskbar behavior.
- Changing taskbar behavior for Drop Shelf, Edge Rail, Rail Handle, Manage Items, Clear confirmation, or any other auxiliary surface.
- Redesigning Settings, changing its dimensions, adding or removing controls, changing Save/Cancel semantics, or changing hotkey, startup, Edge Rail, fullscreen, or Shake-to-open behavior.
- Making Settings modal, always-on-top, owned by About Surface, or owned by another visible window.
- Adding multiple Settings windows, multiple independent drafts, tabs, routing, or a general auxiliary-window framework.
- Persisting invocation context or monitor identity as part of Settings or Profile Data.
- Changing About Surface placement, singleton, taskbar, Source, update placeholder, dimensions, close behavior, or metadata other than author and the requested description.
- Adding license, copyright, acknowledgements, release notes, changelog, support, donation, telemetry, diagnostics, homepage, or Website content to About Surface.
- Implementing Check for updates, adding an updater dependency, performing network checks, signing releases, or changing installer-based update policy.
- Adding a second About metadata command, reading manifests from Vue, hard-coding metadata in frontend source, or duplicating description in Tauri configuration.
- Changing Shelf state, Shelf Batch or Shelf Item lifecycle, source-file safety, persistence, sizing, Drop Placeholder behavior, Edge Rail placement ownership, or Profile Data retention.
- Adding database migrations, settings-schema fields, compatibility aliases, fallback windows, or legacy ownership shims.
- Supporting macOS, Linux, ARM64, x86, mobile, or non-MSVC Windows targets. The target remains Windows x64 through cargo-xwin.
- Starting implementation, bumping to `0.0.17`, packaging, or committing before the required workflow and approval gates close.
- Rewriting completed 0.0.15 or 0.0.16 specifications, release evidence, historical author values, or accepted artifact records.

## Further Notes

- **Settings** and **About Surface** are the canonical glossary terms. Do not call them the Settings page, preferences page, About page, popup, or modal in new product documentation.
- The current taskbar defect is not caused by `skip_taskbar` being enabled; Settings already requests taskbar visibility. The relevant behavioral difference is native ownership by the hidden Drop Shelf, while About Surface is a normal top-level window.
- The approved monitor contract intentionally matches About Surface: Drop Shelf invocation follows the Drop Shelf monitor, notification-area invocation follows the pointer monitor, and repeated requests preserve an existing window position.
- The exact approved Cargo description is **A resident Windows utility providing a temporary drag-and-drop workspace for file and folder references.**
- Cargo package metadata owns author, description, and repository. The running Tauri package owns product name and synchronized runtime version.
- The existing About Surface dimensions and visual hierarchy are expected to accommodate the description through restrained wrapping. If actual WebView2 proof finds clipping, adjust internal spacing within the existing surface before considering a dimension change; changing the native size requires a specification decision.
- No glossary change is required because Settings and About Surface are already defined and this feature introduces no new domain term.
- No ADR is required because the ownership change is reversible presentation behavior with no durable-state or hard-to-reverse platform contract. The reason for the choice and rejected owned-window behavior is recorded here.
- ADR-0002 remains authoritative and unaffected. Settings initial placement does not become Rail placement preference or remembered dismissal placement.
- The user confirmed the design tree and three-slice roadmap entry through `grill-with-docs`, then explicitly approved the primary actual-Windows testing seam during `to-spec` on 2026-10-20. Seam approval does not approve tickets or implementation.
- Final human acceptance of `0.0.16` is complete by user report. The approved 0.0.17 tickets may now be implemented in dependency order; version synchronization remains in the release ticket.
- Git commit authorization is always separate from specification, tickets, implementation, verification, release acceptance, and version synchronization.

## Final human acceptance and commit authorization

The user confirmed `tất cả đã đạt, hãy commit các thay đổi`, accepting the exact `0.0.17` candidate and explicitly authorizing the commit. Historical verification limits remain preserved in the release ticket and roadmap.
