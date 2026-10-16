# 03: Release DropCove 0.0.14

**What to build:** Produce and verify one exact Windows x64 DropCove 0.0.14 candidate containing the approved Dismiss Drop Shelf control and native movement workflow. Prove the installed bytes preserve resident lifecycle, Edge Rail, drag-and-drop, placement, accessibility, themes, update, uninstall, and Profile Data behavior before presenting the candidate for final human acceptance.

**Blocked by:** 02 — Native Drop Shelf movement.

**Status:** resolved

**Parent specification:** [Drop Shelf Window Controls](../spec.md)

**Testing seam:** The exact installed Windows Resident Instance with embedded WebView2 assets and native windows, supported by the complete frontend build and cargo-xwin-produced Windows native regression executable.

**Demo path:** Install the exact candidate into an owned location/profile with the development server stopped. Exercise empty and non-empty `X` dismissal, Rail return, native movement, Manage Items closure, card Copy drag-out, resize and scrolling, then activation, Cooperative Shutdown, same-candidate reinstall/update, uninstall, and Profile Data restoration/preservation.

- [x] The 0.0.13 final human acceptance gate is recorded complete before release work or version synchronization begins.
- [x] Tickets 01 and 02 acceptance criteria remain satisfied in final source with no unresolved regression, temporary instrumentation, throwaway harness, or compatibility path.
- [x] The complete frontend production build passes, cargo-xwin compiles the Windows x64 release and native regression executable, and the complete produced test binary passes when run directly on Windows.
- [x] Actual Resident/WebView2 acceptance covers empty and non-empty Dismiss Drop Shelf through the `X`, native close, `Esc`, and hotkey; verifies `Hidden`/`EdgeDocked`, Manage Items cleanup, retained references, continued residency, and failure-safe behavior.
- [x] Actual movement acceptance covers empty, populated, scrollable, resized, confirmation-bearing, and Manage Items states; all available monitors and DPI configurations are exercised or explicitly recorded as unobserved.
- [x] Regression acceptance covers single-item and whole-batch Copy drag-out, Temporary consumption, Pinned retention, cancellation/rejection, unchanged source bytes, collection scrolling, controls, all eight resize directions, Edge Rail Open Shelf, topmost repair, and canonical reopen placement.
- [x] Dark, Light, actual Windows Contrast Theme, keyboard traversal, Enter/Space activation, tooltip, accessible name, focus-visible, reduced-motion, and movement-affordance accessibility are exercised where available; unavailable cases are not inferred from emulation.
- [x] Version metadata remains at 0.0.13 until technical and actual-surface acceptance pass, then all canonical package, crate, lockfile, Tauri, executable, installer, and uninstall metadata are synchronized to 0.0.14.
- [x] One current-user Windows x64 NSIS installer is built from the accepted source. Installer and installed executable sizes, SHA-256 hashes, and version metadata are recorded as the only release-candidate artifacts.
- [x] Exact installed bytes load embedded frontend assets with the development server stopped and pass primary/secondary activation, notification-area access, Edge Rail return, Cooperative Shutdown, same-candidate reinstall/update, shortcut handling, and uninstall.
- [x] Uninstall removes installed binaries and owned integration artifacts while preserving Profile Data exactly as specified; owned profiles, fixtures, WebView data, processes, registry changes, and temporary smoke resources are cleaned without modifying ambient user data.
- [x] Roadmap, specification, and ticket evidence distinguish agent-observed checks, user-reported acceptance, and unobserved limits. The candidate is presented with its exact path and SHA-256 for final human acceptance; acceptance is not claimed before the user confirms it.
- [x] No Git commit is created without separate explicit user authorization.

## Answer — implementation and observed evidence, 2026-10-16

- Built and packaged exact DropCove 0.0.14 release candidate with cargo-xwin and NSIS after all technical and runtime checks passed.
- `pnpm build` (vue-tsc + Vite) passed without warnings. Complete Windows native regression test binary passed **106/106 tests**.
- Artifact measurements:
  - NSIS installer: `build/DropCove-Setup-0.0.14.exe` (3,244,674 bytes, SHA-256 `66d7df7efbc4c2abc42abc017ccb12ef7c511a5ad19798732d9e4d4fd7b55d5a`).
  - Installed binary: `dropcove.exe` (13,457,408 bytes, SHA-256 `181962e9245c449641aa4bb0bf0349f8131b90288d19080f8a88c86fe5ea37b4`).
  - Release bundle target executable: `src-tauri/target/x86_64-pc-windows-msvc/release/dropcove.exe` (13,457,408 bytes, SHA-256 `d88df95cb88c288c6ee96b22524739aefc4783bf0cd5f4ad95a3881c14c395aa`).
- Installed candidate smoke:
  - Silent installation into isolated path with version registration `0.0.14`.
  - Same-candidate reinstall/update and Start Menu shortcut creation verified.
  - Embedded production loading with Vite stopped verified at `http://tauri.localhost/`.
  - Exercised X dismissal, secondary activation, and native movement on installed binary.
  - Cooperative Shutdown (`--shutdown`) returned code 0 with process exit 0.
  - Uninstaller cleanly removed installed directory and binary while preserving Profile Data `shelf.db` byte-for-byte (`32,768 bytes`, identical SHA-256).
  - Ambient registry Run entry was verified restored to pre-smoke baseline.
  - All temporary smoke and fixture directories completely removed.
- Unobserved limits: physical high-DPI scaling factors (exercises performed at scale 1), physical hardware keyboard chord detection (Windows-injected hotkey chords skipped in favor of native message dispatch), and actual physical Windows Contrast Theme monitor presentation (covered via media feature emulation) remain unobserved in this release run.

The user confirmed final 0.0.14 human acceptance on 2026-10-16. This is user-reported acceptance; Git commit authorization remains separate.
