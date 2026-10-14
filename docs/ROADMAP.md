# DropCove Roadmap

This file is the permanent source of truth for implementation progress. [SPECIFICATION.md](SPECIFICATION.md) defines target product behavior, and [ARCHITECTURE.md](ARCHITECTURE.md) defines the target architecture.

## Delivery rules

- Versions advance sequentially as `0.0.x`. Do not introduce `0.1.0` or `1.0.0` without explicit approval.
- Implement one version per delivery cycle, then stop for review before starting the next version.
- Each version is the smallest usable vertical slice for its workflow. Internal modules ship with the first workflow that needs them.
- A version is complete only after its Windows 11 x64 behavior is exercised, project manifests use the same version, the NSIS installer is built, and verification evidence is recorded below.
- Detailed implementation tickets belong under `.scratch/dropcove/` and are created only for the approved current version.
- Requirements deliberately excluded by the specification remain outside this roadmap.

Status values: `Completed`, `In progress`, `Planned`, or `Blocked`.

## 0.0.1 — Windows application bootstrap

**Status:** Completed — accepted bootstrap baseline.

### Scope

- Establish the Tauri v2, Vue, TypeScript, Pinia, and Rust application shell.
- Configure a borderless, transparent, always-on-top desktop window.
- Configure Windows x64 packaging through cargo-xwin and a current-user NSIS installer.

### Implemented

- Frontend bootstrap and placeholder presentation in `src/main.ts` and `src/App.vue`.
- Tauri process bootstrap in `src-tauri/src/main.rs` and `src-tauri/src/lib.rs`.
- Version `0.0.1` in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Existing `DropCove-Setup-0.0.1.exe` and Windows x64 executable artifacts.
- Build and installer decision recorded in [ADR-0001](adr/0001-cargo-xwin-and-per-user-nsis.md).

### Verification

- Existing source and build artifacts establish the accepted bootstrap baseline.
- Historical interactive smoke evidence was not recorded. This is an explicit exception for `0.0.1`; later versions must satisfy the delivery rules above.

### Deferred

- All Shelf workflows and Windows resident integrations begin in `0.0.2` or later.

## 0.0.2 — Core Drop Shelf round trip

**Status:** Completed.

### Scope

- Add the single Core Shelf State owner with Shelf Batch and Shelf Item ordering, serialized mutations, Temporary Item lifecycle, and duplicate suppression.
- Add per-user profile resolution and SQLite shelf persistence with foreign keys, WAL, transactional writes, restore, and retained sizing data needed by this slice.
- Accept Windows storage-item drops, preserve source order, filter unsupported and repeated paths, and treat exact retained batches as silent no-ops.
- Render the floating Drop Shelf and responsive batch grid with basic automatic growth, status feedback, and batch removal.
- Support copy-only item and whole-batch drag-out, availability preparation, Missing cleanup, Unavailable retention, cancellation safety, and Temporary Item consumption after accepted `Copy`.

### Implemented

- Single Core Shelf State with SQLite transactions, WAL, foreign keys, integrity checking, and per-user profile resolution.
- Single-item Drop Shelf intake, restart restore, Remove Batch, and copy-only OLE item drag-out.
- Windows OLE drag executes on the owning window thread; database operations remain outside presentation input handling.
- Multi-item Windows Shell/CF_HDROP intake, skipped-entry feedback, source-order retention, and ordinal case-insensitive exact-batch duplicate suppression.
- Responsive grid, Automatic Shelf Growth, pending empty-shelf width intent, and persisted non-empty shelf width.
- Availability classification at restore/drag boundaries, durable Missing cleanup, retained Unavailable references, and token-scoped whole-batch Copy completion.
- Accessible mixed-availability modal with cancellation and native drag from the available-items control.

- Windows x64 current-user NSIS packaging (`DropCove-Setup-0.0.2.exe`) with version synchronization across manifests and registry.
### Verification

Completed checks for ticket 01:

- Five native Windows core checks passed: restore, accepted-Copy consumption, canceled-drag retention, persistence-failure retention, and refusal of an invalid database without modifying its bytes.
- `pnpm build` and `cargo xwin build --target x86_64-pc-windows-msvc` succeeded.
- Real Windows smoke accepted a native file drop, restored it after process restart, transferred it to a same-integrity native Copy target, retained it after cancellation outside eligible targets, and removed a folder reference without deleting its source.
- Native core regression suite: 11 tests passed in 0.11s on Windows x64.
- `pnpm build`, `cargo xwin build --release`, and `pnpm tauri build` completed cleanly.
- Full parent specification demo path and release acceptance (sections 59–65) exercised through the installed artifact (`C:\Users\Hung\AppData\Local\DropCove\DropCove.exe`):
  1. Accepted files and folders into an ordered Shelf Batch.
  2. Exact path set in another order was a silent no-op.
  3. Restart restored batch order, source-order items, and sizing.
  4. Missing cleanup removed confirmed-absent file before drag.
  5. Modal confirmation displayed when unavailable items remained.
  6. Destination-accepted Copy consumed only the Available Temporary participant; Unavailable reference remained in the batch.
  7. Silent uninstallation verified; installed binary removed from target path.
### Deferred

- Pinning, Manage Items, full resize intent, resident entry points, Edge Rail, Shake-to-open, shell visuals, and recovery hardening.

## 0.0.3 — Shelf lifecycle and management

**Status:** Completed — tickets 01–06 resolved; packaged and installed 0.0.3 verified. Later versions remain unstarted.

Plan: [Specification](../.scratch/shelf-lifecycle-management/spec.md) · [Delivery map and tickets](../.scratch/shelf-lifecycle-management/map.md).

### Scope

- Add Pin/Unpin, tri-state Bulk Pinning, Remove Item, and confirmed Clear Temporary Items behavior; preserve existing Remove Batch as regression coverage.
- Add Manage Items for multi-item batches, including direct item drag-out, availability labels, bounded placement, lifecycle closure rules, and focus restoration.
- Complete single-item and multi-item card actions and the pinned-versus-temporary drag completion behavior.
- Complete responsive sizing: preferred width, Automatic Shelf Growth, Manual Height Override, resize hit testing, live-growth rules, and final-item reset.
- Add keyboard traversal, accessible names, tooltips for full paths, and `Esc` ordering for the controls introduced by this version.

### Implemented

- Single-item Pin/Unpin: serialized core transaction, async IPC, and accessible card action with visible pinned state.
- Pin state survives restart; native accepted Copy retains Pinned references and consumes Temporary references after Unpin. Failed persistence leaves prior state intact.
- Reused the existing `is_pinned` schema and Temporary-consumption predicate from 0.0.2; user-visible pin behavior and pinned Copy retention were not shipped or verified in 0.0.2.
- Native Manage Items: independent 280-pixel owned popup, bounded item scroller, work-area placement, pointer no-activation, real availability/path presentation, per-item Pin/Unpin/Remove/native Copy, shared-core synchronization, and owner/batch/explicit-close lifetime cleanup.
- Derived tri-state Bulk Pinning on cards and management header, atomic all-item lifecycle updates, X/N pinned summary, Enter/Space focus, canonical Tab/Shift+Tab cycling, exact-origin focus restoration, and guarded native Escape cancellation before popup dismissal.
- Confirmed Clear Temporary Items footer/native dialog: atomic cross-batch cleanup preserves Pinned references and source objects, synchronizes management, prunes empty batches, and resets final-empty sizing without partial state on failure.
- Native eight-edge/corner resizing with hover/cursor feedback, provenance-aware manual logical height, transient empty sizing intent, atomic first-batch retention, final-item reset, actual work-area clamping without preferred-intent overwrite, and management closure before user resize changes geometry.

### Verification

Ticket 01 exercised evidence:

- `pnpm build` passed; cargo-xwin compiled the Windows-targeted suite and direct native `.exe` execution passed all 14 tests. Its default Wine invocation did not execute and is not counted as a passing test run.
- Built and launched the Windows x64 executable with `tauri/custom-protocol` and an owned profile; verified pointer Pin, keyboard Unpin, process-restart restore, actual OLE Copy retention/consumption, canceled/rejected/failed transfer retention, persistence failure reporting, and pinned Remove Batch source safety.
- Verified the native 180 × 180 surface visually. Removed owned smoke fixtures/profile/WebView data and the throwaway native target script; no ambient Shelf profile or recovery data was used.
- This is development-slice proof, not installer acceptance; version metadata and the existing installer remain at 0.0.2 until ticket 06.

Ticket 02 exercised evidence:

- Final frontend build and Windows x64 embedded release build passed; direct native execution passed all 20 core/placement checks.
- Native smoke proved Pinned/Temporary row Copy, cancel/reject/path-error retention, one-item survivor popup, failure rollback, occurrence-specific removal, 280 × 480 bounded management with real availability labels, single-window replacement, explicit/batch/owner-hide closure, and native outer-frame work-area clamping.
- Corrected hidden undecorated native frame initialization before no-activation show; final non-instrumented app smoke completed both row Copies. Removed owned smoke resources, temporary script, and all diagnostic instrumentation. Installer/full-version acceptance remains ticket 06.

Ticket 03 exercised evidence:

- Final frontend and embedded Windows x64 release builds passed; direct native suite execution passed all 23 checks, including bulk rollback/availability invariance/prepared-drag pin transitions.
- Native app proved tri-state synchronization, complete forward/reverse keyboard traversal, explicit/replacement origin focus, real failed-update rollback, physical OLE Escape cancellation followed by separate dismissal, whole-batch Copy closure/retention, and owner/batch cleanup. Owned fixtures/profile/processes and temporary script were removed.
- Baseline tickets 01–02 and earlier changes were committed as requested in `0cee503`; ticket 03 implementation remains uncommitted. Installer acceptance is still ticket 06.

Ticket 04 exercised evidence:

- Frontend/embedded Windows x64 release builds passed; direct native execution passed all 27 checks, including partial-delete rollback and final sizing reset.
- Actual app proved Cancel/Escape no-op, safe default focus, failed-clear rollback on both surfaces, cross-batch mixed cleanup including Missing/Unavailable references, overlapping-path identity/source safety, surviving/disappearing management views, footer visibility, and 180 × 180 final reset. Restart restored the Pinned survivor and empty sizing state.
- Owned test profile/fixtures/WebView handles were cleaned up; no ambient profile used and no new commit created. Full installer acceptance remains ticket 06.

Ticket 05 exercised evidence:

- Frontend/embedded Windows x64 builds passed; direct native suite execution passed all 34 checks. Actual app exercised all eight native directions, width-only versus vertical/corner intent, pending/first accepted batch behavior, failed persistence rollback, manual restart, automatic live growth/reopen recalculation, and final reset.
- Programmatic/interactive full-window bounds stayed inside the observed work area without durable clamp feedback. Available smoke DPI was scale 1; live mixed-DPI/multi-monitor transitions were not separately exercised. Popup closure before movement passed on West and East; an initially obstructed East probe was explained via native hit-window inspection and rechecked unobstructed.
- Both owned smoke/probe profiles, fixtures, WebView handles and app processes were cleaned up. No new commit or installer-release acceptance; ticket 06 is the remaining release gate.

Ticket 06 release evidence:

- Version 0.0.3 synchronized across canonical/package/crate/lock/Tauri metadata, installed PE/runtime and current-user uninstall registration. Installer: [DropCove-Setup-0.0.3.exe](../build/DropCove-Setup-0.0.3.exe), 2.80 MiB, built as Windows x64 currentUser NSIS using cargo-xwin.
- Packaging frontend checks passed; newly compiled 0.0.3 native test executable passed **34/34** checks. Installed process ran at Medium integrity with embedded frontend and an owned test profile.
- Installed-app smoke proved native ordered file/folder intake and duplicate no-op; item/all-Pinned/mixed batch OLE Copy and cancellation/rejection/failure retention; Pin/Unpin, tri-state bulk and confirmation Clear; keyboard traversal/exact-origin focus; width-only/manual resize, restart restore and final reset/source safety.
- Owned test installation was uninstalled cleanly; its executable/registration, profile, fixtures, WebView/copy data and temporary script were removed. Ambient Shelf profiles/recovery data were not used. Implementation tickets 03–05 committed as requested in `e2fefcc`; release metadata/docs remain uncommitted.
- SHA-256: `2d385ca2ac09d60f7d069868131fa1fe8ea3569433aac416bb11b343380508c5`. Installer is unsigned; live mixed-DPI/multi-monitor smoke was not exercised (available scale 1). These limits are not represented as verified coverage.

### Deferred

- Resident process access, Edge Rail, Shake-to-open, advanced shell visuals, and recovery/distribution hardening.

## 0.0.4 — Resident access and settings

**Status:** Completed — tickets 01–05 resolved; Windows x64 current-user NSIS 0.0.4 built and verified from an isolated installed executable. 0.0.5+ remains unstarted.

Plan: [Specification](../.scratch/resident-access-settings/spec.md) · [Delivery map and tickets](../.scratch/resident-access-settings/map.md) · [Windows integration research](../.scratch/resident-access-settings/research/windows-resident-integration.md).

### Scope

- Add the profile-specific single-instance guard and activation signal; a secondary launch shows the existing Drop Shelf and exits.
- Add the notification-area icon, Settings and Exit commands, warning notifications, and resident lifetime independent of the Drop Shelf window.
- Add global hotkey registration and recording, atomic settings persistence, Start with Windows registration, hidden `--autostart`, and rollback-safe Save/Cancel behavior.
- Add cursor-monitor placement for normal launch, activation, tray, and hotkey; Per-Monitor V2 DPI conversion; work-area clamping; and topmost repair without activation.
- Preserve safe operation when hotkey, startup registration, or other optional integration setup fails.

### Implemented

- Ticket 01 resolved: profile-scoped lease before presentation/SQLite startup, acknowledged secondary activation, resident close/hide, transient coordinator visibility, cursor-monitor placement with complete native-frame work-area clamping. User approved warning/Exit in ticket 02 and Settings in ticket 03.
- Ticket 02 resolved: notification-area access through Tauri tray/menu, coordinator-owned warnings/Exit, event-driven visible-only topmost repair without activation, display reflow events and explicit SQLite/native integration teardown.
- Ticket 03 resolved: global hotkey and Settings, focused chord capture, atomic Save/new-hotkey activation, old-hotkey retirement, actual conflict rejection and Cancel retention. User physical Step 5 retest passed; native Enter/Space initiation and Save-disabled/Escape checks passed on the installed build.
- Ticket 04 resolved: Start with Windows On/Off, quoted profile-specific HKCU Run command, hidden startup, full settings schema and rollback-safe joint Save with actual-state reporting and retry.

### Verification

- Ticket 01 integrated suite: 44 passed, 0 failed. Actual Windows executable smoke passed same-profile startup/activation, independent profiles, close/hide and exact snapshot restart retention, 96-DPI native work-area bounds, and bounded unready-primary rejection without creating a database. Ticket 05 subsequently verified the installed 0.0.4 release on both available 96-DPI monitors; mixed-DPI hardware remains unverified.
- Ticket 02 integrated suite: 47 passed, 0 failed. Real native Shell icon/menu/Exit and owned foreground focus/z-order smoke passed; clean exit removed WAL/SHM and restart restored the exact durable item. An owned current-user installed 0.0.3 smoke artifact proved secondary activation, hidden display-event handling, Exit and actual Windows warning toast history delivery. No transient warning banner is claimed. Owned installation/registry/shortcut/profile/WebView cleanup passed; final 0.0.4 packaging and installed proof were completed in ticket 05.
- Recorder fix: fresh final-source integrated suite **55 passed, 0 failed**. Actual Windows input into the installed Settings captured Ctrl+Alt+K, Save persisted VK 75 and closed, and the new shortcut toggled Shelf; installed native Exit returned 0. Owned test profiles were removed; user profile data was untouched. Full coverage limits and exact artifact hashes are in [ticket 03](../.scratch/resident-access-settings/issues/03-global-hotkey-settings.md#evidence-and-manual-retest).
- User confirmed the repaired installed recorder test passed. Additional owned installed-app native Enter/Space checks started listening, disabled Save and allowed Escape cancellation; the owned process exited 0 and its test data was removed. Ticket 03 resolved and unblocked ticket 04, which is now also resolved.
- Ticket 04 final-source cargo-xwin Windows release build succeeded; freshly compiled native suite **66 passed, 0 failed**. Actual Windows smoke proved Run-only On/Off, exact quoted hidden launch, tray/hotkey/secondary access, nonempty Shelf/sizing restore, draft/Cancel retention, real JSON-lock rollback, startup failure warning/retry and malformed/unreadable file safety. Exact owned Run and unrelated entries were restored; owned processes/tabs/temp fixtures were cleaned. See [ticket 04 evidence and coverage limits](../.scratch/resident-access-settings/issues/04-start-with-windows-hidden-start.md#answer). Ticket 05 subsequently completed packaged release proof; actual sign-out/sign-in remains unobserved and vendor CRT PDB warnings remain nonfatal.
- Ticket 05 release: [DropCove-Setup-0.0.4.exe](../build/DropCove-Setup-0.0.4.exe), 3,141,424 bytes, SHA-256 `23ca00c1a3871a422703e8093fdbc350f16342cb98a4037d9eda97c2642d9408`. Version/package/crate/lock/Tauri, built/installed PE and owned uninstall metadata reported 0.0.4. Freshly compiled 0.0.4 suite passed **66/66**; cargo-xwin NSIS packaging included vue-tsc/Vite.
- Installed native proof: secondary activation, retained nonempty Shelf, held-key no-repeat, both 96-DPI monitor bounds, native tray Settings/Exit, foreground topmost repair without focus theft, recorder/Save/Cancel, occupied shortcut, new-key activation/old-key retirement, startup On/Off/hidden launch/retry and real post-effect JSON/Run/hotkey rollback. Missing/malformed/share-denied settings preserved source data; delivered malformed/access warnings observed in Windows toast history. See [ticket 05 evidence](../.scratch/resident-access-settings/issues/05-release-0.0.4.md#release-evidence).
- Owned installation uninstalled, processes/profile/WebView removed, Run and original 0.0.3 user uninstall/shortcuts restored with unchanged ambient executable hash. User installation intentionally stays 0.0.3; 0.0.4 installer is available separately. No commit created. Limits: no mixed-DPI hardware or actual sign-out/sign-in; warning history rather than transient banner; native menu command dispatch used after observing popup because overflow pointer automation was unreliable.

Release gates completed on available hardware. Mixed-DPI and physical sign-in remain unobserved limits, not inferred coverage; later-version work requires user direction.

### Deferred

- Non-empty dismissal transitions to the full Edge Rail workflow in `0.0.5`; Shake-to-open and final hardening remain later.

## 0.0.5 — Edge Rail workflow

**Status:** Completed — tickets 01–05 resolved; Windows x64 current-user NSIS 0.0.5 built and verified from an isolated installed executable. 0.0.6+ remains unstarted.

Plan: [Specification](../.scratch/edge-rail-workflow/spec.md) · [Delivery map and tickets](../.scratch/edge-rail-workflow/map.md). Implementation order: `01 → 02 → 03 → 04 → 05`.

### Scope

- Add `EdgeDocked` display behavior, Rail Handle, delayed expansion/collapse, bounded growth, scrolling, and immediate hiding after the final Shelf Item is removed.
- Support drop-in, item and batch drag-out, Manage Items, pinning, removal, Clear Temporary Items, and Open Shelf without activating DropCove.
- Apply the configured edge, configured-monitor precedence, remembered dismissal monitor, deterministic primary-display fallback, and disconnected-monitor preservation rules.
- Add the geometric fullscreen policy and foreground tracking needed to hide and restore the requested rail.
- Complete display-state transitions and non-empty dismissal behavior across Drop Shelf, Edge Rail, settings changes, display changes, and restart.

### Implemented

- Ticket 01: EdgeDocked/collapsed no-activate Rail Handle, persistent remembered monitor ownership, current-monitor Open Shelf, five-field clean settings cutover, edge/monitor Save/Cancel and placement rollback implemented. Fullscreen control remains hidden until ticket 04; expansion/actions are not started.
- Ticket02: native one-shot hover/OLE timers, expanded summaries, canonical bounded rows/scrolling, shared Copy storage intake, projection synchronization and final-item deadline cleanup implemented.
- Ticket 03: expanded rail management, item and batch Copy drag-out, rail-constrained Manage Items, Bulk Pinning, non-focus-stealing Clear/drag confirmations, transient hold lifecycle with outside-pointer collapse, and Open Shelf transition implemented.
- Ticket 04: geometric fullscreen monitor coverage evaluation, event-driven foreground/location hooks without own-process suppression, `ShowRailOverFullscreen` setting with atomic transaction/rollback, and active drag survival across fullscreen transitions implemented.

### Verification

- Ticket 01: final cargo-xwin embedded release build and vue-tsc/Vite succeeded; fresh native suite **71/71** passed. Actual Windows smoke proved 16×280 edge geometry, foreground preservation, current-monitor Open Shelf, settings Save/Cancel, real post-effect JSON/placement rollback, unavailable preference fallback, hidden restart and final-item hiding. Native show/hide cache mismatch found and fixed, then repro passed. Owned Exit/tabs/profile/WebView cleanup and unchanged Run entries verified. See [ticket 01 evidence](../.scratch/edge-rail-workflow/issues/01-edge-docked-handle-placement-settings.md#answer). Mixed-DPI/physical disconnect remain unobserved limits.
- Ticket02 focused TDD checks passed; frontend and cargo-xwin embedded release build passed. Native hover/leave/re-entry, real OS-marshaled FileDrop Copy/Text rejection/Escape cancellation, growth/no-live-shrink/re-expansion, hidden Shelf/Rail DOM refresh and focus preservation verified. Final rendered rows64/gaps4 and scrolling measured after correcting flex shrink. See [ticket02 evidence and limits](../.scratch/edge-rail-workflow/issues/02-expanded-rail-storage-intake.md#answer). Full suite remains an end-of-delivery gate.
- Ticket 03 fresh full suite **74/74** passed; frontend build and cargo-xwin release passed. Native smoke proved outbound Copy consumption/retention, Escape/cancellation, tri-state Bulk Pinning, item/batch removal, Clear Temporary (mixed/all-Pinned/all-Temporary), Open Shelf transition, and natural 300 ms collapse on outside-pointer hold release. Idempotent flyout subclass fix verified. See [ticket 03 evidence](../.scratch/edge-rail-workflow/issues/03-rail-management-copy-drag-out.md#answer).
- Ticket 04 fresh full suite **75/75** passed; frontend build and cargo-xwin release passed. Native smoke proved geometric fullscreen occlusion hiding and handle-only restoration, cross-monitor isolation, `ShowRailOverFullscreen` toggle in Settings with Save/Cancel semantics, and active OLE drag survival without cancellation or corruption. See [ticket 04 evidence](../.scratch/edge-rail-workflow/issues/04-fullscreen-reflow-settings-transaction.md#answer).
- Ticket 05 release: [DropCove-Setup-0.0.5.exe](../build/DropCove-Setup-0.0.5.exe), 3,196,216 bytes, SHA-256 `01e9f3bf40e766a86be6cd53056c2c632070ac36df1da3898991f096e3ecfc38`. Version/package/crate/lock/Tauri, built/installed PE and uninstall metadata report 0.0.5. Freshly compiled 0.0.5 suite passed **75/75**; cargo-xwin NSIS packaging included vue-tsc/Vite.
- Installed native proof: primary launch & single-instance activation, empty dismissal to Hidden, non-empty dismissal to 16×280 Rail Handle, 200ms/300ms hover expansion/collapse, OLE drop-in and duplicate suppression, outbound Copy drag-out with Temporary consumption and Pinned retention, no-activate Manage Items flyout and Clear Temporary, Open Shelf transition, geometric fullscreen occlusion and handle-only restoration, Settings view with `ShowRailOverFullscreen` Save/Cancel, and final-item removal hiding both surfaces. See [ticket 05 evidence](../.scratch/edge-rail-workflow/issues/05-release-0.0.5.md#answer).
- Owned test uninstaller executed, test directories removed, Run key and original 0.0.3 user uninstall/shortcuts restored with unchanged ambient executable hash. User ambient installation stays 0.0.3; 0.0.5 installer is available separately. No commit created. Limits: dual 96-DPI physical monitors tested; mixed-DPI scaling covered by deterministic geometry tests.
Release gates completed on available hardware. Dual 96-DPI physical monitors verified; mixed-DPI scaling remains covered by deterministic geometry tests. Version 0.0.5 was committed as `f0cdc3d` after explicit user approval.

### Deferred

- Shake-to-open, final visual-resource work, full accessibility conformance, database recovery, and installer lifecycle hardening.

## 0.0.6 — Shake-to-open

**Status:** Completed — three Shake-to-open tickets resolved; 0.0.7 remains unstarted pending user review.

### Scope

- Add the low-level mouse hook, minimal callback capture, bounded ordered queue, axis-reversal detector, sensitivity profiles, detection window, and cooldown.
- Add the Shake session coordinator that saves `Hidden` or `EdgeDocked` state, shows the Drop Shelf near the pointer, and restores prior state after release, rejection, unsupported input, duplicate-only input, cancellation, or false positive.
- Keep the shelf visible near the pointer only after a new Shelf Batch is accepted.
- Apply settings changes and rollback behavior when enabling, disabling, or reconfiguring the hook.
- Preserve all non-Shake workflows when hook installation or processing fails.

### Implemented

- Dedicated `WH_MOUSE_LL` message-pumping thread, bounded 256-sample capture queue, axis-reversal detector, Low/Normal/High sensitivity, 600 ms detection window, and 500 ms cooldown.
- Resident-owned, session-ID-bound transient presentation: no-activate near-cursor reveal, accepted-batch Visible promotion, and Hidden/EdgeDocked restoration after release, rejection, duplicate-only input, unsupported input, cancellation, or failure. Application-owned drag-out is excluded from Shake activation.
- Seven-field strict settings cutover, enabled/Normal defaults, actual Enable/Sensitivity controls, draft/Save/Cancel semantics, and transactional hook snapshot/apply/restore. Hook failures warn without removing other resident workflows.
- Regression fixes found through native delivery checks: symmetric Shelf show/hide despite Tao visibility caching; owner-thread Manage Items subclass installation; fullscreen reevaluation after docking/Shake restoration; and Rail whole-batch drag bypassing an already-held busy guard.

### Verification

- Final `vue-tsc`/Vite and cargo-xwin/NSIS builds passed; fresh native test binary `dropcove_lib-7669ec7c64b55353.exe` passed **82/82**. Vendor CRT PDB linker warnings remain nonfatal; no source dead-code warnings remain.
- [DropCove-Setup-0.0.6.exe](../build/DropCove-Setup-0.0.6.exe): **3,203,433 bytes**, SHA-256 `898acd9a3993f7c618a231abcb38debe95f81bdadba219d4f4c25f2b8c2ecf41`. Installed executable: **13,202,944 bytes**, SHA-256 `955ca1f8ecd243b561b9e40a72705024eb054d0f6113f32f59af906a5f2df4e4`; PE product/file version and isolated uninstall registration report `0.0.6`.
- Final installed Windows checks proved sensitivity profiles, left-button gating, no-activate reveal, 600 ms expiry, cooldown, Hidden Start/secondary activation, exact restoration, real OLE accepted/duplicate/unsupported/canceled outcomes, native pointer whole-batch Copy consumption with Pinned retention, Manage Items, Rail timers, geometric fullscreen hide/restore, actual Settings controls and footer reachability, Save/Cancel, hook disable/re-enable, Clear Temporary, final-item hiding, and clean Exit.
- Native runtime-failure dispatch produced an observed Windows notification and retained resident workflows; unchanged Save reinstalled the hook. Separately, debugger fault injection changed only the hook-type argument of one call in an owned test process, producing actual Win32 `ERROR_INVALID_HOOK_FILTER` (`0x80070592`), a native warning toast, atomic file rollback, resident survival, and successful explicit Save retry. No DLL/code bytes or global OS settings were patched. See [release evidence and limits](../.scratch/shake-to-open/issues/03-release-0.0.6.md#answer).
- Owned installer/profile/WebView/fixture/probe processes and directories were removed. Original `0.0.3` uninstall values and shortcut destinations were restored; all Run values and ambient executable SHA-256 stayed unchanged. Version 0.0.6 was committed as `fd6554a` after explicit user approval. Pointer smoke used injected Windows input on two 96-DPI monitors; physical polling-rate variation, mixed-DPI hardware, and spontaneous host-policy hook denial were not exercised.

### Deferred

- Final shell visuals, accessibility conformance, recovery, and distribution hardening.

## 0.0.7 — Presentation and accessibility conformance

**Status:** Completed — eight `ready-for-agent` vertical slices implemented and verified; Version 0.0.8 remains unstarted.

### Scope

- Add asynchronous shell file/folder icons and image thumbnails with fallbacks, bounded previews, cancellation, and resource release tied to surface lifetime.
- Add virtualization for batch collections and Manage Items lists and clear full-item projections when management surfaces close.
- Complete Light, Dark, and High Contrast resources, visible focus behavior, accessibility names/states, live announcements, and full-path accessibility text.
- Add short state-change animations and disable non-essential motion when Windows client-area animations are disabled.
- Validate that earlier workflows remain fully operable by keyboard and without thumbnails or animation.

### Implemented

- Shared semantic token system in `src/style.css` supporting system Dark baseline, Windows Light (`prefers-color-scheme: light`), and Contrast Themes (`forced-colors: active` using system colors).
- Native frame integration via `windows_theme.rs` dynamically observing `WM_SETTINGCHANGE` / `WM_THEMECHANGED` and synchronizing DWM immersive dark mode live across all windows.
- Windows Shell visual pipeline (`windows_visuals.rs`) with two dedicated COM STA workers, 128 bounded queue, DPI-aware 32px and 24px aspect-fit extraction, direct 32bpp BMP byte transfer, deterministic handle release, and generational cancellation.
- Compact multi-item cards bounding previews to at most the first three Shelf Items.
- Virtualized collection scroller (`useVirtualCollection.ts`) rendering direct lists for <=50 items and bounded viewport+overscan ranges for >50 items with listitem semantics, `aria-setsize`, `aria-posinset`, and Tab/Shift+Tab focus traversal.
- Polite live region announcement channel (`useAnnouncer.ts`) across Drop Shelf and Manage Items.
- Canonical 140ms opacity and transform state-change transitions, suppressed under `prefers-reduced-motion: reduce`.

### Verification

- Final `vue-tsc` & Vite build passed; native test suite executed **84 passed, 0 failed** (including scheduler cancellation and saturation contract tests).
- Packaged installer [DropCove-Setup-0.0.7.exe](../build/DropCove-Setup-0.0.7.exe): **3,237,854 bytes**, SHA-256 `4a938a726e83dc195f465169f1622ac3f2b1f466fd5963c342e5e19239872a0d`.
- Installed executable `dropcove.exe`: **13,326,848 bytes**, SHA-256 `b0ae2fecf75287d10c4eabe214d2e4db775e7b6ec126f5c511bf0de1ebc884c2`.
- Real Windows smoke proved: live Light/Dark and Contrast Theme switching; single-item image thumbnail and file icon extraction from Windows Shell via binary Blob URLs; at-most-3 previews on multi-item cards; bounded virtualization on collections >50 items; clean uninstallation and ambient machine state preservation.
- All changes remain uncommitted pending user approval.
### Deferred

- Recovery and distribution hardening in `0.0.8`.

## 0.0.8 — Recovery and distribution hardening

**Status:** Completed — five vertical slices implemented and verified; Version 0.0.9 remains unstarted.

**Plan:** [Recovery and Distribution Hardening specification](../.scratch/recovery-distribution-hardening/spec.md) and five vertical slices. Primary-source research and ADR-0003/ADR-0004 record the recovery and lifecycle decisions.

### Scope

- Add SQLite integrity checking and recoverable database failure handling, including timestamped preservation of the database and matching WAL/SHM sidecars, clean recreation, and user notification with the backup path.
- Complete persistence-failure rollback and safe-state behavior across Shelf mutations, settings changes, hotkey registration, startup registration, and Shake integration.
- Complete the current-user installer wizard, Visual C++ Redistributable prerequisite check, Start Menu and optional Desktop shortcuts, remembered install directory, and Finish-page launch option.
- Complete update and uninstall coordination with the resident process, bounded shutdown wait, abort behavior, startup-registration cleanup, and profile-data preservation.
- Run the complete target-behavior acceptance pass across Windows 11 x64, multi-monitor/DPI scenarios, restart boundaries, and documented integration failures.

### Implemented

- Evidence-preserving SQLite recovery with exclusive timestamped backup directories, streaming original-SHM staging before read-only validation, clean recreation, fatal obstruction handling, and retained warning fallback.
- Extended behavioral failure coverage using the existing Shelf store and settings/integration coordinator; no second transaction framework.
- Current-user custom NSIS wizard, VC++ x64 check, remembered validated directory, shortcut ownership/defaults, and one shared Start Menu path for all folder configurations.
- Profile-scoped cooperative shutdown with a five-second deadline and confirmed process/lease exit; safe update/repair/downgrade rejection and owned-artifact uninstall cleanup.
- Consistent 0.0.8 metadata and a packaged release candidate; no 0.0.9/0.0.10 implementation started.

### Verification

Observed verification:

- Frontend typecheck/Vite build and cargo-xwin packaging passed; final Windows native executable ran **101 tests, 101 passed, 0 failed**. External SDK `LNK4099` missing-PDB warnings remain; no source warning was suppressed.
- Candidate [DropCove-Setup-0.0.8.exe](../build/DropCove-Setup-0.0.8.exe): **3,231,316 bytes**, SHA-256 `b4b5eb44a85e21fd9e552e0516852cace0fb5f2dc13562e6787e8d2f69bc1288`. Installed executable: **13,383,168 bytes**, SHA-256 `0720386a0b23510f652f08299464b20f4eef17b707719d4879bd65c1ce0c5b08`; PE and uninstall metadata report 0.0.8.
- Actual native recovery, rollback, restart, wizard, conflict, repair/update, busy-timeout abort, startup cleanup, and profile-preserving uninstall smoke passed. Windows Sandbox verified missing VC++ abort on the exact 0.0.8 setup (exit 2, no executable created).
- Exact installed 0.0.8 rendered **14 cards for 200 batches** and **8 rows for a 2,000-item batch**, with correct collection-size semantics. Upgrade preserved Shelf state; uninstall preserved the complete owned profile byte-for-byte.
- Complete release gate passed: user confirmed manual desktop acceptance (mixed-DPI, display swap, disconnect/reconnect, visual recovery notifications, and prior workflows) on 2026-10-14. Release 0.0.8 verified.

### Deferred

- Frontend toolchain and dependency modernization in `0.0.9`, and compact Drop Shelf and Edge Rail UI refinement in `0.0.10`.

## 0.0.9 — Frontend toolchain and dependency modernization

**Status:** Completed — all three tickets resolved; user confirmed final exact-candidate acceptance on 2026-10-14. Version 0.0.10 remains unstarted.

**Plan:** [Frontend Toolchain Modernization specification](../.scratch/frontend-toolchain-modernization/spec.md). [Primary-source upgrade research](../.scratch/frontend-toolchain-modernization/research/frontend-upgrade-constraints.md) records the exact compatibility constraints.

### Scope

- Remove unused `pinia` and `vue-router` dependencies and Pinia's empty application registration; preserve Tauri window-label dispatch and composable state ownership.
- Upgrade the active stack to `@vueuse/core` `15.0.0`, `@vitejs/plugin-vue` `6.0.10`, `tailwindcss` `4.3.3`, `vite` `8.3.4`, `vue-tsc` `3.3.13`, TypeScript `6.0.2`, and compatible Vue `3.5.44`; declare Node `>=22`.
- Keep TypeScript `7.0.2` out of the executable graph because vue-tsc `3.3.13` cannot load its native compiler. Preserve full Vue single-file-component typechecking and defer TypeScript 7 until upstream compatibility exists.
- Migrate Tailwind through `@tailwindcss/vite`, remove obsolete PostCSS/Autoprefixer and empty JavaScript Tailwind configuration, and preserve body utilities plus all Dark, Light, and Windows Contrast `--dc-*` tokens.
- Preserve the Vite/Tauri development ports, host, WebSocket/HMR behavior, native-source watch exclusion, production frontend output, application workflows, and Windows x64 cargo-xwin/NSIS packaging.
- Keep all release metadata at `0.0.8` until dependency resolution, typecheck/build, native tests, runtime smoke, and packaging pass; then synchronize and verify one exact `0.0.9` candidate.

### Implemented

- Approved build graph implemented: VueUse 15, Vue 3.5.44, Vite 8, plugin-vue 6, Tailwind v4 Vite integration, vue-tsc 3.3.13, and TypeScript 6.0.2. Unused Pinia/Vue Router and direct PostCSS/Autoprefixer/configuration were removed.
- Node `>=22` declared; Vite WebSocket configuration migrated to `server.ws`; Vite client types cover TypeScript 6 stylesheet side-effect imports.
- Added Tauri's standard default `custom-protocol` feature after user approval of the standalone packaging gap. The installed candidate now serves embedded assets from `http://tauri.localhost/`, not the Vite development server.
- Version metadata synchronized to 0.0.9 only after technical pre-release gates. TypeScript 7 remains deferred, and 0.0.10 remains unstarted.

### Verification

- Frozen strict-peer pnpm install and full vue-tsc/Vite/Tailwind production build passed. Final production-feature Windows test executable ran **101 tests, 101 passed, 0 failed**. Cargo-xwin/Tauri NSIS packaging passed; external SDK LNK4099 warnings remain unsuppressed.
- Native development smoke proved Vite client mount and WebSocket connection on the configured 1420/1421 ports; disposable HMR and native-source watcher-exclusion probes passed and were removed.
- Exact installed candidate registered PE/installer version 0.0.9, mounted embedded Shelf/Manage Items, loaded Shell Visuals, accepted 71 batches, virtualized 70 items to 8 rows, and preserved references after preparation/cancel. Silent install, same-version repair with Cooperative Shutdown, relaunch, and uninstall/Profile Data preservation passed.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `build/DropCove-Setup-0.0.9.exe` | 3,228,174 | `6a340c51059c06060e0c81a3464c9d9e4a50528cfe5eeb6095d59cec5cf97fca` |
| Installed `dropcove.exe` | 13,383,168 | `f8a9d9ad75637d935159c6d1a8aa55f1f43e07b207c4b514a1f913d2f1aca082` |

- User confirmed all outstanding exact-candidate acceptance tests passed on 2026-10-14: interactive setup/Finish launch, Settings and expanded Rail, Dark/Light/Windows Contrast, focus/reduced motion, Explorer Copy/Move/cancel, and hotkey/startup/Shake workflows.
- Original installation registration, shortcut bytes, Profile Data, and resident were preserved; owned smoke artifacts were removed. Temporary-profile startup polluted the shared Run value before a valid baseline was captured; user approved reconciliation to the verified real installation/profile. Exact original startup bytes are not claimed restored; [Ticket 03](../.scratch/frontend-toolchain-modernization/issues/03-release-0.0.9.md) records the exception.

### Deferred

- TypeScript 7 until vue-tsc/Volar supports its native compiler without dropping Vue single-file-component checks.
- Compact Drop Shelf and Edge Rail UI refinement in `0.0.10`.

## 0.0.10 — Compact Drop Shelf and Edge Rail UI refinement

**Status:** Planned.

### Scope

- Eliminate the corner gap and outer border artifact at all four corners of the Drop Shelf (and related popup surfaces) so the rounded surface border aligns flush with the transparent window frame.
- Fix Edge Rail corner rendering so both desktop-facing corners (top and bottom) are symmetrically rounded with a continuous border while the screen-edge corners remain square (fixing the current bug where the bottom desktop-facing corner is square and mismatched with the rounded top border).
- Remove the Drop Shelf header (`DropCove` title and `References, not copies` subtitle) across all states, and simplify the empty Drop Shelf prompt to a downward icon and `"Drop here"`.
- Remove the drag-hint subtitle (`"Drag batch to copy"`, `"Drag to copy"`, and `"Pinned · Drag to copy"`) from all single-item and multi-item batch cards across Drop Shelf and Edge Rail to reduce card and row height.
- Replace `Pin`, tri-state `Pin All`, `Remove`, `Manage Items`, and `Close` text buttons with compact single-row icon controls across `ShelfBatchCard` and `ManageItems`, complete with hover, `focus-visible`, tooltip, and accessible name/state support.
- Replace `Clear Temporary Items` with a trash-can icon button positioned at the bottom-left corner of both the Drop Shelf and the expanded Edge Rail.
- Remove the `Retained Batches (N)` header title from the expanded Edge Rail and replace `Open Shelf` with a directional icon button positioned at the top-right corner when docked on the `Left` screen edge and at the top-left corner when docked on the `Right` screen edge.

### Implemented

- Not started.

### Verification

Before completion:

- Inspect all four corners of the Drop Shelf and the desktop-facing vs. screen-edge corners of the Edge Rail (collapsed and expanded, `Left` and `Right` edges) on Windows 11 x64 across Dark, Light, and Contrast themes to confirm symmetric corner rounding and no interior gap or mismatched border.
- Verify that the Drop Shelf renders without the top title/subtitle header, displays only the icon and `"Drop here"` when empty, and renders compact batch cards without drag-hint text.
- Exercise single-row icon actions (`Pin`, `Pin All`, `Manage Items`, `Remove`, and `Close`) across Drop Shelf, Edge Rail, and Manage Items via pointer and keyboard (`Tab`, `Shift+Tab`, `Enter`, `Space`), verifying hover, focus rings, tooltips, and `aria-label`/`aria-pressed` semantics.
- Verify the bottom-left trash-can icon triggers `Clear Temporary Items` confirmation on both Drop Shelf and expanded Edge Rail, and verify the top-corner `Open Shelf` icon switches between top-right (`Left` edge) and top-left (`Right` edge) according to the active docking edge.
- Build and launch `DropCove-Setup-0.0.10.exe` on Windows 11 x64 and pass the full native regression and smoke suite.

### Deferred

- TypeScript 7 remains dependency-driven and unassigned until vue-tsc/Volar supports it. Any product scope after `0.0.10`, including `0.1.0` or `1.0.0`, requires explicit approval.
