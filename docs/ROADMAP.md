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
- Responsive grid, Automatic Shelf Sizing, pending empty-shelf width intent, and persisted non-empty shelf width.
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
- Complete responsive sizing: preferred width, Automatic Shelf Sizing, Manual Height Override, resize hit testing, live sizing rules, and final-item reset.
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

**Status:** Completed — all four tickets resolved; user confirmed final human acceptance on 2026-10-15. Git commit authorization remains separate.

**Plan:** [Compact Shelf and Rail Refinement specification](../.scratch/compact-shelf-rail-refinement/spec.md), with four approved implementation tickets.

### Scope

- Eliminate the corner gap and outer border artifact at all four corners of the Drop Shelf (and related popup surfaces) so the rounded surface border aligns flush with the transparent window frame.
- Fix Edge Rail corner rendering so both desktop-facing corners (top and bottom) are symmetrically rounded with a continuous border while the screen-edge corners remain square (fixing the current bug where the bottom desktop-facing corner is square and mismatched with the rounded top border).
- Remove the Drop Shelf header (`DropCove` title and `References, not copies` subtitle) across all states, and simplify the empty Drop Shelf prompt to a downward icon and `"Drop here"`.
- Remove the drag-hint subtitle (`"Drag batch to copy"`, `"Drag to copy"`, and `"Pinned · Drag to copy"`) from all single-item and multi-item batch cards across Drop Shelf and Edge Rail to reduce card and row height.
- Replace `Pin`, tri-state `Pin All`, `Remove`, `Manage Items`, and `Close` text buttons with compact single-row icon controls across `ShelfBatchCard` and `ManageItems`, complete with hover, `focus-visible`, tooltip, and accessible name/state support.
- Replace `Clear Temporary Items` with a trash-can icon button positioned at the bottom-left corner of both the Drop Shelf and the expanded Edge Rail.
- Remove the `Retained Batches (N)` header title from the expanded Edge Rail and replace `Open Shelf` with a directional icon button positioned at the top-right corner when docked on the `Left` screen edge and at the top-left corner when docked on the `Right` screen edge.

### Implemented

- Removed opaque root background and native Drop Shelf shadow; enabled transparent, shadow-free Manage Items windows. CSS owns rounded surface borders; internal confirmation dialogs remain independent modal surfaces.
- Removed Drop Shelf title/subtitle and visible card drag hints; empty state now contains only a downward icon and `Drop here`. Full-path tooltips and accessible drag guidance remain.
- Added dependency-free `currentColor` SVG action icons, 24-pixel controls, native tooltips, binary Pin `aria-pressed`, and three-state Bulk Pinning `aria-checked`.
- Delivered 48-pixel batch cards, 54-pixel Manage Items rows, single trailing action rows, corrected flex virtual-scroll spacers, and compact Rail height tiers.
- Mirrored collapsed/expanded Rail corner ownership, removed its header title, placed inward Open Shelf at the desktop-facing top corner and conditional Clear trash at bottom-left.

### Verification

- `pnpm build` passed complete vue-tsc and production bundling. Cargo-xwin compiled the native regression executable; the focused rail-height test failed before the formula change and passed afterward. Full native suite: **101 passed, 0 failed**. Windows x64 release/Tauri NSIS packaging passed; external SDK missing-PDB LNK4099 warnings remain.
- Real development WebView2 smoke exercised empty/populated Shelf, Pin and tri-state bulk transitions, Manage Items pointer/keyboard opening, Tab order, Escape, compact control semantics, Clear cancel/confirm, and Left/Right Rail. Media emulation inspected Dark, Light, and forced-colors; this is not actual Windows Contrast Theme acceptance.
- Large native collections: 65 Shelf Batches and a 70-item batch; 48-pixel cards and 54-pixel management rows observed. Fixed flex spacer clamping: actual Manage Items scrollTop changed from a 163-pixel ceiling to 1500, with first virtual index 23 and 12 mounted rows. Left Rail expanded to 280 × 640 and scrolled to 2200 with bounded mounting.
- The final exact setup is `build/DropCove-Setup-0.0.10.exe`: **3,231,015 bytes**, SHA-256 `f69173e3eb1d77d0292ede8cb4a2f6f2825cf082aee46ee06121bc8dddedfe9b`. Exact installed executable: **13,383,168 bytes**, SHA-256 `844a545a112ab957f1afc9e22680953d77683826445708262bbdafa2335af418`. Earlier hashes are superseded and are not acceptance artifacts.
- Final installed bytes ran with Vite stopped, loaded embedded `http://tauri.localhost/` assets, and passed native Pin/Remove/Manage Items smoke. Real desktop capture inspected Shelf corners; installed Manage Items rendered in Dark/Light/forced-colors emulation.
- Physical mouse/OLE smoke used an owned Windows Forms Copy destination, not Explorer: Temporary `item-3.txt` copied exact fixture bytes and its reference was consumed; Escape retained Temporary `item-4.txt`; Pinned `item-1.txt`/`item-2.txt` batch copied both exact source bytes and retained both references. All 75 fixture sources remained unchanged.
- Exact candidate installed/reinstalled with exit 0; owned profile shutdown and uninstall exited 0. Uninstall removed the installed executable and preserved the owned Profile Data byte-for-byte. Shared shortcut bytes and installer registrations were restored; original application was restarted, and HKCU Run `DropCove` was restored after its startup reconciliation. Owned smoke processes, fixtures, WebView data, and temporary harness were removed.
- **Ambient-profile restoration limit:** the default `shelf.db` hash differs from its pre-smoke hash after cooperative shutdown/restart. On 2026-10-15 the user accepted this limitation and confirmed ambient profile content remains valid; byte-for-byte database restoration was not claimed because no pre-smoke backup existed.
- **Human gate:** User confirmed all outstanding acceptance tests passed on 2026-10-15 against the exact candidate recorded above (Windows Dark/Light/Contrast corner and keyboard/UI Automation checks, physical Explorer drag-in/drag-out, resident workflows, and ambient-profile content verification). No Git commit is created without separate authorization.

### Deferred

- TypeScript 7 remains dependency-driven and unassigned until vue-tsc/Volar supports it.
- Tailwind component styling migration in `0.0.11` and compact scrolling and Lucide icon consistency in `0.0.12`, approved on 2026-10-15.

## 0.0.11 — Tailwind component styling migration

**Status:** Completed — tickets 01–04 resolved; exact candidate 0.0.11 verified. Later versions remain unstarted.

### Scope

- Replace ordinary component-scoped CSS across the frontend with Tailwind CSS v4 utilities without changing rendered appearance, dimensions, interaction, accessibility, or motion.
- Retain only CSS required for application-wide semantic theme tokens, root sizing and reset behavior, Windows Light and Contrast Theme integration, reduced-motion and focus policy, WebView pseudo-elements, and runtime-computed geometry.
- Remove superseded component style rules instead of preserving parallel Tailwind and handwritten CSS conventions.

### Implemented

- Replaced ordinary component-scoped CSS across all thirteen Vue style blocks and the global stylesheet with direct Tailwind CSS v4 utility classes.
- Added value-free `@theme inline` aliases mapping all twenty-eight `--dc-*` semantic variables without establishing a second token source.
- Preserved runtime dynamic style bindings for virtual container positioning and Shell Visual icon sizing.
- Converted all action button controls to shared literal utility chains without wrapper components, styling props, or `@apply` abstractions.
- Retained plain CSS only for semantic token declarations, theme media queries, reset, transparent WebView canvas, Windows forced-colors policy, global focus-visible indicator, and reduced-motion policy.
- Synchronized all manifests to version `0.0.11` and produced exact release candidate `DropCove-Setup-0.0.11.exe`.

### Verification

- Complete frontend build (`pnpm build`: vue-tsc single-file component typechecking and Vite production bundling) passed.
- Cargo-xwin release compilation and complete native test executable passed (101/101 tests passed).
- Identity-verified before/after WebView2 DOM comparisons matched 35 states and 2,292 stable DOM-path-aligned elements with 0 computed-style/geometry differences.
- Exact release candidate hashes:
  - Installer `build/DropCove-Setup-0.0.11.exe`: 3,232,765 bytes, SHA-256 `81e551a81e716f9d4bf40bcdec082537c8222559650e0a93bb33b9f8ea85d337`.
  - Installed executable: 13,385,728 bytes, SHA-256 `d528988819c05e3bfb4c2a0c96ccdb5cdb52d6affd2fb872d8265b2696240abe` (unbundled executable SHA-256: `010d4f254b84d8b554ffaad7b819e0c51597d774f2ce77943cb6785bfb370373`).
- User confirmed human acceptance across all interactive on-screen gates.
- Installed lifecycle smoke verified silent installation, secondary routing, Cooperative Shutdown with exit 0, in-place upgrade, and uninstallation removing binary while preserving Profile Data.
- Disposable baseline scripts and fixtures cleaned up.
- Changes remain uncommitted pending user approval.
### Deferred

- Compact scrollbar styling, persistent scroll-viewport insets, and Lucide icon replacement remain in `0.0.12` so the Tailwind migration has no intentional visual changes.
- TypeScript 7 remains dependency-driven and unassigned until vue-tsc/Volar supports it.

## 0.0.12 — Compact scrolling and Lucide icon consistency

**Status:** Completed — tickets 01–03 resolved; exact Windows x64 NSIS candidate and installed lifecycle verified; user accepted the installed 0.0.12 candidate on 2026-10-15.

### Scope

- Apply one compact scrollbar treatment to every user-facing vertical scroll surface in the Drop Shelf, Edge Rail, Manage Items, and Settings: an 8-pixel interaction channel, an approximately 4-pixel visible rounded thumb, a transparent track, semantic theme colors, visible idle state, stronger hover and drag states, and Windows Contrast Theme system-color behavior.
- Preserve a fixed top viewport inset at every scroll position so fully or partially visible Shelf Batches and rows never touch the containing surface border; enforce the invariant across both non-virtualized and virtualized scroll containers in the Drop Shelf, Edge Rail, and Manage Items without fixed component-specific offsets.
- Add `lucide-vue-next` and replace all handwritten action and empty-state SVG paths with Lucide icons while preserving tooltips, accessible names and states, focus behavior, and 24-by-24-pixel control targets; use 14-pixel action glyphs and a 28-pixel empty-state glyph.
- Preserve narrow-surface layout, virtualization measurements, scrolling bounds, card and row density, drag workflows, and existing Drop Shelf and Edge Rail placement behavior.

### Implemented

- Added compact native WebView2 scrollbar styling across the four approved scroll owners, with semantic idle/hover/active colors and forced-colors system mappings.
- Added external 4-pixel collection viewport boundaries, retained horizontal scrollport padding, and redistributed vertical padding without increasing total clearance. Shared virtualization measures horizontal content origin from actual computed padding rather than component-specific offsets.
- Made Drop Shelf virtual columns follow the actual CSS grid track count through resize observation.
- Added `lucide-vue-next` 1.0.0 and replaced handwritten action/empty-state paths; observed 14-pixel actions, 24-pixel targets, 28-pixel empty-state glyph, filled Pin, and mixed Pin/Minus.
- Corrected the expanded Edge Rail height budget to `max(280, min(640, 52*n+62))` with user approval; preserved header/footer, row density, targets, and min/max heights.
- Synchronized version metadata to `0.0.12` after implementation acceptance and produced `build/DropCove-Setup-0.0.12.exe`.

### Verification

- Complete the existing frontend, native, Windows x64, and NSIS delivery checks required by the roadmap delivery rules.
- Exercise the real WebView2 Drop Shelf, Edge Rail, Manage Items, and Settings scroll surfaces in Dark, Light, and Windows Contrast Themes, including idle, hover, drag, keyboard-focus, minimum-width, and maximum-scroll states.
- Exercise both ordinary (≤50 items) and virtualized (>50 items) collections at the start, middle, and end of each applicable viewport; verify the persistent top inset, bounded mounting, correct width and column calculations, keyboard navigation, and unchanged drag and management workflows.
- Inspect every Lucide replacement at normal and compact widths and verify the 24-pixel control target, 14-pixel action glyph, 28-pixel empty-state glyph, tooltip, accessible name, and pressed, checked, mixed, disabled, hover, and focus-visible states where applicable.

Observed development smoke on 2026-10-15:
- `pnpm build` passed Vue typechecking and Vite production bundling; cargo-xwin Windows x64 release build passed with non-fatal vendor CRT missing-PDB warnings.
- Newly produced Windows native test executable passed 101/101 tests. `cargo xwin test` compiled successfully but attempted the unavailable Wine runner; the resulting executable was run directly on Windows.
- Real WebView2 showed the top clip moving from y=1 to y=5 on the Drop Shelf, an 8-pixel scrollbar channel, 2-pixel transparent thumb borders, bounded virtual mounting, ordinary 11-batch and 50-item collections, and reactive 1/2/3-column layouts at widths 180/356/524.
- Virtualized Manage Items reached item index 55 at maximum scroll with 10 mounted rows; Tab revealed the first item and Escape restored the originating Manage Items focus. Its boundary measured 4 pixels in Dark, Light, and emulated forced-colors modes. Actual Windows Contrast Theme acceptance remains unobserved.
- Expanded Edge Rail reached batch index 56 at maximum scroll with 14 mounted cards and a 4-pixel boundary. At 11 batches, measured window height was 624, header 28, footer 28, borders 2, inner scrollport 562, and scroll content 572: a 10-pixel premature scroll range.
- The approved formula `52*n+52` conflicts with the preserved header/footer/border budget: 11 rows plus gaps and total clearance need 576 pixels, but only 566 remain. Changing the formula or chrome geometry requires user approval; neither was silently changed.
- Initial development smoke executable: 13,385,728 bytes, SHA-256 `49469a4eae7e0dfb652ca9007ea871ddc3f4d7a1e24cc7729d3ba07c0161b2e5`. After restoring scrollport horizontal padding and measuring the virtual horizontal origin from computed padding, the rebuilt executable was 13,385,728 bytes, SHA-256 `cbcd6e2c81e37652cb355cbf57a4adae8a6586e9c24512617df5922bf3ac84bd`. Both use metadata version 0.0.11; neither is a packaged 0.0.12 candidate.
- Static Standards and Spec reviews returned no source findings; runtime evidence supersedes their inferred rail-fit claim. Settings runtime, physical scrollbar hover/drag, Left Rail, actual Windows Contrast Theme, full native drag workflow, and exact installer lifecycle remain unverified.
- Corrected-build real WebView2 assertions passed: Drop Shelf retained 7 pixels on both sides of virtual cards at widths 180/356/524, with 1/2/3 columns and 8/16/24 mounted cards; Rail retained 6 pixels on both sides; Manage Items retained 7 pixels on both sides with 10 mounted rows. All three retained the fixed 4-pixel top boundary. Frontend and cargo-xwin builds passed again after the correction.
- Owned smoke Residents exited through Cooperative Shutdown; browser attachments, fixtures, profiles, and WebView data were removed. Existing installed user Resident was not stopped. No Git commit was created.
- Corrected Edge Rail height budget verification on 2026-10-15:
  - Executable: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe` (13,385,728 bytes, SHA-256 `db96271dd6654f184f7f8ff644ef1d8ba2be0940c1235bcd71b781bfa18d7276`).
  - Real Windows WebView2 expanded Rail proved exact height formula `52*n+62` across batch counts 5 through 11 and 12 on an isolated profile with unique CDP port and owned fixtures seeded via `accept_drop`.
  - Docked Drop Shelf to Edge Rail via native main window `WM_CLOSE`; expanded via `on_rail_pointer_enter` and `tick_rail`.
  - Proved batch counts 5 through 11 match expected heights 322, 374, 426, 478, 530, 582, and 634 (52px step) with zero vertical scroll overflow (`scrollHeight == clientHeight`, no premature scrollbar).
  - Count 11 measured window height 634, scroll content 572, inner scrollport 572; the former 10-pixel deficit is fully resolved.
  - Proved batch count 12 achieves clamped maximum height 640, scroll content 624, client height 578, vertical overflow present (`scrollHeight > clientHeight`), and reachable maximum scroll with the 12th card fully visible in the viewport.
  - Proved 4px fixed top viewport boundary outside the scrollport and 6px horizontal clearance between cards and the rail container / scrollbar channel across all tested counts.
  - Clean cooperative shutdown (`--shutdown --profile`) exited code 0; primary process exited cleanly; owned profiles, WebView2 data, fixtures, and scaffolds deleted.
  - Verification limits: Batch counts 1-4 and virtualized >50 items not re-exercised in this smoke run; left-docked rail edge, high DPI display scaling, actual Windows Contrast Theme, physical mouse drag/hover, Settings runtime, and packaging/installer remain unverified here.

- Approved rail formula correction compiled successfully; focused native rail regression passed 1/1 and the complete rebuilt Windows native suite passed 101/101 on 2026-10-15. Real WebView2 tier verification eliminated premature overflow for every count 5–11; count 12 retained the 640-pixel cap and reachable final card. Remaining acceptance limits above are unchanged.

Official candidate and acceptance on 2026-10-15:
- User reported all requested interactions successful on the corrected development build, closing tickets 01 and 02. This is human-reported interaction/theme acceptance, not an agent observation or acceptance of the later exact installer.
- Official `make build` passed frontend typechecking/bundling, cargo-xwin release compilation, and NSIS packaging. A first attempt could not replace the review executable while it was running; that exact profile was cooperatively shut down before the successful build.
- Exact installer `build/DropCove-Setup-0.0.12.exe`: 3,233,822 bytes, SHA-256 `27f569cf9d37c59380a090d9e531128531689fd4958b056db40f78ec27428411`.
- Bundled build executable: 13,385,728 bytes, SHA-256 `e1fadfff88382a1afca41240ba317c4206b5b3ec313507db8a97f23a8a9fdbd7`. Separately measured installed executable: 13,385,728 bytes, SHA-256 `296b50c7c31bcb335148c6b8042744375746f7c246f353a1b094efe8035eff0a`.
- Exact installed lifecycle smoke passed silent installation and version registration, embedded frontend loading, intake/pinning/management/mixed glyph state, secondary routing, Cooperative Shutdown (exit 0), same-candidate reinstall/update, safe shortcut-conflict abort, and uninstall removing binary while preserving Profile Data. Owned smoke data/processes were cleaned up; original user installation/profile/shortcut/registry baseline restored.
- Final Standards and Spec reviews found no actionable source defect. The user explicitly accepted the installed 0.0.12 candidate on 2026-10-15 after receiving its exact installer path and SHA-256, closing the final physical/theme acceptance gate. This is human-reported acceptance, separate from agent observations. Fractional/high-DPI and multi-monitor combinations were not measured by the agent. All changes remain uncommitted; no Git commit was authorized or created.
### Deferred

- TypeScript 7 remains dependency-driven and unassigned until vue-tsc/Volar supports it.

## 0.0.13 — Surface spacing consistency

**Status:** Completed — user confirmed final human acceptance of candidate 0.0.13 on 2026-10-16.

**Plan:** [Surface Spacing Consistency specification](../.scratch/surface-spacing-consistency/spec.md).

### Scope

- Keep an `8`-logical-pixel preferred separation between Manage Items and the originating Shelf Batch when opened from either the Drop Shelf or expanded Edge Rail, while preserving right/left/below/above preference, the `16`-pixel work-area margin, and on-screen clamping as the constrained fallback.
- Align expanded Edge Rail Open Shelf and Clear Temporary Items controls to matching `4`-pixel outer vertical offsets with existing `24 × 24` targets, `6`-pixel horizontal insets, and mirrored Left/Right placement.
- Preserve the current Rail height contract `max(280, min(642, 52 × batch count + 70))`, collection geometry, scrolling, themes, accessibility semantics, lifecycle, drag behavior, and source-file safety.

### Planned verification

- Deterministic native placement regression for all four candidate directions, all work-area corners, complete-gap eligibility, constrained work areas, and fallback clamping.
- Frontend production build and complete cargo-xwin-produced Windows native regression executable.
- Actual Windows Resident/WebView2 inspection for both Manage Items origins and both Rail docking edges in available Dark, Light, Windows Contrast, DPI, and monitor configurations; every unavailable configuration remains explicitly unobserved.
- After implementation acceptance, synchronize version metadata to `0.0.13`, produce one exact Windows x64 NSIS candidate, and verify those installed bytes according to the delivery rules.

### Deferred

- TypeScript 7 remains dependency-driven and unassigned until vue-tsc/Volar supports it.
- Any redesign beyond the approved Manage Items and Edge Rail spacing corrections.

### Implemented

- Added `GAP = 8.0` in `src-tauri/src/manage_items.rs` for right, left, below, and above placement candidates with complete monitor margin checks and fallback on-screen clamping.
- Adjusted Edge Rail header in `src/components/rail/RailHandle.vue` to `items-start pt-[4px] pb-0` with `28px` height; preserved footer at `min-h-[24px] pb-[4px]`.
- Preserved canonical Rail height formula `max(280, min(642, 52 × batch count + 70))`, collection bounds, and height tiers.
- Synchronized canonical metadata across `VERSION`, `package.json`, `Cargo.toml`, and `tauri.conf.json` to `0.0.13`.

### Observed verification on 2026-10-15

- Frontend build (`vue-tsc && vite build`) and complete cargo-xwin Windows native regression passed (**104/104 tests passed**).
- Real Windows Resident WebView2 surfaces measured:
  - Drop Shelf Manage Items popup: card right `3523` to popup left `3531` = **8.0 logical px**.
  - Left Edge Rail: card right `2193` to popup left `2201` = **8.0 logical px**.
  - Right Edge Rail: card left `3567` to popup right `3559` = **8.0 logical px**.
  - Edge Rail corner actions: Open Shelf at top `4px` (inner, `5px` outer) and Clear at bottom `4px` (inner, `5px` outer); header and footer measured exactly `28px` each.
- Exact candidate artifacts:
  - Installer `build/DropCove-Setup-0.0.13.exe`: **3,233,859 bytes**, SHA-256 `2f0e9588ef72b8ca098da23a81e1e967587dcc4ebcecb024457dcbc54aa65ac3`.
  - Installed `dropcove.exe`: **13,387,264 bytes**, SHA-256 `a84dc4b6692947211d0c5f808fb2525951d673a471225f8cece03c19dc1ff3f9`.
  - Unbundled target release binary: **13,387,264 bytes**, SHA-256 `268ebb2f0fbe2274ecc4ab8454d4abf006ba4058c3f5afb0693ab234f5104f74`.
- Installed lifecycle smoke:
  - Silent installation into isolated path registered DisplayVersion `0.0.13`.
  - Embedded production loading with Vite stopped verified.
  - Seeded batch intake via `accept_drop` IPC and restart restore.
  - Measured Manage Items gap from card right `2103` to popup left `2111` = `8px`.
  - Measured matching 4px Open Shelf/Clear vertical offsets.
  - Secondary activation returned code 0; Cooperative Shutdown returned code 0.
  - Reinstall/update of same candidate returned code 0; Start Menu shortcut verified.
  - Uninstaller removed binary while preserving profile `shelf.db` (32,768 bytes).
  - Temporary smoke files cleaned; registry baseline DisplayVersion 0.0.12 restored.
- **Unobserved limits:** fractional DPI scaling, mixed-DPI multi-monitor environments, Windows Light Theme, actual Windows Contrast Theme, and live on-screen work-area boundary clamp remain unobserved in this release run.

Final release gate completed: the user confirmed `0.0.13 đã nghiệm thu` on 2026-10-16. This is user-reported human acceptance, not additional agent-observed coverage. Git commit authorization remains separate.

## 0.0.14 — Drop Shelf window controls

**Status:** Completed — user confirmed final human acceptance on 2026-10-16. Git commit authorization remains separate.

**Plan:** [Drop Shelf Window Controls specification](../.scratch/drop-shelf-window-controls/spec.md).

### Scope

- Add a compact titleless `28`-logical-pixel Drop Shelf control strip with an existing-style `24 × 24` top-right `X` action named `Hide Drop Shelf`.
- Route the `X` action through the existing Dismiss Drop Shelf behavior: an empty Shelf transitions to `Hidden`, a non-empty Shelf transitions to `EdgeDocked`, Manage Items closes, Shelf references remain intact, and the Resident Instance continues running.
- Allow native Windows movement from the remaining control-strip drag region while excluding the close action, Shelf Batches, action controls, scrollbars, dialogs, and eight-direction resize handles.
- Close Manage Items before movement, block movement during resize or modal/transactional interaction, and preserve existing item/batch drag-out, scrolling, confirmation, and resize behavior.
- Keep movement session-only. Hotkey, activation, Open Shelf, and restart continue to use the established cursor-monitor or Edge Rail monitor placement rules; no position setting or persistence schema is added.
- Preserve Dark, Light, and Windows Contrast presentation, keyboard access to the close action, accessible name and tooltip behavior, focus visibility, resident lifecycle, and source-file safety.

### Implemented

- Added titleless 28px control strip in `src/components/shelf/DropShelf.vue` with 24×24 top-right `X` action (`ActionIcon name="close"`) and centered decorative movement affordance (`ActionIcon name="grip"`).
- Added backend command `dismiss_drop_shelf` in `src-tauri/src/lib.rs` validating caller window (`main` only) and delegating to `resident::dismiss_shelf`.
- Added backend command `prepare_shelf_move` in `src-tauri/src/lib.rs` and `shelf_resize.rs` validating caller window, rejecting active resize/content drag, and closing Manage Items before native drag begins.
- Scoped `core:window:allow-start-dragging` exclusively to `"main"` in `src-tauri/capabilities/main-movement.json`.
- Added `startMove` in `src/composables/useShelf.ts` with primary-pointer, modal/busy/resizing guards, and `finally` reset, calling native `window.startDragging()`.
- Implemented two-stage Escape in `useShelf.ts`: first Escape closes open Manage Items; second Escape docks non-empty Shelf.
- Synchronized canonical version metadata across `VERSION`, `package.json`, `Cargo.toml`, and `tauri.conf.json` to `0.0.14`.

### Verification

- Complete frontend build (`vue-tsc && vite build`) and complete cargo-xwin Windows native regression suite passed (**106/106 tests passed**, including new work-area clamp unit test).
- Real Windows Resident WebView2 surfaces measured:
  - Drop Shelf control strip: exactly 28 logical px high.
  - Close button: 24×24 target at x=158/y=3, elementFromPoint hit tested true.
  - Four-row automatic growth: measured 28px strip, 48px cards, clientHeight 212px, scrollHeight 212px (0 premature overflow).
- Real Windows interaction smoke proved:
  - Empty Shelf X dismisses to `Hidden`, retaining Resident; secondary activation returns 0.
  - Non-empty Shelf X dismisses to `EdgeDocked` Rail Handle, closes open Manage Items, and preserves references after Open Shelf.
  - Non-main callers (`edge-rail`, `manage-items`) attempting `dismiss_drop_shelf` or `prepare_shelf_move` rejected with error; dragging capability denied by ACL on non-main surfaces.
  - Keyboard Enter and Space activate X with visible focus; two-stage Escape closes Manage Items first, then docks Shelf on second press.
  - Native drag from control strip moved window within monitor 1 and across monitor boundary to `DISPLAY2` (`x=2373`); release cleanly restored controls via `finally` handler.
  - Modal Clear Temporary dialog blocked window movement; Escape canceled dialog and preserved all references.
  - Physical Windows OLE Copy drag-out transferred Temporary item to real Forms target, consumed only the transferred Temporary reference, copied all Pinned batch items, and retained Pinned references. Escape cancellation retained all references. Source fixture files remained byte-for-byte unchanged.
  - All eight resize directions functional; held resize disabled X and marked `aria-busy=true`.
  - Manual position preserved across content mutation/unpin; reopen via secondary launch repositioned to cursor monitor.
- Exact candidate artifacts:
  - Installer `build/DropCove-Setup-0.0.14.exe`: **3,244,674 bytes**, SHA-256 `66d7df7efbc4c2abc42abc017ccb12ef7c511a5ad19798732d9e4d4fd7b55d5a`.
  - Installed binary: `dropcove.exe`: **13,457,408 bytes**, SHA-256 `181962e9245c449641aa4bb0bf0349f8131b90288d19080f8a88c86fe5ea37b4`.
  - Unbundled target release binary: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe`: **13,457,408 bytes**, SHA-256 `d88df95cb88c288c6ee96b22524739aefc4783bf0cd5f4ad95a3881c14c395aa`.
  - Silent installation into isolated path registered DisplayVersion `0.0.14`.
  - Same-candidate reinstall/update and Start Menu shortcut creation verified.
  - Embedded production loading with Vite stopped verified; control strip and 24×24 X rendered at `http://tauri.localhost/`.
  - Installed X dismissal, secondary activation, and native movement verified.
  - Cooperative Shutdown (`--shutdown`) returned 0 with process exit 0.
  - Uninstaller cleanly removed installed directory and binary while preserving Profile Data `shelf.db` byte-for-byte (`32,768 bytes`, identical SHA-256).
  - Ambient HKCU Run registry value restored to pre-smoke baseline.
  - All temporary smoke and fixture directories removed.
- **Unobserved limits:** physical high-DPI scaling factors (exercises performed at scale 1), physical hardware keyboard chord detection (Windows-injected hotkey chords skipped in favor of native message dispatch), and actual physical Windows Contrast Theme monitor presentation (covered via media feature emulation) remain unobserved in this release run.

The user confirmed `0.0.14 đã nghiệm thu` on 2026-10-16. This is user-reported final acceptance, not additional agent-observed coverage. Git commit authorization remains separate.

### Deferred

- Durable Drop Shelf position, placement settings, custom snapping, moving Manage Items with the Drop Shelf, drag movement for other surfaces, and using the `X` action to exit DropCove.

## 0.0.15 — Drop Shelf utility menu and About surface

**Status:** Completed — user confirmed all acceptance checks passed and explicitly authorized committing the changes on 2026-10-16. All four tickets are resolved.

**Plan:** [Drop Shelf Utility Menu and About Surface specification](../.scratch/drop-shelf-utility-menu-about/spec.md), with four approved slices: [01 Clear confirmation](../.scratch/drop-shelf-utility-menu-about/issues/01-drop-shelf-clear-confirmation.md), [02 notification-area About](../.scratch/drop-shelf-utility-menu-about/issues/02-about-surface-notification-area.md), [03 Drop Shelf Utility Menu](../.scratch/drop-shelf-utility-menu-about/issues/03-drop-shelf-utility-menu.md), and [04 release](../.scratch/drop-shelf-utility-menu-about/issues/04-release-0.0.15.md).

### Scope

- Move the Drop Shelf's Clear Temporary Items confirmation into a dedicated decorated Tauri window sized for the complete message and actions. Keep it fixed-size, taskbar-hidden, always-on-top, centered on the Drop Shelf monitor, modal to Drop Shelf interaction, and reusable through focus while already open. `X` and `Escape` cancel. Use the shorter copy `Clear temporary items?` and `Pinned items stay. Source files and folders aren’t deleted.`
- Preserve the existing confirmed Clear Temporary Items lifecycle: remove Temporary references, preserve Pinned references, never delete source filesystem objects, retain atomic persistence rollback, and close stale confirmation state safely. The Edge Rail confirmation remains unchanged in this version.
- Add an always-visible `24 × 24` More action at the Drop Shelf's bottom-right. Open an in-shelf popover above it with `Settings` followed by `About`; outside click or `Escape` closes the popover and returns focus. Reuse the existing Settings surface and singleton opener.
- Add one decorated, fixed-size, taskbar-visible About Surface centered on the invoking monitor. Repeated requests focus the existing surface; `X` and `Escape` close it. Show the existing DropCove logo, product name, synchronized runtime/build version, author `DropCove Contributors` sourced from `Cargo.toml.package.authors`, and a `Source` link sourced from `Cargo.toml.package.repository` that opens in the default browser.
- Include a disabled `Check for updates` action with `Coming soon`. Do not add updater dependencies, configuration, permissions, background checks, or a clickable no-op.
- Add `About` to the Windows notification-area menu between `Settings` and `Exit DropCove`, opening or focusing the same About Surface without changing Drop Shelf visibility or Resident Instance lifetime.
- Follow the existing Segoe UI typography and DropCove surface, accent, text, border, focus, and danger tokens. Preserve Dark, Light, Windows Contrast, keyboard, focus-return, and reduced-motion behavior.
- Read the About version from synchronized runtime metadata. Actual pre-bump builds displayed `0.0.14`; the synchronized exact candidate displays `0.0.15` without a hard-coded frontend milestone.
- Keep `Cargo.toml.package.homepage` unset until DropCove has a public product site. Do not present the repository as `Website`, and do not duplicate author or repository values in frontend or Tauri configuration; project the Cargo metadata to the About Surface through one Rust command.

### Deferred

- Implementing update checks or adding a Tauri updater integration.
- Moving the Edge Rail Clear Temporary Items confirmation or adding the Drop Shelf Utility Menu to the Edge Rail.
- Additional About metadata such as description, license, copyright, or release notes.
- A public product homepage and `Website` field until a canonical product-site URL exists.

### Delivery evidence and final acceptance gate

- Frontend production build and cargo-xwin Windows release/NSIS packaging passed. The full native Windows suite passed **111/111**; focused new geometry/caller coverage passed **5/5**. Standards and Spec reviews have no remaining actionable findings. External xwin library missing-PDB `LNK4099` warnings remain.
- Exact installer: `build/DropCove-Setup-0.0.15.exe`, **3,289,290 bytes**, SHA-256 `5ddf9b2676576d8e25ebb79918d9f9fa3de243f51f3fee5586ce9b16bdec0a78`. Exact installed executable: **13,870,592 bytes**, SHA-256 `909b2b4f651c815985cd866fd49df2820a8e436c5bb63d936ff2aecbe6f66bc5`. Earlier provisional 0.0.15 bytes are superseded.
- Real Windows/WebView2 smoke covered native Clear visibility/cancel/confirm/stale-state/rollback, modal guards, More navigation and outside-focus repair, outside Pin All activation, Settings/About singleton reuse, tray About without revealing Shelf, runtime metadata, Source browser opening, two-monitor placement at scale 1, and Dark/Light/forced-color/reduced-motion rendering. Final installed assets loaded from `http://tauri.localhost/` while localhost/Vite access was blocked in the owned WebView; the existing user Vite process was not stopped.
- Silent install, same-candidate reinstall, secondary activation, Cooperative Shutdown with Clear open, restart reference retention, and completed uninstall passed. Owned Profile Data hashes survived uninstall; ambient profile bytes matched the safety snapshot; captured installer registry/shortcut state was restored and owned smoke resources were removed.
- **Startup baseline limitation:** the first baseline launch reconciled HKCU Run before its original bytes were captured. Cleanup restored the subsequently captured installed 0.0.14/default-profile registration; exact original pre-baseline Run restoration cannot be claimed.
- **Historical agent coverage limits:** injected OLE Escape did not establish cancellation and Temporary references were consumed in those attempts, although accepted Copy/Pinned retention and native cancellation-state regressions passed. The user subsequently confirmed all acceptance checks passed. Physical high/mixed-DPI, actual Windows Contrast Theme, screen-reader output, browser-launch failure injection, physical fullscreen/Edge Rail combinations, and literal dev-server-stopped coverage remain unobserved. [Ticket 04](../.scratch/drop-shelf-utility-menu-about/issues/04-release-0.0.15.md) records the now-closed user acceptance gates and preserves these agent-observation limits.

The user confirmed `tất cả đã đạt, hãy commit các thay đổi` on 2026-10-16, closing final human acceptance and explicitly authorizing the commit. This is user-reported acceptance, not additional agent-observed verification.

## 0.0.16 — Drag placement feedback and Drop Shelf fit

**Status:** Completed — the user reported final acceptance and commit of `0.0.16` on 2026-10-20. Historical agent-observation limits below remain unchanged.

**Plan:** [Drag Placement Feedback and Drop Shelf Fit](../.scratch/drag-placement-shelf-fit/spec.md), with [01 Automatic Shelf Sizing](../.scratch/drag-placement-shelf-fit/issues/01-exact-automatic-shelf-sizing.md), [02 Drop Shelf placeholder](../.scratch/drag-placement-shelf-fit/issues/02-drop-shelf-drop-placeholder-feedback.md), [03 Edge Rail placeholder](../.scratch/drag-placement-shelf-fit/issues/03-edge-rail-drop-placeholder-feedback.md), and [04 release](../.scratch/drag-placement-shelf-fit/issues/04-release-dropcove-0.0.16.md).

### Scope

- Add one `48`-logical-pixel Drop Placeholder for each supported external storage drag over the Drop Shelf or expanded Edge Rail. The placeholder represents one prospective Shelf Batch, occupies the newest-first insertion position, uses a dashed drag-hover border and distinct semantic background, and does not preview filenames, thumbnails, or item counts.
- Remove the Drop Shelf's surface-wide drag glow while preserving native Copy cursor feedback and the independent resize-hover accent outline. Keep the Edge Rail free of any additional surface-wide drag highlight.
- Keep drag-hover geometry stable: the Drop Shelf does not preview-resize, the Edge Rail retains its `200 ms` drag-entry expansion delay, and accepted content alone triggers the established surface sizing updates.
- Apply Automatic Shelf Sizing with Drop Shelf height tiers `180` px for 0–2 responsive rows, `230` px for 3 rows, `282` px for 4 rows, and `334` px for 5 or more rows; six or more rows scroll.
- Without a Manual Height Override, recalculate height in both directions after accepted content, width-only resize, Remove Item, Remove Batch, Clear Temporary Items, Successful Drag-Out consumption, and confirmed-Missing cleanup. Automatic shrink keeps the top-left origin fixed.
- Preserve Manual Height Override for height-changing vertical or corner resize while content remains. Width-only resize stays automatic, and final-item removal resets the next opening to `191 × 180`.
- Preserve newest-first Shelf ordering, one-batch-per-drop intake, duplicate and unsupported-input behavior, source-file safety, Edge Rail height tiers, Light/Dark/Windows Contrast presentation, and existing drag-entry/collapse timing.

### Planned verification

- Replace regression expectations for `236/292/348` and no-live-shrink behavior with exact Automatic Shelf Sizing coverage, including `save_size(width, None)` growing and shrinking after responsive column changes while a Manual Height Override continues to win.
- Exercise every row tier and overflow boundary at representative one-, two-, and three-column widths; verify exact window heights, top-left anchoring, final-item reset, and no premature scrollbar.
- Exercise Remove Item, Remove Batch, Clear Temporary Items, Successful Drag-Out consumption, and confirmed-Missing cleanup with and without a Manual Height Override.
- Exercise real Windows OLE drag enter, leave, unsupported payload, application-owned drag, accepted drop, and duplicate-only drop on both Drop Shelf and Edge Rail; verify placeholder position, stable hover geometry, Copy cursor, 200 ms Rail expansion, and absence of surface-wide glow.
- Verify Dark, Light, Windows Contrast, reduced-motion, native frontend build, complete cargo-xwin-produced Windows regression executable, Windows x64 packaging, and exact installed candidate only after implementation acceptance.

### Deferred

- Showing incoming filenames, thumbnails, or item counts before drop acceptance.
- Preview-resizing either surface during drag hover or changing Edge Rail expansion/collapse timing.
- Arbitrary insertion positions, batch reordering, or more than one Drop Placeholder for a single drag operation.

### Observed implementation and release evidence — 2026-10-20

- Exact Automatic Shelf Sizing is derived in the shared Shelf State projection. Pending empty-shelf manual intent remains transient; populated automatic snapshots no longer retain a larger prior height. Removed the obsolete two-batch column-height assertions and added operation-level tier/width/pruning regressions, including non-final item retention and manual-height precedence.
- Real WebView2 measured one-column row counts 1–6 at `180/180/230/282/334/334` pixels. Rows 3–5 had equal scroll content and viewport heights of `160/212/264`; row 6 overflowed at the `334` cap. Six batches at widths `348/516/347` yielded heights `230/180/334`. Kept the canonical 8-pixel collection boundary and reclaimed the redundant 4-pixel footer top padding to eliminate premature overflow.
- Shared 48-pixel dashed Drop Placeholder, transient virtual leading slot, unchanged authoritative collection size, no Drop Shelf-wide glow, and no hover-time native resize were exercised. Real OLE covered supported multi-file input, unsupported text, leave, duplicate-only drops, application-owned self-drop rejection, and exact-byte Pinned Copy retention. Virtualized 66-batch Shelf content retained bounded mounting, three-column layout at width 524, and a reachable final index 65 with 8-pixel bottom clearance.
- Real Left/Right Rail drag entry displayed the placeholder after native expansion without foreground-focus theft. Minimum and maximum expanded sizes remained `280 × 280` and `280 × 642`; leave cleared the marker and returned to the `16 × 280` handle. Accepted and duplicate-only drops preserved the separate Rail sizing/timer contract.
- Complete frontend typecheck/production build and cargo-xwin Windows release/NSIS packaging passed. Final versioned Windows regression executable `dropcove_lib-1ae260f7822f63f3.exe` passed **113/113** directly. External SDK missing-PDB `LNK4099` linker warnings remain nonfatal and unsuppressed.
- Exact installer: `build/DropCove-Setup-0.0.16.exe`, **3,287,614 bytes**, SHA-256 `5002330c008cdde3978857355e9f86f331b968a614d87442303747d4c3605a29`. Exact installed executable: **13,869,568 bytes**, SHA-256 `eeccf4037e0c547aac3cd3181096703ec8315a6b93741e7bea1e6fc3479f77a3`. PE FileVersion was `0.0.16.0`; runtime and uninstall metadata reported `0.0.16`.
- Exact installed assets loaded from `http://tauri.localhost/assets/index-oUXleKGu.js` with the owned Vite service stopped and port 1420 refusing connections. Installed OLE proved Shelf/Rail placeholders, both docking edges, no preview resize, and immediate Shelf shrink `282 → 230 → 180` with unchanged top-left origin. Dark/Light visuals and installed forced-colors/reduced-motion emulation were inspected.
- Silent installation, same-candidate reinstall, secondary activation, restart retention, Cooperative Shutdown, and uninstall all succeeded. Reinstall preserved installed executable identity and owned Profile Data hashes. Uninstall removed the installed directory while preserving owned Profile Data byte-for-byte. Ambient profile and fixture bytes were unchanged; installer registry, shortcut bytes, and the originally captured HKCU Run value were restored exactly. Owned processes, attachments, profiles, fixtures, copies, Vite service, and temporary scaffolding were removed. No owned fixture outputs were present on the Desktop at cleanup.
- **Unobserved/final human gate:** physical high/mixed-DPI, actual Windows Contrast Theme, screen-reader output, physical keyboard/resize gestures, tray/hotkey/Shake and fullscreen combinations were not all re-exercised. Native regressions and media emulation are not physical coverage. The retained installer is ready for user acceptance, not yet accepted; no Git commit was created.
- **Final two-axis review:** independent Standards and Spec reviews of the scoped uncommitted implementation and new `DropPlaceholder.vue` module returned **0 actionable findings per axis**. No source changes were required after the exact candidate was packaged; final human acceptance remains pending.

The user subsequently reported `0.0.16 Ticket 04 đã đạt và đã được commit` on 2026-10-20, closing the human gate and unblocking `0.0.17`. This is user-reported acceptance and commit state, not additional agent-observed verification.

## 0.0.17 — Settings window and About metadata polish

**Status:** Completed — all three tickets resolved; the user confirmed final `0.0.17` acceptance and explicitly authorized committing the changes.

**Plan:** [Settings Window and About Metadata Polish](../.scratch/settings-window-about-metadata/spec.md), with [01 Taskbar-visible, invocation-centered Settings](../.scratch/settings-window-about-metadata/issues/01-taskbar-visible-invocation-centered-settings.md), [02 Cargo-owned About author and description](../.scratch/settings-window-about-metadata/issues/02-cargo-owned-about-author-description.md), and [03 release](../.scratch/settings-window-about-metadata/issues/03-release-dropcove-0.0.17.md). All three tickets are resolved.

### Scope

- Create Settings as a singleton decorated top-level window that is visible on the Windows taskbar and remains independent of Drop Shelf visibility. The Resident Instance continues to own application shutdown and Settings cleanup.
- Center Settings within the invoking monitor's work area on first creation: use the Drop Shelf monitor for a Drop Shelf request and the pointer monitor for a notification-area request, with primary-monitor fallback. Repeated requests show and focus the existing window without repositioning it.
- Change `Cargo.toml` package metadata to `authors = ["HungNth"]` and `description = "A resident Windows utility providing a temporary drag-and-drop workspace for file and folder references."`.
- Project the Cargo package description through the existing authoritative About metadata command and display it directly below the runtime version in the About Surface. Do not duplicate the description in Vue or Tauri configuration.
- Preserve Settings Save/Cancel, shortcut recording, startup, Edge Rail, Shake-to-open, singleton, and Resident Instance lifecycle behavior.

### Planned delivery slices

1. Settings presentation: invocation-aware initial centering, taskbar visibility, singleton reuse, and Resident Instance cleanup.
2. Package and About metadata: Cargo author/description projection and About Surface presentation.
3. Release gate: synchronized `0.0.17` manifests, Windows x64 regression checks, cargo-xwin/NSIS packaging, installed-app smoke, and human acceptance.

### Planned verification

- Exercise first opening from both Drop Shelf and notification-area entry points, including available secondary-monitor placement, complete outer-frame work-area bounds, taskbar presence, repeated-request focus without movement, and independent Drop Shelf hide/show behavior.
- Exercise Settings Save, Cancel, shortcut recording, close, singleton reuse, and Exit DropCove cleanup after the ownership change.
- Verify runtime About metadata reports author `HungNth` and the exact Cargo description, with the description rendered immediately below the synchronized runtime version without a frontend copy.
- Complete version synchronization, the native Windows regression suite, frontend production build, cargo-xwin Windows release/NSIS packaging, installed-candidate lifecycle smoke, and final human acceptance before marking `0.0.17` complete.

### Deferred

- Remembering or restoring a user-moved Settings position.
- User-configurable Settings placement or centering policy.
- Changing taskbar behavior for Drop Shelf, Edge Rail, Manage Items, or confirmation surfaces.

### Observed implementation and release evidence — 2026-10-20

- Shared invocation-context placement now serves Settings and About. Settings is a decorated top-level singleton, taskbar-visible with hidden/EdgeDocked Shelf, initially centered from the complete native outer frame. Actual primary/secondary Drop Shelf invocation, native tray routing, moved-position reuse, minimized restore, Save/Cancel/native close, recorder focus-loss cancellation, startup/Shake/fullscreen controls, share-denied Save rollback, and Cooperative Shutdown were exercised.
- Cargo authors are `HungNth`; the package description is projected through the existing About metadata command and rendered directly below the runtime version. Exact installed About reports `0.0.17`; its complete two-line description fits the fixed `400 × 460` surface without scrolling. Dark/Light and emulated forced-colors/reduced-motion rendering were inspected.
- Frontend production typecheck/build, cargo-xwin release/NSIS packaging, and the fresh Windows native test executable passed. Complete suite: **114/114**. External SDK missing-PDB `LNK4099` warnings remain nonfatal and unsuppressed.
- Installer: [DropCove-Setup-0.0.17.exe](../build/DropCove-Setup-0.0.17.exe), **3,291,898 bytes**, SHA-256 `9488308f7c441a235a79eff9e0eeef739e28360ab0b658248eaa975d523f4973`. Installed executable: **13,877,760 bytes**, SHA-256 `e4a485be4cce67a3b029201bfa361b43908a8cab8bcebc023c4a70a66a93228b`. Runtime/uninstall version and synchronized manifests report `0.0.17`. Installed assets loaded from `http://tauri.localhost/` while port 1420 refused connections.
- Silent install, same-candidate reinstall, secondary activation, exact Pinned-reference restart retention, Cooperative Shutdown, and uninstall passed. An initial isolated `/NS` reinstall was rejected because the ambient shortcut targeted the original installation; temporarily protecting/restoring its exact bytes allowed the same candidate to reinstall successfully. Uninstall preserved owned Profile Data and source bytes and removed the executable. Owned smoke resources were removed; ambient executable/profile and shortcut hashes remained unchanged, and captured uninstall registry values plus HKCU Run were restored.
- **Registry capture limitation:** the actual manufacturer key `Software\dropcove\DropCove` was identified after installation. Its default path was restored from captured pre-install uninstall metadata, retaining observed language `1033`; exact original bytes for this key were not independently captured. See [Ticket 03](../.scratch/settings-window-about-metadata/issues/03-release-dropcove-0.0.17.md) for the complete evidence and final human gate.
- **Historical agent-observation limits:** physical high/mixed-DPI, tray invocation on the secondary display (host pointer constrained to primary), actual Windows Contrast Theme, screen-reader output, and complete physical shortcut/monitor-selector/Source-browser handoff were not all re-exercised. The user subsequently accepted all checks; this does not reclassify those limits as agent-observed coverage.
- Independent two-axis review of the final scoped implementation against baseline `a1f677cbb1941c28c27d6506d38d8467c2ba6781` returned **0 Standards findings** and **0 Spec findings**; no source changes were required after packaging the exact candidate.
- The user confirmed `tất cả đã đạt, hãy commit các thay đổi`, closing final human acceptance and explicitly authorizing the commit. The disclosed registry capture limitation remains preserved.
