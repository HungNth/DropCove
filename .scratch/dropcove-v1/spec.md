# DropCove V1.0 Specification

Status: ready-for-agent

## Problem Statement

Windows users frequently need to carry files and folders between applications, windows, monitors, and stages of a workflow. The normal workflow requires keeping File Explorer windows open, repeatedly navigating back to source folders, or temporarily copying files somewhere else. Those workarounds lose the context of which files were gathered together, consume screen space, and make repeated reuse cumbersome.

Users need a fast, lightweight, Windows-native place to hold references to files and folders without moving, copying, or owning those files. It must interoperate with native and third-party Windows drag-and-drop targets, remain available without stealing focus unnecessarily, preserve state across restarts, and consume almost no CPU while idle.

## Solution

DropCove provides a temporary drag-and-drop workspace for Windows. Each accepted drop creates an ordered Shelf Batch containing Shelf Items for the unique filesystem paths in that operation. DropCove stores Path References only; it never copies, moves, renames, overwrites, or deletes the referenced filesystem objects.

Users summon a floating Drop Shelf with a configurable global hotkey, drag individual items or entire batches into compatible applications, pin reusable items, remove references manually, and restore both temporary and pinned content after restart. Later V1 stages add an Edge Rail and shake-to-open while preserving the same reference and lifecycle semantics.

The first usable vertical slice proves the unpackaged EXE installation path, native drag-in/out, batching, SQLite persistence, hotkey, tray lifecycle, and the application behavior seam. Edge Rail, shake-to-open, Compact/Expanded modes, Composition polish, and later MSIX/Microsoft Store submission are layered on only after the native workflow is stable.

## User Stories

1. As a Windows user, I want to temporarily hold files and folders, so that I can continue working without keeping their source folder visible.
2. As a Windows user, I want DropCove to hold references instead of copies, so that adding large files is immediate and does not duplicate disk usage.
3. As a Windows user, I want DropCove never to move, rename, overwrite, or delete my source files, so that using the workspace cannot damage my filesystem.
4. As a Windows user, I want to summon the Drop Shelf with `Ctrl + Shift + Space`, so that it is available immediately from any normal desktop workflow.
5. As a Windows user, I want to configure the global hotkey, so that it does not conflict with my other applications.
6. As a Windows user, I want a visible warning when the configured hotkey cannot be registered, so that the primary invocation method never fails silently.
7. As a Windows user, I want DropCove to continue running when hotkey registration fails, so that I can repair the setting through the tray.
8. As a keyboard user, I want hotkey invocation to activate the Drop Shelf, so that I can interact with it immediately.
9. As a Windows user, I want the hotkey to toggle the shelf, so that the same action can show or dismiss it.
10. As a Windows user, I want `Esc` to dismiss the shelf, so that closing it is fast and familiar.
11. As a Windows user, I want the close button to dismiss rather than terminate DropCove, so that the hotkey remains available.
12. As a Windows user, I want an explicit tray command to exit DropCove, so that I can terminate the background process deliberately.
13. As a Windows user, I want DropCove to start with Windows by default, so that the global hotkey is ready after login.
14. As a Windows user, I want to disable Start with Windows, so that I retain control over startup applications.
15. As a Windows user, I want auto-start to restore data without showing or focusing the shelf, so that login is not interrupted.
16. As a Windows user, I want a second DropCove launch to activate the existing instance, so that I never get competing shelves, hotkeys, or databases.
17. As a multi-monitor user, I want hotkey invocation to show the shelf on the monitor containing the cursor, so that the shelf appears where I am working.
18. As a multi-monitor user, I want the floating shelf to use the current monitor work area and DPI, so that it is correctly placed and sized.
19. As a Windows user, I want the visible shelf to remain topmost, so that it stays reachable during cross-application work.
20. As a Windows user, I want the shelf to remain visible when another application receives focus, so that I can prepare a destination before dragging out.
21. As a Windows user, I want to drag files from Explorer or the desktop into DropCove, so that I can collect them without browsing again later.
22. As a Windows user, I want to drag folders into DropCove, so that folder workflows are supported alongside files.
23. As a Windows user, I want one accepted drop operation to create one Shelf Batch, so that I can remember which items were gathered together.
24. As a Windows user, I want items within a Shelf Batch to preserve source order, so that the batch reflects the original operation.
25. As a Windows user, I want duplicate paths within one incoming drop to be deduplicated, so that one accidental duplicate does not create redundant items in the same batch.
26. As a Windows user, I want the same path dropped in a later operation to create a new Shelf Item in a new Shelf Batch, so that separate workflow contexts remain distinct.
27. As a Windows user, I want new Shelf Batches shown newest first, so that recent work is immediately accessible.
28. As a Windows user, I want DropCove to accept local-drive paths, so that normal filesystem workflows work.
29. As a Windows user, I want DropCove to accept removable-drive paths, so that I can stage files from external media.
30. As a Windows user, I want DropCove to accept UNC and network paths, so that shared-drive workflows work.
31. As a Windows user, I want DropCove to accept cloud-backed items that provide filesystem paths, so that synced folders work like other files.
32. As a Windows user, I want stream-only virtual items rejected rather than materialized, so that DropCove remains reference-only.
33. As a Windows user, I want supported paths accepted from a mixed payload, so that one unsupported item does not discard valid files.
34. As a Windows user, I want to know how many incoming items were skipped, so that partial acceptance is never silent.
35. As a Windows user, I want a payload with no supported paths rejected, so that DropCove does not create empty batches.
36. As a Windows user, I want a Shelf Item to refer to whatever currently exists at its path, so that behavior remains simple and path-based.
37. As a Windows user, I understand that replacing a file at the same path changes what the Path Reference resolves to, so that DropCove does not pretend to track historical file identity.
38. As a Windows user, I want every newly added Shelf Item to be temporary by default, so that consumed references clean themselves up.
39. As a Windows user, I want to pin a Shelf Item, so that it remains available for repeated reuse.
40. As a Windows user, I want to unpin a Shelf Item, so that its next accepted drag-out can consume it.
41. As a Windows user, I want pinned status shown clearly, so that reusable items are distinguishable from temporary items.
42. As a Windows user, I want to remove one Shelf Item manually, so that I can clean the shelf without touching the source file.
43. As a Windows user, I want to remove an entire Shelf Batch, so that I can discard one workflow context at once.
44. As a Windows user, I want to clear all temporary items while retaining pinned items, so that reusable references survive cleanup.
45. As a Windows user, I want confirmation before bulk removal, so that I do not accidentally discard many references.
46. As a Windows user, I want empty Shelf Batches removed automatically, so that the shelf never shows meaningless containers.
47. As a Windows user, I want temporary and pinned items restored after application restart, so that a crash or restart does not lose my workspace.
48. As a Windows user, I want temporary and pinned items restored after Windows restart, so that a Windows update does not clear the workspace.
49. As a Windows user, I want every completed state change persisted immediately, so that abrupt process termination loses as little state as possible.
50. As a Windows user, I want DropCove to retain only metadata and paths, so that it never becomes a hidden file store.
51. As a Windows user, I want DropCove not to persist thumbnail bytes, so that derived images do not become a second content store.
52. As a Windows user, I want corrupt database data preserved as a timestamped backup, so that DropCove does not silently overwrite recoverable state.
53. As a Windows user, I want a clear notification when the database is reset after an open or corruption failure, so that an empty shelf is explainable.
54. As a Windows user, I want to drag one Shelf Item out, so that I can place a specific file or folder in another application.
55. As a Windows user, I want to drag an entire Shelf Batch out in one operation, so that grouped files remain easy to transfer together.
56. As a Windows user, I want native multi-file drag behavior, so that compatible Windows applications receive the batch normally.
57. As a Windows user, I want DropCove to offer copy semantics only, so that a drag-out cannot move my source files.
58. As a Windows user, I want a temporary reference removed only after the destination accepts the drag, so that canceled or rejected operations do not consume it.
59. As a Windows user, I want pinned references retained after an accepted drag, so that reusable assets remain available.
60. As a Windows user, I want an accepted mixed batch to retain pinned items and consume temporary items, so that lifecycle is evaluated per Shelf Item.
61. As a Windows user, I want canceled drag operations to leave references unchanged, so that pressing `Esc` is safe.
62. As a Windows user, I want rejected or failed drag operations to leave references unchanged, so that application incompatibility does not lose state.
63. As a Windows user, I accept that a temporary reference may reappear if DropCove crashes after the destination accepts a drop but before persistence commits, so that DropCove favors retaining a reference over losing one incorrectly.
64. As a Windows user, I want V1 drag-out limited to one item or one whole batch, so that native interoperability is stabilized before arbitrary cross-batch selection.
65. As a Windows user, I want a path that is confirmed absent marked Missing, so that deleted or moved files do not crash DropCove.
66. As a Windows user, I want a temporarily inaccessible path marked Unavailable rather than Missing, so that disconnected drives and networks are described accurately.
67. As a Windows user, I want Missing references detected during restore, shelf display, and pre-drag validation, so that stale paths are caught at useful event boundaries.
68. As a Windows user, I want a drag attempt to clean up confirmed-Missing references, so that invalid entries do not persist indefinitely.
69. As a Windows user, I want Missing cleanup to remove only the Shelf Item reference, so that no filesystem object is deleted.
70. As a Windows user, I accept that Missing cleanup is not rolled back when the subsequent drag is canceled, so that invalid references remain removed.
71. As a Windows user, I want an empty batch removed when all of its references are cleaned up as Missing, so that the shelf remains coherent.
72. As a Windows user, I want Unavailable references retained, so that temporarily disconnected content can recover later.
73. As a Windows user, I want a warning before dragging only the available portion of a batch, so that partial batch output is explicit.
74. As a Windows user, I want available items dragged after I confirm a partial operation, so that one unavailable network item does not block the rest forever.
75. As a Windows user, I want image Shelf Items to show Windows-provided thumbnails when available, so that visual assets are recognizable.
76. As a Windows user, I want thumbnail failure to fall back to the native file icon, so that an item always has a usable visual.
77. As a Windows user, I want non-image files and folders to use Windows icons, so that visuals match the operating system.
78. As a Windows user, I want thumbnails loaded lazily, so that large shelves do not block interaction.
79. As a Windows user, I want a unified floating shelf in the first usable release slice, so that native workflows can be proven before adding multiple presentation modes.
80. As a Windows user, I want the Edge Rail added after the floating shelf workflow is stable, so that docking does not hide core drag defects.
81. As a Windows user, I want the Edge Rail to appear when I dismiss a non-empty shelf, so that held content remains reachable.
82. As a Windows user, I want the shelf to become Hidden when dismissed while empty, so that DropCove consumes no visible screen space unnecessarily.
83. As a Windows user, I want the pre-Edge-Rail vertical slice to hide a non-empty shelf without losing its content, so that staging does not require impossible display transitions.
84. As a Windows user, I want the Edge Rail to overlay the screen rather than reserve desktop work area, so that it does not resize or reposition other windows.
85. As a Windows user, I want the Edge Rail docked to the monitor where the shelf was dismissed, so that it remains in the active workflow context.
86. As a Windows user, I want the default rail edge to be Right, so that the initial behavior is predictable.
87. As a Windows user, I want to configure Left or Right docking and the target monitor, so that the rail fits my setup.
88. As a Windows user, I want the rail to stay on its selected monitor rather than follow the cursor, so that it does not jump unexpectedly.
89. As a Windows user, I want the collapsed rail to remain directly draggable, so that common transfers do not require opening the full shelf.
90. As a Windows user, I want the rail to expand after a short hover, so that details are available without accidental expansion.
91. As a Windows user, I want drag-enter expansion after a short delay, so that dropping new files into the rail is practical without flicker.
92. As a Windows user, I want delayed collapse after leaving the rail, so that small pointer movements do not close it immediately.
93. As a Windows user, I want rail hover and direct drag not to activate DropCove, so that my current application keeps focus.
94. As a Windows user, I want opening the full shelf from the rail to activate DropCove, so that keyboard management works.
95. As a Windows user, I want batch flyouts in the rail, so that I can drag one item from a multi-item Shelf Batch.
96. As a Windows user, I want vertical rail scrolling, so that the rail does not grow indefinitely.
97. As a Windows user, I want the rail hidden over fullscreen applications by default, so that videos, presentations, and games are not covered.
98. As a power user, I want a setting that allows the rail over fullscreen applications, so that I can keep it permanently visible if desired.
99. As a Windows user, I want fullscreen detection to be event-driven, so that the rail does not require background polling.
100. As a Windows user, I want shake-to-open added only after Edge Rail is stable, so that the global input feature is isolated from earlier risks.
101. As a Windows user, I want shake-to-open enabled by default, so that the defining gesture is immediately available.
102. As a Windows user, I want to disable shake-to-open, so that the global mouse hook is removed when I do not want the feature.
103. As a Windows user, I want shake sensitivity configurable, so that the gesture can fit my mouse and movement style.
104. As a Windows user, I want a shake while dragging to show the shelf near the cursor, so that I can deposit files without traveling to a fixed location.
105. As a Windows user, I want canceled or unsupported shake workflows to restore the previous shelf state, so that false positives do not permanently change the UI.
106. As a Windows user, I want cooldown after a shake trigger, so that one gesture does not repeatedly summon the shelf.
107. As a Windows user, I want an accepted shake drop to keep the shelf near the cursor, so that I can inspect and manage the new Shelf Batch.
108. As a privacy-conscious user, I want shake detection to inspect only minimal mouse input state, so that the feature does not behave like broad input monitoring.
109. As a performance-conscious user, I want no continuous cursor polling, so that idle CPU remains negligible.
110. As a Windows user, I want distinct Compact and Expanded modes in final V1, so that quick access and management remain separate concerns.
111. As a Windows user, I want responsive Composition animations only after behavior is stable, so that polish never blocks core interaction.
112. As a user with reduced-motion settings, I want DropCove to respect Windows animation preferences, so that the interface remains accessible.
113. As a Windows user, I want DropCove responsive with at least 100 Shelf Batches and 1,000 Shelf Items, so that normal temporary-workspace growth remains smooth.
114. As a performance-conscious user, I want idle CPU to average at most 0.1% on the release build, so that the resident utility is effectively idle.
115. As a Windows user, I want the resident process to show the shelf within a 150 ms p95 target, so that invocation feels immediate.
116. As a Windows user, I want an idle working-set target below 150 MB, so that the utility remains lightweight for its WinUI stack.
117. As a Windows user, I want thumbnail and database work kept off the UI thread, so that drag and pointer interactions remain responsive.
118. As a Windows user, I want drag-in compatibility with Explorer and Desktop, so that the primary sources work before V1 release.
119. As a Windows user, I want drag-out compatibility with Explorer, browser file inputs, VS Code, and at least one common chat application, so that representative destination types are proven.
120. As a Windows user, I want folder drag-out verified with Explorer, so that folder references are not nominal-only.
121. As a Windows user, I want failed cross-integrity drag/drop to retain temporary references, so that Windows security restrictions cannot consume content.
122. As a security-conscious user, I want DropCove to run unelevated without `uiAccess`, so that the utility does not expand its privilege unnecessarily.
123. As a Windows 11 user, I want an x64 self-contained application installed by an EXE, so that I can use DropCove before signing or Store publication.
124. As a Windows user, I want the installer not to require a separately installed .NET runtime, so that setup works on a clean machine.
125. As a Windows user, I want the EXE installer to install and uninstall DropCove cleanly, so that the first build behaves like a normal desktop application.
126. As a maintainer, I want the installer built from the CLI without Visual Studio tooling, so that local development and CI remain license-independent.
127. As a privacy-conscious user, I want no telemetry, background update checks, or application-owned network traffic, so that DropCove remains local and predictable.
128. As a future Microsoft Store user, I want Store publication deferred until the application is stable and Partner Center identity is available, so that signing does not block early delivery.
129. As a maintainer, I want deterministic behavior tests at the application seam, so that lifecycle regressions are caught without testing private implementation details.
130. As a maintainer, I want native interoperability verified against the installed EXE application, so that mocks cannot falsely prove OLE, focus, DPI, hook, or window behavior.

## Implementation Decisions

- The product is implemented in C# on .NET 10 with WinUI 3, Windows App SDK, XAML, CommunityToolkit.Mvvm, CsWin32, WinRT, Win32, OLE, Microsoft.UI.Composition, and SQLite.
- The initial product targets supported Windows 11 releases and x64. ARM64 is deferred until native integration is verified there.
- The solution starts with four product modules: application/UI, core shelf and settings domain, services, and native interop.
- CommunityToolkit.Mvvm is used for observable UI state and commands. Domain types and services remain concrete unless a second implementation or real replaceable boundary appears.
- `DropShelfManager` is the application behavior boundary for Shelf Batch state, Shelf Item lifecycle, display state, placement, and persisted state transitions.
- `DragDropService` owns native drag-in/out behavior. Views and view models do not contain OLE-specific logic.
- A Shelf Batch represents one accepted incoming drop operation and preserves source order.
- Paths are deduplicated within one incoming drop operation only. The same path may appear independently in later Shelf Batches.
- Shelf Item identity is per occurrence in a Shelf Batch, not global filesystem-path identity.
- A Path Reference resolves by current path. DropCove does not track filesystem object identity across replacement or rename.
- DropCove accepts local, removable, UNC/network, and cloud-backed filesystem paths. Stream-only virtual items are rejected and never materialized.
- Mixed incoming payloads accept supported paths and report the unsupported count. Empty accepted results do not create Shelf Batches.
- DropCove never copies source contents into application storage and never moves, renames, overwrites, or deletes source filesystem objects.
- New Shelf Items are Temporary Items. Pinning converts an item to a Pinned Item; unpinning returns it to temporary lifecycle.
- Manual removal includes Remove Item, Remove Batch, and Clear Temporary Items. Bulk actions require confirmation and remove references only.
- Drag-out offers the OLE `Copy` effect only.
- A Successful Drag-Out is the destination accepting the operation with a non-empty Copy result. DropCove does not claim that the destination completes later internal processing.
- Successful drag-out removes participating Temporary Items and retains participating Pinned Items. Cancel, rejection, and failure leave participating references unchanged.
- V1 drag-out supports one item or one entire Shelf Batch. Arbitrary multi-selection across batches is deferred.
- The unavoidable crash window after destination acceptance and before local commit uses at-least-once semantics: a temporary reference may reappear after restart.
- Missing Item means the path is confirmed absent. Unavailable Item means the path may still exist but is inaccessible because of volume, network, cloud, or permission state.
- Availability validation occurs during restore, when the shelf is shown, and immediately before drag-out. V1 does not use a realtime filesystem watcher.
- Starting a drag removes confirmed-Missing references before native drag begins and removes an empty Shelf Batch. This cleanup is not rolled back if the later drag is canceled.
- Unavailable references are retained. A partial batch drag requires explicit confirmation and includes only available items.
- SQLite persistence is mandatory in Stage 1 and is per user.
- Every completed state mutation is committed transactionally: drop-in, pin/unpin, manual removal, Missing cleanup, and successful drag-out.
- Persisted state contains batch order and creation time plus item identity, path, type, and pinned state. It excludes source bytes and thumbnail bytes.
- Database open/corruption failure preserves a timestamped backup, informs the user, and creates a new database rather than overwriting the failed file.
- Image thumbnails use Windows Shell thumbnail/cache behavior and are retained only as needed in process memory. Other files and folders use Windows icons; thumbnail failure falls back to the icon.
- DropCove is single-instance. Later launches redirect activation to the running instance.
- Start with Windows is enabled by default and configurable. Auto-start restores content while leaving the UI hidden and unfocused.
- The global hotkey defaults to `Ctrl + Shift + Space`, is configurable, activates the floating shelf, and reports registration conflicts through the tray.
- The close button, `Esc`, and hotkey toggle dismiss the shelf. The process exits only through an explicit tray command.
- The floating shelf remains topmost and does not auto-hide merely because another application receives focus.
- Before Edge Rail exists, Hidden may contain items. After Edge Rail exists, dismissing an empty shelf enters Hidden and dismissing a non-empty shelf enters EdgeDocked.
- Hotkey placement uses the default work-area position on the monitor containing the cursor, with correct current-monitor DPI placement and sizing.
- Edge Rail is an overlay rather than a Windows AppBar and does not reserve desktop work area.
- Edge Rail docks by default to the Right side of the monitor where the shelf was dismissed, remembers monitor and side, and allows Left/Right and monitor configuration.
- Edge Rail hover/direct drag does not activate DropCove. Opening the full shelf activates it.
- Edge Rail expands after 200 ms hover or drag-enter and collapses after 300 ms pointer/drag leave unless a flyout remains open.
- Edge Rail is hidden over fullscreen applications by default. The user can override this; detection is event-driven.
- Shake-to-open is implemented after Edge Rail and is enabled by default with a disable setting and configurable sensitivity.
- While shake is enabled, a low-level global mouse hook remains installed. The callback performs minimal work and queues analysis elsewhere; cursor polling is forbidden.
- Shake is a heuristic because Windows exposes no documented global external OLE file-drag start event. False positives use cooldown and previous-state restoration.
- An accepted shake drop keeps the shelf near the cursor; cancellation or unsupported content restores the previous display and placement state.
- Distinct Compact and Expanded shelf modes and Composition animations are final V1 polish after native workflows are stable.
- Animations respect Windows reduced-motion and animation settings.
- DropCove runs unelevated and guarantees drag/drop only between applications at the same integrity level. V1 does not use `uiAccess` or an elevated helper.
- The initial release artifact is an unpackaged x64 self-contained WinUI publish installed by a CLI-built NSIS EXE. It has no package identity.
- The app uses `WindowsPackageType=None` and self-contained Windows App SDK deployment. The initial installer carries the complete published output and accounts for the x64 Visual C++ Redistributable prerequisite.
- NSIS is the initial installer compiler because its standalone `makensis.exe` CLI and zlib/libpng license permit commercial use without a developer-seat license. Visual Studio, VS workloads, MSVC Build Tools, standalone MSBuild, and Visual Studio XAML tooling are not required.
- MSIX/Microsoft Store delivery is deferred until the application is stable and Partner Center supplies its package name and publisher. The development manifest identity is not a production identity.
- DropCove has no telemetry, background update checker, or application-owned network traffic. The EXE installer has no built-in updater.
- Delivery is staged: unpackaged EXE vertical slice; Edge Rail; shake-to-open; Compact/Expanded modes and polish; later MSIX/Microsoft Store submission.
- Release targets are idle CPU average at or below 0.1%, zero periodic cursor polling, shelf-show latency p95 at or below 150 ms from the resident process, and idle working-set target below 150 MB.
- V1 responsiveness is validated with at least 100 Shelf Batches and 1,000 Shelf Items using lazy thumbnail loading and UI virtualization.

## Testing Decisions

- Tests assert externally visible shelf behavior and persisted outcomes rather than private methods, internal collection implementation, generated MVVM plumbing, or source text.
- The primary automated seam is `DropShelfManager` operating with a real temporary SQLite database.
- Automated tests submit accepted incoming path sets as drop operations, apply user actions and native drag outcomes at the manager boundary, and assert Shelf Batch/Shelf Item state plus restored persisted state.
- No new interface is introduced solely for testing. Native outcomes are represented at the application boundary; native APIs themselves are not replaced by a large mock hierarchy.
- Automated behavior coverage includes within-drop deduplication, cross-drop duplicate independence, source order, newest-first batch order, temporary/pinned transitions, mixed-batch consumption, manual removal, empty-batch cleanup, and bulk-clear confirmation outcomes.
- Persistence coverage includes restart restoration of temporary and pinned items, immediate committed mutations, Missing cleanup durability, canceled drag behavior, at-least-once crash recovery expectations, and corrupt-database backup/reset behavior.
- Availability coverage distinguishes Missing and Unavailable, validates cleanup rules, and verifies confirmed partial drag behavior without deleting filesystem objects.
- Native behavior is verified through actual installed-EXE smoke scenarios rather than mock-only tests.
- Installed-EXE smoke scenarios cover Explorer/Desktop drag-in, one-item and whole-batch drag-out to Explorer, folder drag-out to Explorer, browser file input, VS Code, and at least one common chat application.
- Installed-EXE smoke scenarios cover accepted, canceled, rejected, unsupported, and same-integrity failure outcomes. Temporary references for valid participating items remain unless the destination accepts the drag; confirmed-Missing references may already have been removed during pre-drag cleanup and are not restored if the later drag is canceled.
- Installed-EXE smoke scenarios verify the cross-integrity limitation in both directions where practical and confirm failed operations preserve references.
- Window smoke scenarios cover single-instance redirection, global hotkey registration/conflict, activation, topmost behavior, dismiss behavior, tray Exit, and hidden auto-start restore.
- Monitor smoke scenarios cover current-monitor hotkey placement, per-monitor DPI, remembered rail monitor/side, and fullscreen rail behavior.
- Edge Rail smoke scenarios cover no-activate hover/direct drag, 200 ms expansion, 300 ms collapse, flyout suppression of collapse, scrolling, direct item drag, whole-batch drag, and drag-in.
- Shake smoke scenarios cover hook enable/disable, cooldown, supported drop, unsupported payload, cancellation, false-positive restoration, cursor-monitor placement, and accepted-drop state retention.
- Performance verification runs against installed EXE release builds and measures idle CPU, periodic activity, summon latency, working set, thumbnail responsiveness, SQLite responsiveness, and the 100-batch/1,000-item target.
- Accessibility verification checks keyboard access, high-DPI behavior, reduced-motion compliance, and final Compact/Expanded management interactions.
- Installer verification checks clean x64 EXE installation, launch, startup registration, uninstall, self-contained .NET and Windows App SDK deployment, and the x64 Visual C++ Redistributable prerequisite. Microsoft Store submission is a later track.
- The repository initially contained design documentation but no application code or prior test suite. The application-seam tests and installed-EXE smoke matrix establish the initial convention.

## Out of Scope

- Clipboard history.
- Text snippets.
- URL management.
- Clipboard images or clipboard quick-drop.
- Cloud synchronization.
- User accounts.
- Plugin systems.
- Multiple named shelves.
- Pinned workspaces or project-specific shelves.
- Search and filtering.
- File synchronization.
- Copying source files into DropCove-managed storage.
- File move semantics during drag-out.
- Tracking filesystem object identity across rename, replacement, or volume changes.
- Materializing virtual stream-only drag items.
- Realtime filesystem watching in V1.
- Arbitrary multi-selection across batches in the first vertical slice.
- Opening files or folders directly from the shelf in the first vertical slice.
- Top or bottom Edge Rail docking.
- Reserving desktop work area through Windows AppBar behavior.
- Guaranteed drag/drop interoperability with elevated applications.
- `uiAccess`, running DropCove elevated by default, or an elevated helper process.
- MSIX/Microsoft Store submission and production signing before the EXE installer proves a working application and Partner Center identity is available.
- ARM64 and x86 release artifacts in the initial release.
- In-application update checking or background updating.
- Telemetry or automatic crash upload.
- Full UI automation infrastructure before native behavior is stable.

## Further Notes

- The domain vocabulary is authoritative: Drop Shelf, Edge Rail, Shelf Batch, Shelf Item, Path Reference, Temporary Item, Pinned Item, Missing Item, Unavailable Item, and Successful Drag-Out must retain their defined meanings.
- The initial unpackaged self-contained EXE decision is architectural: it permits CLI-only development and a working app before signing or Microsoft Store identity exists.
- The global mouse hook decision is architectural: Windows does not provide a documented global event for external OLE drag start, so shake-to-open is necessarily heuristic. Polling remains prohibited.
- Native interoperability is the dominant technical risk. UI polish, additional presentation modes, animation, and MSIX/Microsoft Store submission must not precede proof of actual Explorer-to-DropCove-to-Explorer behavior in the installed EXE build.
- Microsoft Store submission is deferred until the EXE installer works and Partner Center supplies the package identity and publisher.
- An accepted OLE drop is the strongest source-side completion signal available. DropCove cannot guarantee or observe later internal processing by the destination application.
