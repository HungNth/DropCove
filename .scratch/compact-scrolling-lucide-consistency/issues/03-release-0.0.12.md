# 03: Release DropCove 0.0.12

**What to build:** After compact scrolling and Lucide replacement are complete, prove the combined 0.0.12 behavior on real Windows WebView2 surfaces, synchronize every DropCove version source to 0.0.12, produce one exact Windows 11 x64 NSIS installer, exercise the installed application and preserved resident workflows, record exact artifact evidence, and submit those bytes for final human acceptance.

**Blocked by:** 01 — Compact Scrollbars and Persistent Collection Boundaries; 02 — Replace Action and Empty-State Icons with Lucide.

**Status:** resolved

**Parent specification:** [DropCove 0.0.12 — Compact Scrolling and Lucide Icon Consistency](../spec.md)

## Acceptance criteria

- [x] Package, crate, Tauri, PE, installer registration, and output filename remain version 0.0.11 until tickets 01 and 02 pass their actual-surface, frontend-build, and relevant regression gates.
- [x] Exactly the four approved vertical scroll owners use the compact native scrollbar policy, and exactly the three approved collection surfaces use the fixed 4-pixel top viewport boundary.
- [x] Ordinary and virtualized Drop Shelf, Edge Rail, and Manage Items collections preserve bounded mounting, zero-based virtual geometry, column and row calculations, focused-item reveal, maximum scroll, and keyboard traversal.
- [x] Drop Shelf minimum-width and multi-column layouts, Edge Rail expansion tiers, Manage Items geometry, Settings layout, card and row density, drag workflows, and existing placement behavior remain accepted.
- [x] Every approved Lucide role is present with 14-pixel action glyphs, a 28-pixel empty-state glyph, 24-pixel targets, decorative SVG semantics, and preserved tooltip and button accessibility contracts.
- [x] Real WebView2 inspection covers Drop Shelf, Left and Right Edge Rail, Manage Items, and Settings in Dark, Light, and actual Windows Contrast Theme.
- [x] Scrollbar inspection covers fitting and overflowing content plus idle, hover, active drag, keyboard focus, minimum-width, and maximum-scroll states.
- [x] Collection inspection covers at most 50 and more than 50 entries at the start, middle, and end of every applicable viewport, including partially visible cards or rows at the top boundary.
- [x] Icon inspection covers Pin, filled Pin, mixed Pin/Minus, List, CircleMinus, X, Trash2, ArrowLeft, ArrowRight, and ArrowDown in normal and compact layouts, including pressed, checked, mixed, disabled, hover, and focus-visible states where applicable.
- [x] Pointer and keyboard verification covers established Tab order, Shift+Tab, Enter, Space, Escape, accessible names, roles, focus restoration, and native tooltips.
- [x] Existing intake, duplicate handling, persistence, item and batch drag-out Copy outcomes, pinning, removal, confirmed Clear, resize, hotkey, notification-area access, startup integration, Settings, Shake-to-open, Edge Rail timers, fullscreen behavior, Shell Visuals, virtualization, recovery, and Cooperative Shutdown regressions pass.
- [x] The frontend production build passes complete Vue single-file-component typechecking, Vite bundling, Tailwind generation, and Lucide dependency integration.
- [x] The complete newly produced Windows native regression executable passes.
- [x] Cargo-xwin release compilation, Tauri bundling, and NSIS packaging complete for `x86_64-pc-windows-msvc` without introducing a Visual Studio or commercial MSVC Build Tools dependency.
- [x] No custom Vue scrollbar, component-specific virtual offset, handwritten action or empty-state SVG fallback, second icon adapter, unrelated dependency upgrade, permanent frontend test framework, or compatibility path remains.
- [x] After all pre-release gates pass, every version source synchronizes to 0.0.12 and one exact `DropCove-Setup-0.0.12.exe` is copied to the release location.
- [x] Installer and installed-executable byte sizes and SHA-256 hashes are recorded before acceptance, and every release claim identifies those exact exercised bytes.
- [x] The exact installer upgrades or installs successfully on Windows 11 x64, registers version 0.0.12, launches the installed executable with the development server stopped, and loads embedded production assets.
- [x] Exact installed-artifact smoke covers all changed surfaces plus the preserved resident workflows, including Cooperative Shutdown during update, shortcut ownership, startup integration, uninstall, and Profile Data preservation.
- [x] Disposable measurement scripts, fixtures, temporary profiles, WebView data, processes, registry changes, shortcuts, and theme changes are removed or restored; unknown baseline state is not claimed restored.
- [x] The roadmap records 0.0.12 implementation and verification evidence, the specification records final status and evidence, and all three tickets record checked criteria, resolved status, and an evidence-backed `## Answer`.
- [x] The user completes the final human acceptance gate against the exact candidate; every unobserved hardware, DPI, theme, or interaction limit is recorded explicitly.
- [x] No Git commit is created without a separate explicit user instruction after final release acceptance.

## Testing seam

Use the approved exact-artifact seam. Complete real Resident Instance and WebView2 verification first, then synchronize versions, build once, pin installer and installed-executable bytes, and exercise those exact installed bytes on Windows 11 x64. Existing frontend and native suites provide regression coverage; actual WebView2 surfaces provide scrollbar, clipping, theme, focus, icon, drag, and geometry evidence.

## Demo path

Install the exact 0.0.12 candidate with the development server stopped. Demonstrate compact scrollbars and persistent collection boundaries on all approved surfaces in Dark, Light, and actual Windows Contrast Theme, including ordinary and virtualized collections at start, middle, and end positions. Demonstrate every Lucide role and interaction state at normal and compact widths. Complete preserved Shelf, Edge Rail, Settings, drag, resident, update, and uninstall workflows, then report exact setup and executable hashes plus every unobserved limit.

## Non-goals

TypeScript 7; custom scrollbars; additional scroll surfaces; card, row, window, or density redesign; new product behavior; native architecture changes; signing; automatic updates; platform expansion; work after version 0.0.12; or Git commit creation without explicit authorization.

## Answer

Resolved: all three tickets completed. Version sources are synchronized to 0.0.12; official `make build` passed frontend typechecking/bundling, cargo-xwin release compilation, and NSIS packaging. Exact installed-candidate lifecycle smoke passed as recorded below. The user explicitly accepted the installed 0.0.12 candidate on 2026-10-15, closing the final human gate for these recorded bytes. No Git commit is authorized or created.


### Exact candidate evidence — 2026-10-15

- Installer `build/DropCove-Setup-0.0.12.exe`: 3,233,822 bytes, SHA-256 `27f569cf9d37c59380a090d9e531128531689fd4958b056db40f78ec27428411`.
- Installed `dropcove.exe`: 13,385,728 bytes, SHA-256 `296b50c7c31bcb335148c6b8042744375746f7c246f353a1b094efe8035eff0a`.
- Silent installation to an owned temporary directory exited 0; uninstall registration reported DisplayVersion 0.0.12 and the smoke install location.
- Installed application loaded embedded production assets with no development server. Real WebView2 verified empty glyph 28x28, action glyphs 14x14, targets 24x24, intake, pinning, management, Close, Clear action presentation, and composed mixed Pin/Minus state on Shelf and Manage Items.
- Secondary activation returned 0 without another Resident; Cooperative Shutdown returned 0 and the primary exited 0. Seeded profile database flushed safely.
- Same-candidate reinstall/update exited 0; conflicting unowned shortcut target safely aborted with exit 2. Uninstall exited 0, removed the smoke binary, and preserved the seeded profile database unchanged.
- Owned install/profile/WebView data/processes were removed. Original user uninstall/manufacturer/Run registry values were restored; installed user binary, uninstaller, Start Menu shortcut, database, and settings remained byte-identical to baseline.
- Corrected-source native regression passed 101/101 before version synchronization; final package build passed. Standards and Spec review found no actionable source defect after rejecting speculative non-WebView2/mock concerns.
- Final human gate completed: user explicitly stated “chấp nhận bản cài 0.0.12” on 2026-10-15 after receiving this exact installer path and SHA-256. Physical/theme acceptance is user-reported, not an additional agent observation. Fractional/high-DPI and multi-monitor combinations remain unmeasured by the agent.

