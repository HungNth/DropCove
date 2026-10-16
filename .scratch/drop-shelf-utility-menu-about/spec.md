# DropCove 0.0.15 — Drop Shelf Utility Menu and About Surface

Status: resolved
Version: 0.0.15
Design confirmation: approved by user on 2026-10-16
Testing seam: actual Windows Resident Instance with the real WebView2 and native Drop Shelf, Clear Temporary Items confirmation, Settings, About Surface, and notification-area menu; approved by user on 2026-10-16
Delivery prerequisite: completed — user confirmed final 0.0.14 human acceptance on 2026-10-16
Ticket decomposition: all four approved tickets are resolved; user confirmed final 0.0.15 acceptance and explicitly authorized committing the changes.
Implementation approval: user authorized all four tickets after accepting 0.0.14; final 0.0.15 acceptance and Git commit authorization are now explicitly granted.
Parent roadmap: [DropCove Roadmap](../../docs/ROADMAP.md), version 0.0.15
Canonical behavior: [Product Specification](../../docs/SPECIFICATION.md), sections 3, 5, 8, 11, 13, and 15
Canonical architecture: [Architecture](../../docs/ARCHITECTURE.md), sections 2, 3, 5, 8, 10, and 11
Placement ownership: [ADR-0002](../../docs/adr/0002-edge-rail-placement-ownership.md)

## Problem Statement

The compact Drop Shelf can be as small as `191 × 180` logical pixels. Its current Clear Temporary Items confirmation is rendered inside that WebView, so the confirmation is constrained by the Drop Shelf viewport. At minimum size, the explanation and actions can be clipped, wrapped excessively, or forced behind the available scroll area. The user cannot reliably read the safety statement or reach every action before deciding whether to remove Temporary Items.

The Drop Shelf also has no direct entry point for resident-level utilities. Settings is available only from the Windows notification-area menu, and DropCove has no About Surface. Users must leave the Drop Shelf to discover Settings and cannot inspect the installed product identity, synchronized runtime version, author, or source location from the application itself.

A naive presentation-only change would be unsafe. A Vue teleport cannot escape the native Drop Shelf viewport. A second Clear implementation could bypass the existing serialized Shelf mutation, stale-state checks, persistence rollback, Pinned Item retention, and source-file safety. Independent tray and Drop Shelf About implementations could create duplicate windows, inconsistent metadata, or conflicting focus behavior. Hard-coded author, source, or version strings could drift from the installed binary and packaging metadata.

DropCove needs one coherent resident utility slice: a readable Clear Temporary Items confirmation outside the compact Drop Shelf, a compact Drop Shelf Utility Menu, one reusable About Surface, and one notification-area About command, all without changing Shelf semantics, Edge Rail behavior, persistence, or the Resident Instance lifecycle.

## Solution

Move only the Drop Shelf Clear Temporary Items confirmation to a dedicated, decorated Tauri window. The window is fixed-size, taskbar-hidden, always-on-top, centered on the Drop Shelf monitor, and modal to Drop Shelf interaction. It presents the shorter copy **Clear temporary items?** and **Pinned items stay. Source files and folders aren’t deleted.** `Cancel`, `Escape`, and the window close action cancel without mutation. `Clear` revalidates current Temporary Item availability and delegates to the existing atomic Clear Temporary Items operation.

Add an always-visible `24 × 24` **More** action at the Drop Shelf’s bottom-right. It opens an in-shelf Drop Shelf Utility Menu above the action with **Settings** followed by **About**. The menu follows an accessible menu-button interaction, closes on outside click or `Escape`, and returns focus when dismissal does not transfer focus to a newly opened surface. Settings uses its existing singleton surface and lifecycle.

Add one decorated, fixed-size, taskbar-visible About Surface. It is non-modal and single-instance: Drop Shelf and notification-area requests use the same native opener, repeated requests show, raise, and focus the existing surface, and closing it permits recreation. A newly created About Surface is centered on the Drop Shelf monitor when invoked from the Drop Shelf and on the pointer monitor when invoked from the notification area.

The About Surface reuses the canonical DropCove logo and displays the product name, synchronized runtime/build version, author, and a **Source** link. Cargo package metadata owns author and repository. The repository is not called a Website, and Cargo homepage remains unset until DropCove has a public product site. One read-only Rust metadata projection provides the frontend with display metadata from the authoritative build sources. The Source link opens the repository in the default browser through the already-installed opener integration.

Show a disabled **Check for updates** action with **Coming soon**. This reserves the intended About layout without adding an updater, background check, fake success result, clickable no-op, dependency, capability, or release endpoint.

Add **About** between **Settings** and **Exit DropCove** in the Windows notification-area menu. It opens or focuses the same About Surface without showing the Drop Shelf, changing Shelf state, or affecting Resident Instance lifetime.

Use the actual Windows Resident Instance as the single primary testing seam. Exercise the real WebView2 and native surfaces, notification-area menu, focus and positioning behavior, metadata, opener integration, Shelf mutation, installed assets, and exact NSIS candidate together. Existing native regressions and production builds support this seam; no frontend component-test framework or screenshot snapshot system is introduced.

## User Stories

1. As a Drop Shelf user, I want the Clear Temporary Items confirmation outside the compact Drop Shelf viewport, so that I can read the complete message and reach every action.
2. As a user with the Drop Shelf at its minimum size, I want the confirmation to remain fully usable, so that window size cannot hide a destructive-looking decision.
3. As a user clearing Temporary Items, I want concise confirmation copy, so that I can understand the result quickly.
4. As a user, I want the confirmation to state that Pinned Items stay, so that I know retained references are safe.
5. As a user, I want the confirmation to state that source files and folders are not deleted, so that the action cannot be mistaken for filesystem deletion.
6. As a cautious user, I want `Cancel` focused initially, so that an accidental Enter does not clear Temporary Items.
7. As a keyboard user, I want `Escape` to cancel the Clear confirmation, so that I can leave without reaching for the pointer.
8. As a pointer user, I want the confirmation window close action to cancel, so that standard Windows close behavior is safe.
9. As a user who cancels, I want every Temporary and Pinned Item retained, so that cancellation has no side effects.
10. As a user who confirms, I want all current Temporary Items removed across Shelf Batches, so that Clear retains its established meaning.
11. As a user with mixed Temporary and Pinned Items, I want only Temporary Items removed, so that Pinned references remain available.
12. As a user, I want Clear to re-check the current Shelf state when I confirm, so that a stale confirmation cannot apply an outdated assumption.
13. As a user whose Temporary Items are no longer present, I want the stale confirmation to close safely without changing unrelated state.
14. As a user encountering a persistence failure, I want visible and durable Shelf state left unchanged and the failure reported, so that Clear remains atomic.
15. As a user, I want duplicate Clear requests to focus the existing confirmation, so that multiple confirmation windows cannot compete.
16. As a user deciding whether to clear, I want Drop Shelf dismissal, movement, resize, drag-out, and utility actions blocked, so that the confirmation remains modal to its owner.
17. As a user explicitly exiting DropCove, I want normal resident shutdown to close the confirmation, so that modal presentation cannot prevent Exit DropCove or Cooperative Shutdown.
18. As an Edge Rail user, I want the existing Edge Rail Clear confirmation unchanged in 0.0.15, so that this release does not expand into Rail transient behavior.
19. As a Drop Shelf user, I want a visible More action at the bottom-right, so that resident utilities are discoverable without opening the notification area.
20. As a user with an empty Shelf, I want More to remain available, so that Settings and About do not depend on Shelf content.
21. As a user with Temporary Items, I want Clear to remain at the bottom-left and More at the bottom-right, so that destructive and utility actions are spatially distinct.
22. As a pointer user, I want the Drop Shelf Utility Menu to open above More, so that both utility actions fit within the compact surface.
23. As a keyboard user, I want More reachable in the natural Tab order with a visible focus indicator, so that utility access does not require a pointer.
24. As a keyboard user, I want the menu to support standard menu-button activation and navigation, so that Settings and About are predictable.
25. As a keyboard user, I want `Escape` to close the Drop Shelf Utility Menu and return focus to More, so that I can recover my place.
26. As a pointer user, I want an outside click to close the utility menu, so that it behaves like a transient popover.
27. As a screen-reader user, I want More to expose its expanded state and menu relationship, so that the transient UI is understandable.
28. As a user choosing Settings, I want the established Settings surface opened or focused, so that no second settings lifecycle is introduced.
29. As a user choosing About, I want the utility menu to close before About receives focus, so that two transient interactions do not overlap.
30. As a user, I want a centered About Surface, so that product information appears as a deliberate application surface rather than inside the tiny Drop Shelf.
31. As a user, I want About to show the canonical DropCove logo, so that the surface is immediately recognizable.
32. As a user, I want About to show the product name, so that I can identify the application.
33. As a user, I want About to show the installed runtime version, so that support and release discussions refer to the actual running binary.
34. As a user running a development build before the release bump, I want About to show `0.0.14`, so that a roadmap target is not misrepresented as the installed version.
35. As a user running the synchronized 0.0.15 candidate, I want About to show `0.0.15`, so that the surface follows the released build metadata automatically.
36. As a user, I want About to show `DropCove Contributors` from canonical package metadata, so that author attribution does not drift across surfaces.
37. As a user, I want the GitHub location labeled **Source**, so that a repository is not misrepresented as a product Website.
38. As a user, I want Source to open in my default browser, so that I can reach the canonical repository directly.
39. As a user whose browser cannot be opened, I want the About Surface to report the failure without closing or changing Shelf state, so that the error is actionable.
40. As a user, I do not want an empty Website field, so that About does not display invented or misleading product metadata.
41. As a user, I want **Check for updates** visible but disabled with **Coming soon**, so that its future location is clear without implying a check occurred.
42. As a user, I do not want a clickable update control that does nothing, so that the interface does not fake functionality.
43. As a user, I want About to be non-modal, so that I can keep using DropCove or Settings while reading product information.
44. As a user opening About repeatedly, I want the existing surface shown, raised, and focused, so that duplicate windows are not created.
45. As a user closing About, I want a later request to recreate it cleanly, so that close behaves like a standard auxiliary window.
46. As a user opening About from the Drop Shelf, I want a new About Surface centered on the Drop Shelf monitor, so that it appears in my current work area.
47. As a user opening About from the notification area, I want a new About Surface centered on the pointer monitor, so that multi-monitor placement follows the invocation context.
48. As a user who manually moved an open About Surface, I want repeated requests to focus it without unexpectedly moving it, so that reopening does not fight my placement.
49. As a user opening About from the notification area while the Drop Shelf is hidden, I want About to open without showing the Shelf, so that utility access does not change presentation state.
50. As a notification-area user, I want **About** between **Settings** and **Exit DropCove**, so that informational and configuration commands remain separate from process exit.
51. As a user, I want tray About and Drop Shelf About to open the same surface, so that content and lifecycle are identical.
52. As a user, I want opening About or Settings to leave the Resident Instance alive, so that resident access remains available.
53. As a Dark presentation user, I want the utility menu, Clear confirmation, and About Surface to use the established DropCove tokens, so that the new surfaces feel native to the application.
54. As a Light presentation user, I want text, borders, controls, and logo treatment to remain legible, so that the design is not dark-theme-only.
55. As a Windows Contrast user, I want system colors, visible boundaries, disabled state, and focus indicators preserved, so that the new UI remains operable.
56. As a reduced-motion user, I want no decorative animation required to understand menu or window state, so that motion preferences remain respected.
57. As a keyboard user, I want every enabled action operable with standard keyboard input and visible focus, so that the feature is accessible without a pointer.
58. As a screen-reader user, I want concise names, roles, states, and reading order, so that icon and metadata presentation are understandable.
59. As a user, I want the logo to be the sole visual emphasis and metadata to remain restrained, so that About matches DropCove rather than a generic card-heavy template.
60. As a maintainer, I want one resident-owned opener per auxiliary surface, so that tray and frontend callers cannot create competing lifecycles.
61. As a maintainer, I want one read-only About metadata interface, so that the frontend does not know manifest locations or repeat literals.
62. As a maintainer, I want author and repository owned by Cargo package metadata, so that About and packaging share authoritative identity data.
63. As a maintainer, I want Cargo homepage left unset until a real product site exists, so that installer and About metadata stay semantically correct.
64. As a maintainer, I want the existing Shelf mutation to remain the only Clear implementation, so that persistence and rollback behavior stay local.
65. As a security-conscious maintainer, I want frontend window-opening commands restricted to their intended calling surfaces, so that arbitrary windows cannot trigger resident presentation changes.
66. As a release owner, I want one actual-Windows seam covering native windows, WebView2, tray, focus, metadata, opener, and Shelf state, so that the integrated behavior is proven together.
67. As a release owner, I want the exact installed NSIS candidate exercised with embedded assets and the development server stopped, so that accepted behavior matches shipped bytes.
68. As a maintainer, I want no new database field, settings field, migration, dependency, updater, telemetry, or compatibility path, so that 0.0.15 remains the smallest complete slice.
69. As a user, I want source files and folders untouched by utility, About, cancel, failure, and confirmed Clear paths, so that the feature cannot introduce deletion behavior.
70. As a release owner, I want version metadata synchronized only after implementation and actual-surface acceptance, so that About never reports an unreleased milestone prematurely.

## Implementation Decisions

- Treat 0.0.15 as one resident utility and informational-surface slice. It follows 0.0.14 and must not expand or retroactively modify the accepted scope of earlier compact-presentation milestones.
- Do not start implementation until final 0.0.14 human acceptance is complete and the 0.0.15 tickets are created and approved. `ready-for-agent` marks this specification as complete; it does not grant implementation or Git commit authorization.
- Keep the Core Shelf State Module as the only owner of Clear Temporary Items mutation semantics. Presentation requests confirmation; confirmation acceptance delegates to the existing serialized mutation and refresh path.
- Introduce one resident-owned Clear confirmation module with a small interface for request, confirm, cancel, state refresh, and shutdown cleanup. Its implementation owns the native confirmation window and the Drop Shelf modal guard; it does not own Shelf data.
- Permit the Drop Shelf as the only product caller that can request this dedicated confirmation in 0.0.15. Reject unauthorized frontend callers without creating or focusing a window or changing Shelf state.
- Use one decorated Tauri WebView window for the Drop Shelf confirmation. Its title is `DropCove`; its logical size is `360 × 200`; it is fixed-size, non-minimizable, non-maximizable, taskbar-hidden, always-on-top, and owned by the Drop Shelf.
- Center a newly created confirmation within the work area of the monitor containing the Drop Shelf. Clamp the complete outer frame to that work area and honor Per-Monitor V2 DPI behavior.
- Reuse an existing confirmation by refreshing its current availability projection, then showing, raising, and focusing it. Never create duplicate Clear confirmation windows.
- Present the exact heading `Clear temporary items?`, body `Pinned items stay. Source files and folders aren’t deleted.`, and actions `Cancel` and `Clear`. Focus `Cancel` when the window opens.
- Treat the confirmation as modal to Drop Shelf interaction even though it is a separate native window. While it is open, block Dismiss Drop Shelf, native movement, resize initiation, Shelf drag-out, Clear re-entry, Drop Shelf Utility Menu activation, and other Drop Shelf transactional interaction.
- Keep explicit Resident exit and Cooperative Shutdown authoritative. Shutdown closes the confirmation and clears its modal guard before normal teardown completes.
- Keep the canonical Edge Rail fullscreen policy scoped to the Edge Rail's own local Clear prompt. Fullscreen hiding of the Edge Rail continues to discard only that Rail prompt; the dedicated Drop Shelf confirmation has no Edge Rail fullscreen lifecycle because the Edge Rail is not its owner.
- `Cancel`, `Escape`, and the confirmation window close action all close the window, clear the modal guard, preserve every Shelf Item, and restore focus to the Clear action when that action still exists and the Drop Shelf remains active.
- Confirm revalidates current Temporary Item availability in the backend before mutation. It must not trust an earlier frontend snapshot. If no Temporary Items remain, close the confirmation, clear the modal guard, publish current Shelf state, and perform no removal.
- Confirm uses the existing atomic Clear Temporary Items operation. It removes Temporary Items across all Shelf Batches, preserves Pinned Items, never deletes source filesystem objects, and preserves prior visible and durable state on persistence failure.
- Preserve existing post-confirm behavior: once confirmation is accepted, close the confirmation and release its modal guard; if mutation fails, report the failure through existing Shelf status or resident warning behavior rather than reopening the confirmation or inventing a fallback mutation.
- Keep the expanded Edge Rail Clear Temporary Items confirmation, transient hold, fullscreen cancellation, and collapse behavior unchanged. Do not route Edge Rail Clear through the new window in this version.
- Add one always-visible `24 × 24` bottom-right More action to the Drop Shelf footer. Keep the conditional Clear Temporary Items action at bottom-left. The footer remains present when the Shelf has no Temporary Items so More stays discoverable.
- Use the established icon language and a More/ellipsis glyph that inherits `currentColor`. Provide the visible/native tooltip and accessible name `More`.
- Implement the Drop Shelf Utility Menu as an in-WebView popover anchored above More, approximately `132` logical pixels wide, containing exactly `Settings` followed by `About`.
- Follow the accessible menu-button pattern: More exposes menu availability and expanded state; opening moves focus to the first item; Up/Down, Home/End, Enter, Space, Tab dismissal, and `Escape` follow standard menu behavior; outside click dismisses the menu.
- Return focus to More after dismissal when focus has not intentionally transferred to Settings or About. Close the menu before invoking either native opener.
- Reuse the established singleton Settings opener. Add only the minimal frontend-to-resident request needed by the Drop Shelf; Settings state, Save/Cancel, startup, hotkey, Edge Rail placement, warnings, and rollback remain unchanged.
- Introduce one resident-owned About module with a small open/focus/close/shutdown interface. Both the Drop Shelf and notification-area menu call the same module; neither caller constructs or owns the native window independently.
- Use one decorated Tauri WebView window for About. Its title is `About DropCove`; its initial logical size is `400 × 460`; it is fixed-size, taskbar-visible, non-modal, and single-instance.
- When creating About from the Drop Shelf, center its complete outer frame within the Drop Shelf monitor work area. When creating it from the notification-area menu, center it within the monitor work area containing the pointer.
- If About already exists, show, raise, and focus it without changing its current position. If the user closes it with `X` or `Escape`, destroy the window and allow a later request to recreate and recenter it.
- Opening About from the notification area must not show, activate, move, or otherwise change the Drop Shelf or Edge Rail. Opening About does not change Shelf state or Resident Instance lifetime.
- Route the About and Clear views through the existing label-based frontend composition rather than adding another frontend entry point, router, application instance, or general window framework.
- Add only the new window labels and least required capabilities. Do not grant About or Clear windows Shelf mutation, window-drag, filesystem, or unrelated resident permissions.
- Reuse the canonical bundled DropCove logo asset. Do not create a second logo, remote image fetch, data copy, or decorative illustration.
- Use one read-only About metadata interface from Rust to the frontend. It returns display-ready product name, runtime version, author, and source URL from authoritative build metadata; the frontend does not read manifests directly or contain duplicated literals.
- Keep the existing synchronized application-version pipeline authoritative. About reads the runtime package version. Development remains `0.0.14` until the approved release bump; the synchronized 0.0.15 candidate then reports `0.0.15` automatically.
- Keep `DropCove Contributors` in Cargo package authors as the canonical About author. Add the canonical GitHub URL as Cargo package repository. Do not duplicate these values in frontend source, package metadata for unrelated tooling, or Tauri bundle configuration.
- Leave Cargo package homepage and Tauri bundle homepage unset until DropCove has a real public product site. Label the repository link `Source`, never `Website`.
- Open Source through the already-installed Tauri opener integration and default browser. Keep the visible source text available. On opener failure, leave About open and report a concise, non-destructive error within the surface.
- Show only the requested About content: logo, DropCove, runtime version, Author, Source, disabled Check for updates, and Coming soon. Do not add description, license, copyright, changelog, release notes, telemetry, donation, or support content.
- Render Check for updates as a genuinely disabled control with adjacent or associated `Coming soon` text. Do not attach a click handler, install an updater, create an endpoint, request updater permissions, or report a fabricated result.
- Add About between Settings and Exit DropCove in the native Windows notification-area menu. Keep left-click/double-click show-Shelf behavior and Exit DropCove semantics unchanged.
- Preserve the existing Settings and Exit warning path when a native utility request fails. About opening failures use the same resident warning ownership; the Drop Shelf menu additionally retains focus and may show its existing status presentation.
- Follow the accepted compact frontend-design plan: Segoe UI; existing canvas, surface, elevated-surface, text, border, accent, focus, danger, and disabled tokens; logo as the single visual emphasis; centered product identity; restrained left-aligned metadata rows; one quiet bottom action area; no gradient, decorative card stack, generic SaaS chrome, or unnecessary animation.
- Preserve Dark, Light, Windows Contrast, reduced-motion, visible focus, accessible names, disabled-state communication, logical reading order, and minimum target sizing across all new controls and surfaces.
- Add no database schema, settings field, persisted window geometry, migration, dependency, updater, background task, polling loop, telemetry, generalized navigation abstraction, generalized auxiliary-window manager, or compatibility layer.
- Update the canonical product and architecture documentation during delivery to describe the new resident utility entry points, native confirmation ownership, About metadata projection, and notification-area menu. Keep ADR-0002 unchanged because Edge Rail placement ownership is unchanged.
- No new ADR is required. The native confirmation choice records a real viewport and accessibility trade-off, but it remains a reversible presentation implementation behind the existing Resident and Core Shelf State ownership, adds no persistence or new state owner, and is fully captured by this specification.
- Do not create a Git commit without separate explicit user authorization.

## Testing Decisions

- Use one primary behavioral seam: the actual Windows Resident Instance with the real WebView2 Drop Shelf and native Clear confirmation, Settings, About Surface, and notification-area menu. This highest seam proves native-window lifecycle, cross-window focus, modal ownership, monitor placement, frontend presentation, tray routing, metadata, opener behavior, and Shelf mutation together.
- Use existing native regression suites as supporting coverage for serialized Clear Temporary Items semantics, Temporary/Pinned lifecycle, atomic rollback, resident visibility, Settings lifecycle, monitor geometry, shutdown, and installer behavior. Add permanent cases only for plausible new user-visible regressions not already covered.
- Run the complete frontend production build for Vue typechecking and bundling. Do not add a frontend component-test framework solely for the Drop Shelf Utility Menu or About Surface.
- Compile the Windows target with cargo-xwin, then run the complete produced Windows native test executable directly. A compile-only result or unavailable runner is not runtime proof.
- Exercise the Drop Shelf at the `191 × 180` minimum with one Shelf Batch and Temporary content. Open Clear and verify the complete heading, safety copy, Cancel, and Clear actions are visible without relying on the Drop Shelf scrollport.
- Measure the Clear confirmation’s logical client size, complete outer-frame work-area containment, fixed-size behavior, taskbar absence, decoration, topmost behavior, and initial Cancel focus on the actual Windows surface.
- Exercise Clear cancellation through Cancel, `Escape`, and the window close action. Verify every Temporary and Pinned reference remains, source bytes remain unchanged, the modal guard clears, and focus returns coherently.
- Request Clear repeatedly while the confirmation is open. Verify one native window exists, current availability is refreshed, and the existing window is shown, raised, and focused.
- While Clear is open, attempt Hide Drop Shelf, native movement, resize, Shelf drag-out, More, repeated Clear, and relevant keyboard dismissal paths. Verify the Drop Shelf remains visible and stationary and no competing interaction begins.
- Exercise explicit Exit DropCove and Cooperative Shutdown while Clear is open. Verify confirmation cleanup does not deadlock or strand the Resident Instance.
- Exercise stale availability by changing authoritative Temporary Item state before confirmation acceptance through a deterministic native seam. Confirm must re-check current state, avoid removing unrelated items, close stale presentation safely, and publish the current snapshot.
- Exercise mixed Temporary/Pinned Shelf Batches. Confirm removes all Temporary Items across batches, preserves Pinned Items and batch ordering, closes empty batches as established, and never deletes source files or folders.
- Exercise the existing realistic persistence-failure seam for Clear where practical. Confirm the prior visible and durable state remains, the confirmation closes as specified, and the failure is reported without fallback deletion.
- Exercise the Edge Rail Clear action and its fullscreen/transient behavior to prove it remains unchanged and does not open the new Drop Shelf confirmation window.
- Inspect empty, Temporary-only, Pinned-only, mixed, status-bearing, automatically grown, and manually resized Drop Shelves. More remains bottom-right and Clear remains conditional bottom-left without obscuring content or resize handles.
- Exercise More and the Drop Shelf Utility Menu with pointer, Tab, Shift+Tab, Enter, Space, arrow keys, Home, End, `Escape`, outside click, and focus transfer. Verify accessible name, expanded state, menu roles, item order, visible focus, and focus return.
- Open Settings from the utility menu. Verify the existing Settings surface is shown or focused, no duplicate Settings window is created, the menu closes first, and Settings Save/Cancel behavior is unchanged.
- Open About from the utility menu. Verify the menu closes first and one About Surface receives focus.
- Open About from the notification-area menu with the Drop Shelf visible, hidden, and EdgeDocked. Verify the same About Surface opens or focuses and no Shelf presentation state changes.
- Invoke About repeatedly from both entry points. Verify one window exists, an existing window is shown/raised/focused without repositioning, close destroys it, and a later request recreates it.
- Create About from the Drop Shelf on each available monitor and from the notification area with the pointer on each available monitor. Observe correct initial centering, work-area containment, DPI scale, fixed sizing, and taskbar presence.
- Verify the About reading order and exact requested content. Inspect the canonical logo, DropCove name, runtime version, Author, Source, disabled Check for updates, and Coming soon; confirm no Website or extra metadata appears.
- Before the release bump, verify About reports the actual synchronized `0.0.14` build version. After the approved synchronized bump, verify the exact candidate reports `0.0.15`. Never accept a hard-coded milestone value.
- Verify Author and Source match Cargo package metadata in the running binary. Confirm homepage remains absent and neither About nor installer metadata presents the repository as a product Website.
- Activate Source with pointer and keyboard and observe the default browser opening the canonical repository URL through the real opener integration. Exercise a practical opener failure seam where available and verify About stays open with a clear error.
- Verify Check for updates is visibly and programmatically disabled, skipped or announced according to native disabled semantics, has no click result, performs no network activity, and creates no updater configuration or background work.
- Inspect Clear, Utility Menu, Settings handoff, and About in Dark, Light, and actual Windows Contrast Theme where available. Verify semantic tokens, system colors, borders, logo legibility, disabled state, hover, and focus indicators.
- Inspect keyboard and accessibility behavior with Windows accessibility tooling where available: window titles, roles, names, expanded/disabled state, reading order, initial focus, focus return, and duplicate-window absence.
- Preserve reduced-motion behavior. Menu appearance may respond directly to user action but must not depend on decorative entrance animation; native windows require no custom animation.
- Verify caller and capability restrictions in the running application: only the intended Drop Shelf caller can request its Clear confirmation or utility navigation, About and Clear labels receive only required permissions, and unrelated surfaces cannot invoke privileged Shelf or window operations.
- Run existing actual Windows Clear, drag-out, dismissal, movement, resize, Settings, tray, startup, Cooperative Shutdown, persistence, and source-safety regressions to detect integration breakage beyond the new surfaces.
- Keep version metadata at `0.0.14` until feature implementation and actual-surface acceptance pass. Then synchronize all canonical version sources to `0.0.15`, build one exact Windows x64 NSIS candidate, and record installer and installed-executable sizes and SHA-256 hashes.
- Exercise the exact installed candidate with embedded frontend assets and the development server stopped. Cover Clear, More, Settings, About from both entry points, Source opening, resident activation, Edge Rail preservation, Cooperative Shutdown, same-candidate reinstall/update, uninstall, and Profile Data preservation.
- Record unavailable physical DPI, mixed-DPI, multi-monitor, Contrast Theme, browser-opener, or accessibility-tool coverage as unobserved. Do not infer it from source, emulation, or another environment.
- Add no permanent screenshot snapshots, CSS-class assertions, source-text assertions, icon-path assertions, mock-echo tests, command-forwarding tests, or tests that merely prove a button exists or does not throw.
- A good permanent test must catch a plausible consumer-visible bug: duplicate windows, stale Clear acceptance, unauthorized caller, wrong retained items, failed rollback, incorrect runtime metadata, wrong monitor selection, broken focus/modal state, or changed resident lifecycle.

## Out of Scope

- Implementing Check for updates, installing or configuring a Tauri updater, adding signing or release endpoints, polling for releases, background checks, or update notifications.
- Adding a public product Website before a canonical product-site URL exists, populating Cargo homepage, or labeling the GitHub repository as Website.
- Adding About content beyond logo, product name, runtime version, Author, Source, disabled Check for updates, and Coming soon.
- Adding description, license, copyright, acknowledgements, release notes, changelog, support, donation, telemetry, analytics, or diagnostics to About.
- Moving the Edge Rail Clear Temporary Items confirmation into the dedicated native confirmation window.
- Adding the Drop Shelf Utility Menu, Settings, or About controls to the Edge Rail or Rail Handle.
- Redesigning the existing Settings surface, changing Save/Cancel semantics, adding settings fields, or changing startup, hotkey, Shake-to-open, or Edge Rail placement preferences.
- Changing Clear Temporary Items scope, Temporary/Pinned lifecycle, atomic persistence behavior, batch ordering, source-file safety, or failure rollback.
- Adding source filesystem deletion, Move semantics, recycle-bin integration, secure deletion, or confirmation for unrelated removal actions.
- Adding another frontend router, application entry point, state-management framework, component framework, modal framework, menu framework, or generalized auxiliary-window manager.
- Adding a new icon dependency, replacing the established icon language, redesigning the Drop Shelf, restoring a title/subtitle, or changing unrelated card, status, scrollbar, resize, movement, or empty-state presentation.
- Persisting Clear or About window positions, adding placement settings, remembering utility-menu state, or changing canonical Drop Shelf or Edge Rail placement ownership.
- Allowing multiple About or Clear confirmation windows.
- Showing or activating the Drop Shelf when About is opened from the notification area.
- Changing left-click/double-click tray behavior, Exit DropCove semantics, Cooperative Shutdown, installer lifecycle, or Profile Data retention.
- Adding macOS, Linux, x86, ARM64, mobile, or non-MSVC behavior. The target remains Windows x64.
- Bumping version metadata before implementation and actual-surface acceptance.
- Changing 0.0.14 scope, evidence, artifacts, acceptance requirements, or commit status.
- Creating a Git commit without separate explicit user authorization.

## Further Notes

- The user approved the complete design tree on 2026-10-16: dedicated native Drop Shelf Clear confirmation, Drop Shelf-only scope, in-shelf Utility Menu, singleton About Surface, notification-area About reuse, compact visual direction, Cargo-owned author/source metadata, runtime version sourcing, and the 0.0.15 roadmap placement.
- The user approved one primary actual-Windows testing seam on 2026-10-16. Supporting builds and native regressions provide diagnostics but do not replace direct observation of the real WebView2/native/tray behavior.
- **Drop Shelf Utility Menu**, **Settings**, **About Surface**, and **Temporary Item** are canonical glossary terms. Do not substitute Settings menu, About page/modal/popup, temporary file, or main window.
- Cargo package metadata owns `DropCove Contributors` and the canonical Source repository. Homepage remains intentionally absent.
- The existing opener integration is sufficient for Source. No dependency is needed.
- The native Clear confirmation decision is documented here rather than in an ADR because it does not introduce a durable state owner, persistence model, or hard-to-reverse platform contract. Its reason must remain visible: an in-WebView dialog or teleport cannot escape the compact native Drop Shelf viewport.
- ADR-0002 remains authoritative. This feature does not change Settings-owned Edge Rail preference, Shelf-owned remembered dismissal monitor, or Open Shelf monitor behavior.
- All four tickets are resolved. The user confirmed all remaining acceptance checks passed and explicitly authorized committing the changes; prior agent-observed evidence and its limitations remain unchanged.

## Delivery Evidence — 2026-10-16

- Exact candidate: `build/DropCove-Setup-0.0.15.exe`, 3,289,290 bytes, SHA-256 `5ddf9b2676576d8e25ebb79918d9f9fa3de243f51f3fee5586ce9b16bdec0a78`. Exact installed executable: 13,870,592 bytes, SHA-256 `909b2b4f651c815985cd866fd49df2820a8e436c5bb63d936ff2aecbe6f66bc5`.
- Full Windows native regression passed 111/111; frontend production build and cargo-xwin/NSIS packaging passed. Standards review and final Spec review have no remaining actionable finding. The installed outside-pointer focus defect was fixed, rebuilt, reinstalled, and checked on empty/populated Shelves without suppressing outside Pin All activation.
- Actual Windows smoke proved independent Clear layout, cancellation, atomic rollback, stale-state handling, Pinned retention, owner modal guards, About/Settings singleton reuse, native tray About with hidden Shelf, runtime 0.0.15 metadata, Source browser opening, two-monitor centering at scale 1, embedded assets, modal Cooperative Shutdown, restart, same-candidate reinstall, and completed uninstall with Profile Data preservation.
- The first baseline launch changed HKCU Run before its original value was captured. Later cleanup restored the captured post-reconciliation installed-app registration, installer metadata, and shortcut bytes, and proved ambient profile bytes unchanged since the safety snapshot; original pre-baseline Run bytes are not claimed restored.
- Injected physical OLE Escape did not prove cancellation: Temporary references were consumed in those attempts. Accepted Copy and Pinned retention were proven against an owned Windows Forms destination; native cancellation-state regressions passed. Physical Escape/rejection was deferred to human acceptance, which the user has now confirmed passed.
- Literal Vite-server shutdown was not performed; localhost was unavailable to the owned installed WebView while embedded assets loaded. Physical high/mixed-DPI, physical Windows Contrast Theme, screen-reader output, browser-launch failure injection, and physical fullscreen/Edge Rail combinations remain unobserved. Full evidence is in Ticket 04. All owned smoke resources were removed; the user subsequently confirmed all acceptance checks passed and authorized the commit.

## Final human acceptance and commit authorization

The user confirmed `tất cả đã đạt, hãy commit các thay đổi` on 2026-10-16. All remaining acceptance gates are closed by user-reported acceptance, and committing the changes is explicitly authorized. Historical agent observations and limitations above are retained rather than reclassified as additional agent verification.
