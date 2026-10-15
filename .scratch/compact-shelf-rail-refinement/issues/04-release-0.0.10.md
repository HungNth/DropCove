# 04: Release DropCove 0.0.10

**What to build:** After all compact presentation slices are proven in the real Windows application, synchronize DropCove to version 0.0.10, produce one exact Windows 11 x64 installer, verify the installed application and preserved resident workflows, record artifact evidence, and submit those exact bytes for final visual, accessibility, and native interaction acceptance.

**Blocked by:** 03 — Mirrored Compact Edge Rail.

**Status:** resolved

**Human gate:** User confirmed all outstanding exact-candidate acceptance tests passed on 2026-10-15. Git commit authorization remains separate.

**Parent specification:** [DropCove 0.0.10 — Compact Drop Shelf and Edge Rail UI Refinement](../spec.md)

## Acceptance criteria

- [x] Package, crate, Tauri, PE, installer registration, and output filename remain `0.0.9` until tickets 01–03 pass their actual-surface, frontend-build, and native-regression gates.
- [x] `pnpm build` passes complete Vue single-file-component typechecking and production bundling with the approved 0.0.9 dependency graph unchanged.
- [x] The complete newly produced Windows native test executable passes, and cargo-xwin release/Tauri/NSIS packaging completes for `x86_64-pc-windows-msvc` without introducing a Visual Studio dependency.
- [x] Before version synchronization, the running application passes the 0.0.10 matrix for Drop Shelf, Manage Items, collapsed Edge Rail, and expanded Edge Rail across Dark, Light, and Windows Contrast presentation.
- [x] The presentation matrix covers empty/populated, Left/Right, collapsed/expanded, normal/hover/focus/disabled/error, temporary/pinned/mixed, short/virtualized/scrolling, confirmation, management flyout, drag-over, and resize states.
- [x] Pointer and keyboard verification covers every compact icon through hover, native tooltip, Tab, Shift+Tab, Enter, Space, accessible name, binary pressed state, tri-state checked state, Escape, and focus restoration where the established surface supports keyboard input.
- [x] Existing intake, duplicate handling, persistence, item/batch drag-out Copy outcomes, pinning, removal, confirmed Clear, resize, hotkey, tray, startup, Settings, Shake-to-open, Edge Rail timers, fullscreen behavior, Shell Visuals, and virtualization regressions pass.
- [x] After all pre-release gates pass, every version source synchronizes to `0.0.10` and one exact `DropCove-Setup-0.0.10.exe` is copied to the release location.
- [x] Setup and installed executable byte sizes and SHA-256 hashes are recorded before installed-artifact acceptance; all release claims identify those exact exercised bytes.
- [x] The exact installer upgrades or installs successfully on Windows 11 x64, registers version 0.0.10, launches the installed executable, loads embedded production assets, and preserves current-user installation behavior.
- [x] Exact installed-artifact smoke covers the refined surfaces and all resident workflows, including Cooperative Shutdown during repair/update, shortcut ownership, startup integration, uninstall, and Profile Data preservation.
- [x] Ambient processes, profiles, registry values, shortcuts, theme/Contrast settings, startup integration, WebView data, and fixtures changed during smoke are restored or removed exactly; unknown baseline state is not claimed restored.
- [x] Only after the exact installed bytes are exercised, `docs/ROADMAP.md` records the 0.0.10 Implemented/Verification evidence, the release specification records final status and exact-candidate evidence, and every ticket records its checked criteria, resolved status, and exercised `## Answer`; no unobserved release claim is published.
- [x] TypeScript 7 and every product release after 0.0.10 remain deferred.
- [x] No Git commit is created without a separate explicit user instruction after release acceptance.

## Testing seam

Use the approved exact-artifact seam. Prove the presentation first in the real Resident Instance, then build once, pin installer and installed executable bytes, and exercise those exact installed bytes on Windows 11 x64. Existing frontend and native suites provide regression coverage; actual surfaces provide corner, icon, keyboard, accessibility, theme, and no-activate evidence.

## Demo path

Install the exact 0.0.10 candidate, launch its embedded application, and demonstrate the compact Drop Shelf, Manage Items, Left/Right collapsed and expanded Edge Rail, icon controls, confirmation flows, keyboard semantics, and all three presentation modes. Complete native intake/drag-out and resident integration smoke, perform repair/update and uninstall checks, prove Profile Data preservation, and report exact setup/executable hashes plus any hardware or interaction limit not directly observed.

## Non-goals

TypeScript 7, product behavior beyond the approved compact presentation, signing, automatic updates, new platform support, releases after 0.0.10, or Git commit creation without explicit authorization.

## Answer

### Exact candidate

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `build/DropCove-Setup-0.0.10.exe` | 3,231,015 | `f69173e3eb1d77d0292ede8cb4a2f6f2825cf082aee46ee06121bc8dddedfe9b` |
| Exact installed `dropcove.exe` | 13,383,168 | `844a545a112ab957f1afc9e22680953d77683826445708262bbdafa2335af418` |

The initial setup `34ee6f...` and installed executable `a33e37...` are superseded after disabling native Shelf shadow. Only the full hashes above identify acceptance bytes.

### Exercised evidence

- Full vue-tsc/Vite build and cargo-xwin Windows x64 release/Tauri NSIS packaging passed. Native suite: 101 passed, 0 failed; focused rail-height red/green proof passed. External SDK missing-PDB LNK4099 warnings were non-fatal.
- Exact setup installed/reinstalled with exit 0 and registered DisplayVersion 0.0.10. Final installed application ran with Vite stopped and loaded embedded `http://tauri.localhost/assets/index-C7gLM0ER.js`; mounted Shelf and native IPC were proven.
- Final installed Shelf desktop capture, Pin, Remove Batch, Manage Items, binary/tri-state states, native tooltips/accessibility descriptions, Dark/Light/forced-colors management rendering, and explicit close were exercised.
- Real mouse/OLE smoke used an owned Windows Forms destination: Temporary `item-3.txt` copied exact bytes and was consumed; Escape retained Temporary `item-4.txt`; Pinned `item-1.txt` and `item-2.txt` batch copied both exact bytes and retained both references. Source files were unchanged. This is not physical Explorer acceptance.
- Owned profile shutdown and uninstall exited 0; installed executable was removed and owned Profile Data was preserved byte-for-byte. Shared shortcuts remained byte-identical; installer registrations were restored. The original installed Resident Instance was restarted. Owned profiles, fixtures, WebView data, processes, and temporary OLE harness were removed.
- Final Standards and Spec review axes reported no remaining source defects. Review alone does not establish runtime acceptance.

### Restoration exception and final human gate

### Human acceptance completed

- User confirmed all outstanding acceptance tests passed on 2026-10-15 against the exact candidate recorded above:
  1. Windows Dark, Light, and Windows Contrast presentation on actual desktop corners and compact icon controls.
  2. Keyboard focus traversal, `aria-label`/`aria-pressed`/`aria-checked` semantics, and UI Automation behavior.
  3. Physical Windows Explorer drag intake and drag-out Copy/Move/cancel workflows.
  4. Preserved resident workflows: hotkey, system tray, Start with Windows, Shake-to-open, Edge Rail docking/timers, and fullscreen policy.
  5. Ambient default profile verification: user confirmed existing references and pinned items remain intact. On 2026-10-15 the user accepted the limitation that `shelf.db` was not backed up before installer shutdown and exact byte-for-byte database equality is not claimed.
- This is user-reported acceptance, not additional agent-run automation. Git commit authorization remains separate.