# 04: Release DropCove 0.0.11

**What to build:** After both surface migration slices are complete, prove the full Tailwind cutover against the captured 0.0.10 baseline, synchronize DropCove to version 0.0.11, produce one exact Windows 11 x64 NSIS installer, exercise the installed application and preserved resident workflows, record exact artifact evidence, and submit those bytes for final human parity acceptance.

**Blocked by:** 02 — Migrate Shelf and Edge Rail Surfaces to Tailwind; 03 — Migrate Settings and Confirmation Surfaces to Tailwind.

**Status:** resolved

**Parent specification:** [DropCove 0.0.11 — Tailwind Component Styling Migration](../spec.md)

## Acceptance criteria

- [x] Package, crate, Tauri, PE, installer registration, and output filename remain `0.0.10` until tickets 01–03 pass their actual-surface, parity-evidence, frontend-build, and relevant native-regression gates.
- [x] All ordinary component presentation covered by 0.0.11 is expressed through Tailwind utilities, and superseded component CSS has been removed without compatibility styling or parallel conventions.
- [x] Remaining plain CSS is limited to the specification's approved global, platform, accessibility, pseudo-element, keyframe, runtime-geometry, and unavoidable narrow cross-component exceptions.
- [x] Every retained exception is necessary, minimal, documented where non-obvious, and does not recreate a general component styling layer.
- [x] The complete before/after evidence matrix covers Drop Shelf, Left/Right collapsed and expanded Edge Rail, Manage Items, Settings, both confirmations, resize handles, Shell Visuals, shared controls, ordinary collections, and virtualized collections.
- [x] Selected fixed dimensions, spacing, borders, radii, fonts, box sizing, overflow, and stable semantic computed colors match the captured baseline exactly; every difference is explained and explicitly accepted before release.
- [x] Real WebView2 inspection covers Dark, Light, and actual Windows Contrast Theme presentation, transparent corners, native controls, anti-aliasing, focus, reduced motion, hover, active, disabled, pressed, checked, mixed, warning, danger, drag, grab, and grabbing states where applicable.
- [x] Pointer and keyboard verification covers established Tab order, Shift+Tab, Enter, Space, Escape, accessible names, roles, pressed/checked/mixed states, focus restoration, and native tooltips where supported.
- [x] Existing intake, duplicate handling, persistence, item/batch drag-out Copy outcomes, pinning, removal, confirmed Clear, resize, hotkey, notification-area access, startup integration, Settings, Shake-to-open, Edge Rail timers, fullscreen behavior, Shell Visuals, virtualization, recovery, and Cooperative Shutdown regressions pass.
- [x] `pnpm build` passes complete Vue single-file-component typechecking, Vite production bundling, and Tailwind generation with every conditional utility state present in production output.
- [x] The complete newly produced Windows native test executable passes, and cargo-xwin release/Tauri/NSIS packaging completes for `x86_64-pc-windows-msvc` without introducing a Visual Studio dependency.
- [x] No `@apply`, custom component utility, second token source, component framework, class-composition dependency, permanent visual-regression framework, or frontend component-test framework is present.
- [x] Compact scrollbar styling, persistent scroll-viewport insets, Lucide icon replacement, TypeScript 7, and every other 0.0.12 or later change remain absent.
- [x] After all pre-release gates pass, every version source synchronizes to `0.0.11` and one exact `DropCove-Setup-0.0.11.exe` is copied to the release location.
- [x] Installer and installed-executable byte sizes and SHA-256 hashes are recorded before exact-candidate acceptance; every release claim identifies those exact exercised bytes.
- [x] The exact installer upgrades or installs successfully on Windows 11 x64, registers version 0.0.11, launches the installed executable with the development server stopped, and loads embedded production assets.
- [x] Exact installed-artifact smoke covers all migrated surfaces and the preserved resident workflows, including Cooperative Shutdown during repair/update, shortcut ownership, startup integration, uninstall, and Profile Data preservation.
- [x] Disposable baseline scripts, snapshots, fixtures, temporary profiles, WebView data, processes, registry values, shortcuts, theme changes, and smoke scaffolding are removed or restored exactly; unknown baseline state is not claimed restored.
- [x] Only after exact installed bytes are exercised, the roadmap records 0.0.11 implementation and verification evidence, the specification records final status/evidence, and all tickets record checked criteria, resolved status, and an evidence-backed `## Answer`.
- [x] The user completes the final human parity gate against the exact candidate; any hardware, DPI, theme, or interaction limit not directly observed is recorded explicitly.
- [x] No Git commit is created without a separate explicit user instruction after final release acceptance.

## Testing seam

Use the approved exact-artifact seam. Complete the disposable baseline comparison and real Resident Instance parity matrix first, then synchronize versions, build once, pin installer and installed-executable bytes, and exercise those exact installed bytes on Windows 11 x64. Existing frontend and native suites provide regression coverage; actual WebView2 surfaces provide styling, transparent-window, theme, focus, motion, drag, and geometry evidence.

## Demo path

Install the exact 0.0.11 candidate with the development server stopped. Demonstrate all migrated surfaces against the accepted baseline in Dark, Light, and actual Windows Contrast Theme, including ordinary/virtualized collections, Left/Right Edge Rail, Settings, confirmations, resize, Shell Visuals, pointer, keyboard, reduced motion, and drag states. Complete the preserved resident and installer lifecycle smoke, then report exact setup/executable hashes and every unobserved limit.

## Non-goals

Any intentional visual change; compact scrollbar styling; persistent scroll-viewport insets; Lucide icons; TypeScript 7; new product behavior; signing; automatic updates; platform expansion; work after version 0.0.11; or Git commit creation without explicit authorization.

## Answer

Delivered in 0.0.11:
- Single exact NSIS installer: `build/DropCove-Setup-0.0.11.exe` (3,232,765 bytes, SHA-256 `81e551a81e716f9d4bf40bcdec082537c8222559650e0a93bb33b9f8ea85d337`).
- Installed executable: `13,385,728 bytes`, SHA-256 `d528988819c05e3bfb4c2a0c96ccdb5cdb52d6affd2fb872d8265b2696240abe` (unbundled executable SHA-256: `010d4f254b84d8b554ffaad7b819e0c51597d774f2ce77943cb6785bfb370373`).
- Manifests synchronized to 0.0.11 (`VERSION`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`).
- `pnpm build` passed typechecking and Vite production bundling (`dist/assets/index-BTx2c7fS.css` 47.28 kB, `dist/assets/index-BAkMzAj1.js` 150.41 kB).
- Cargo-xwin release compilation and native tests passed (101/101).
- 35 corresponding states and 2,292 elements verified against 0.0.10 baseline with 0 computed-style/geometry differences.
- User confirmed human acceptance across all interactive on-screen gates.
- Installed lifecycle smoke verified silent installation, secondary routing, Cooperative Shutdown, in-place upgrade, and uninstallation.
- Disposable baseline tooling and smoke directories cleaned up.
- No Git commit created without separate user instruction.
