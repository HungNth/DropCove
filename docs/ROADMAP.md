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

**Status:** In progress — tickets 01 and 02 resolved. Tickets 03–06 remain unimplemented.

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

Before completion:

- Exercise temporary, pinned, and mixed batches through item drag, batch drag, bulk pinning, explicit removal, and Clear Temporary Items.
- Restart after each durable mutation and observe the same lifecycle state.
- Exercise pointer and keyboard Manage Items flows, including focus restoration and active-drag dismissal guards.
- Resize horizontally and vertically and observe automatic-height versus manual-height persistence rules.
- Build and launch `DropCove-Setup-0.0.3.exe` on Windows 11 x64.

### Deferred

- Resident process access, Edge Rail, Shake-to-open, advanced shell visuals, and recovery/distribution hardening.

## 0.0.4 — Resident access and settings

**Status:** Planned.

### Scope

- Add the profile-specific single-instance guard and activation signal; a secondary launch shows the existing Drop Shelf and exits.
- Add the notification-area icon, Settings and Exit commands, warning notifications, and resident lifetime independent of the Drop Shelf window.
- Add global hotkey registration and recording, atomic settings persistence, Start with Windows registration, hidden `--autostart`, and rollback-safe Save/Cancel behavior.
- Add cursor-monitor placement for normal launch, activation, tray, and hotkey; Per-Monitor V2 DPI conversion; work-area clamping; and topmost repair without activation.
- Preserve safe operation when hotkey, startup registration, or other optional integration setup fails.

### Implemented

- Not started.

### Verification

Before completion:

- Launch a second process and observe activation of the existing resident process without duplicate shelf state.
- Exercise tray, hotkey, normal launch, settings Save/Cancel, rejected hotkeys, and sign-in hidden start.
- Restart with valid, missing, malformed, and unreadable settings and observe the documented defaults and safe retained configuration.
- Move between monitors with different DPI and observe correct placement, bounds, and topmost behavior.
- Build and launch `DropCove-Setup-0.0.4.exe` on Windows 11 x64.

### Deferred

- Non-empty dismissal transitions to the full Edge Rail workflow in `0.0.5`; Shake-to-open and final hardening remain later.

## 0.0.5 — Edge Rail workflow

**Status:** Planned.

### Scope

- Add `EdgeDocked` display behavior, Rail Handle, delayed expansion/collapse, bounded growth, scrolling, and immediate hiding after the final Shelf Item is removed.
- Support drop-in, item and batch drag-out, Manage Items, pinning, removal, Clear Temporary Items, and Open Shelf without activating DropCove.
- Apply the configured edge, configured-monitor precedence, remembered dismissal monitor, deterministic primary-display fallback, and disconnected-monitor preservation rules.
- Add the geometric fullscreen policy and foreground tracking needed to hide and restore the requested rail.
- Complete display-state transitions and non-empty dismissal behavior across Drop Shelf, Edge Rail, settings changes, display changes, and restart.

### Implemented

- Placement ownership and fallback are documented in [ADR-0002](adr/0002-edge-rail-placement-ownership.md); runtime implementation has not started.

### Verification

Before completion:

- Dismiss empty and non-empty shelves and observe `Hidden` and `EdgeDocked` respectively.
- Exercise every rail action without foreground activation and verify expansion/collapse timing and size tiers.
- Disconnect and reconnect configured and remembered monitors; observe primary-display fallback without settings loss.
- Cover the selected monitor with a fullscreen foreground window and verify both fullscreen-policy settings.
- Build and launch `DropCove-Setup-0.0.5.exe` on Windows 11 x64.

### Deferred

- Shake-to-open, final visual-resource work, full accessibility conformance, database recovery, and installer lifecycle hardening.

## 0.0.6 — Shake-to-open

**Status:** Planned.

### Scope

- Add the low-level mouse hook, minimal callback capture, bounded ordered queue, axis-reversal detector, sensitivity profiles, detection window, and cooldown.
- Add the Shake session coordinator that saves `Hidden` or `EdgeDocked` state, shows the Drop Shelf near the pointer, and restores prior state after release, rejection, unsupported input, duplicate-only input, cancellation, or false positive.
- Keep the shelf visible near the pointer only after a new Shelf Batch is accepted.
- Apply settings changes and rollback behavior when enabling, disabling, or reconfiguring the hook.
- Preserve all non-Shake workflows when hook installation or processing fails.

### Implemented

- Not started.

### Verification

Before completion:

- Exercise Low, Normal, and High sensitivity at their documented movement and reversal thresholds.
- Verify the `600 ms` detection window, `500 ms` cooldown, left-button requirement, placement near the pointer, and each restoration outcome.
- Force hook registration failure and observe a warning while tray, hotkey, Drop Shelf, and Edge Rail remain usable.
- Build and launch `DropCove-Setup-0.0.6.exe` on Windows 11 x64.

### Deferred

- Final shell visuals, accessibility conformance, recovery, and distribution hardening.

## 0.0.7 — Presentation and accessibility conformance

**Status:** Planned.

### Scope

- Add asynchronous shell file/folder icons and image thumbnails with fallbacks, bounded previews, cancellation, and resource release tied to surface lifetime.
- Add virtualization for batch collections and Manage Items lists and clear full-item projections when management surfaces close.
- Complete Light, Dark, and High Contrast resources, visible focus behavior, accessibility names/states, live announcements, and full-path accessibility text.
- Add short state-change animations and disable non-essential motion when Windows client-area animations are disabled.
- Validate that earlier workflows remain fully operable by keyboard and without thumbnails or animation.

### Implemented

- Not started. Basic accessibility for controls remains part of every earlier version rather than being postponed here.

### Verification

Before completion:

- Exercise all Drop Shelf, Manage Items, Settings, and Edge Rail actions using keyboard navigation and accessibility inspection.
- Switch among Light, Dark, and High Contrast and disable client-area animations without losing behavior or legibility.
- Open and close large collections while loading visuals and observe bounded previews, virtualization, cancellation, and released resources.
- Build and launch `DropCove-Setup-0.0.7.exe` on Windows 11 x64.

### Deferred

- Recovery and distribution hardening in `0.0.8`.

## 0.0.8 — Recovery and distribution hardening

**Status:** Planned.

### Scope

- Add SQLite integrity checking and recoverable database failure handling, including timestamped preservation of the database and matching WAL/SHM sidecars, clean recreation, and user notification with the backup path.
- Complete persistence-failure rollback and safe-state behavior across Shelf mutations, settings changes, hotkey registration, startup registration, and Shake integration.
- Complete the current-user installer wizard, Visual C++ Redistributable prerequisite check, Start Menu and optional Desktop shortcuts, remembered install directory, and Finish-page launch option.
- Complete update and uninstall coordination with the resident process, bounded shutdown wait, abort behavior, startup-registration cleanup, and profile-data preservation.
- Run the complete target-behavior acceptance pass across Windows 11 x64, multi-monitor/DPI scenarios, restart boundaries, and documented integration failures.

### Implemented

- Not started.

### Verification

Before completion:

- Corrupt or obstruct the shelf database and observe preserved evidence, clean recovery, empty startup, and a warning containing the backup path.
- Inject persistence and integration failures and observe no user-visible partial mutation or silent loss of the prior safe configuration.
- Install, update, and uninstall through the documented wizard and resident-process shutdown paths; verify profile preservation and startup-registration removal.
- Exercise the end-to-end example in `SPECIFICATION.md` and every target requirement assigned to `0.0.2` through `0.0.8`.
- Build and launch `DropCove-Setup-0.0.8.exe` on Windows 11 x64.

### Deferred

- No target requirement is intentionally deferred beyond `0.0.8`. Advancing to another version requires a new approved scope; advancing to `0.1.0` or `1.0.0` requires explicit approval.
