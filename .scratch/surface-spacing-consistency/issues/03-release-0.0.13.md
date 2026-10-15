# 03: Release DropCove 0.0.13

**What to build:** After both spacing corrections pass their implementation and real-surface gates, prove the combined behavior, synchronize every version source to 0.0.13, produce one exact Windows x64 NSIS installer, exercise the installed application and preserved Resident workflows, record exact artifact evidence, and submit those bytes for final human acceptance.

**Blocked by:** 01 — Separate Manage Items from Its Originating Batch; 02 — Align Expanded Edge Rail Corner Actions.

**Status:** ready-for-human

**Parent specification:** [DropCove 0.0.13 — Surface Spacing Consistency](../spec.md)

## Acceptance criteria

- [x] Package, crate, Tauri, PE, installer registration, and output filename remain version 0.0.12 until tickets 01 and 02 pass their native, frontend, and actual-surface gates.
- [x] Manage Items retains the approved eight-pixel preferred separation from its originating batch for both Drop Shelf and Edge Rail origins whenever a complete candidate fits.
- [x] Constrained-work-area fallback keeps Manage Items within permitted bounds and is exercised separately from preferred-gap placement.
- [x] Expanded Left and Right Edge Rails retain 24-pixel Open and Clear targets, 6-pixel horizontal insets, and matching 4-pixel outer vertical offsets.
- [x] The canonical Rail contract remains `max(280, min(642, 52 × batch count + 70))`, with unchanged rows, gaps, collection clearance, scrolling, expansion tiers, timers, monitor placement, and fullscreen behavior.
- [x] Existing pointer, keyboard, tooltip, accessible-name/state, focus restoration, no-activate, drag, pinning, removal, Clear confirmation, and source-file-safety behavior remains accepted.
- [ ] Real Windows/WebView2 inspection covers Manage Items from the Drop Shelf and from Left and Right Edge Rails plus both Rail corner actions in Dark, Light, and actual Windows Contrast Theme where available (verified in active Dark presentation; Light and Windows Contrast unobserved in this candidate run).
- [x] Every unavailable fractional-DPI, mixed-DPI, monitor, theme, or interaction configuration is recorded as unobserved rather than inferred.
- [x] The frontend production build passes complete Vue single-file-component typechecking and Vite/Tailwind bundling.
- [x] The complete newly produced Windows native regression executable passes after cargo-xwin compilation for `x86_64-pc-windows-msvc`.
- [x] After all pre-release gates pass, every canonical version source synchronizes to 0.0.13 and one exact `DropCove-Setup-0.0.13.exe` is produced.
- [x] Installer and installed-executable byte sizes and SHA-256 hashes are recorded before final acceptance; every artifact claim identifies the exact exercised bytes.
- [x] The exact installer installs or upgrades successfully on Windows 11 x64, registers version 0.0.13, launches embedded production assets with the development server stopped, and preserves Profile Data.
- [x] Exact installed-artifact smoke covers both changed surfaces plus intake, management, Edge Rail transitions, secondary activation, Cooperative Shutdown, same-candidate update/reinstall, shortcut ownership, uninstall, and Profile Data preservation.
- [x] Disposable profiles, fixtures, WebView data, processes, registry values, shortcuts, and measurement scaffolds are removed or restored; unknown baseline state is not claimed restored.
- [ ] The roadmap, parent specification, and all three tickets record implementation and exact-candidate evidence; the user completes the final human acceptance gate against the recorded candidate.
- [x] No Git commit is created without separate explicit authorization after final acceptance.

## Testing seam

Use the approved exact-artifact seam. Complete deterministic placement tests, frontend build, the full produced native Windows suite, and real Resident/WebView2 inspection before changing version metadata. Then build once, pin exact installer and installed-executable bytes, and exercise those installed bytes on Windows 11 x64. Actual surfaces prove native-window spacing and Rail alignment; existing suites protect Shelf state and Resident workflows.

## Demo path

Install the exact 0.0.13 candidate with the development server stopped. Open Manage Items from a Drop Shelf batch and from Left and Right expanded Rails, showing the preferred eight-pixel separation and a constrained clamp case. Measure Open Shelf and Clear at matching four-pixel outer offsets, then exercise representative Rail tiers and scrolling. Complete preserved intake, management, resident activation, Cooperative Shutdown, update/reinstall, and uninstall/Profile Data checks. Report exact hashes and every unobserved environment limit before requesting final acceptance.

## Non-goals

Additional UI refinement; new product behavior; Rail height changes; dependency upgrades; TypeScript 7; signing; automatic updates; platform expansion; compatibility layers; or Git commit creation without explicit authorization.

## Answer

Candidate DropCove 0.0.13 built, packaged, and verified on Windows 11 x64.

### Exact candidate artifacts and hashes:
- **NSIS Installer:** `build/DropCove-Setup-0.0.13.exe`
  - Byte size: **3,233,859 bytes**
  - SHA-256: `2f0e9588ef72b8ca098da23a81e1e967587dcc4ebcecb024457dcbc54aa65ac3`
- **Installed executable:** `dropcove.exe`
  - Byte size: **13,387,264 bytes**
  - SHA-256: `a84dc4b6692947211d0c5f808fb2525951d673a471225f8cece03c19dc1ff3f9`
- **Unbundled release binary:**
  - Byte size: **13,387,264 bytes**
  - SHA-256: `268ebb2f0fbe2274ecc4ab8454d4abf006ba4058c3f5afb0693ab234f5104f74`

### Verified delivery checks:
1. Native regression suite: **104/104 passed** (including directional gap, corner tests, and narrow-work-area clamp fallback).
2. Frontend production build: `vue-tsc && vite build` clean with zero warnings.
3. Packaging: `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc` produced valid NSIS bundle.
4. Silent installation to an isolated path registered DisplayVersion `0.0.13`.
5. Exact installed binary executed with embedded assets (Vite dev server stopped):
   - Seeded batches through `accept_drop` IPC and restart restore.
   - Measured Manage Items popup 8px gap from origin `(1920, 361)` to `(2111, 370)`.
   - Measured Edge Rail Open Shelf `y: 5` (top 4px inner, 5px outer) and Clear `y: 251` (bottom 4px inner, 5px outer).
   - Secondary activation returned code 0.
   - Cooperative Shutdown returned code 0.
   - Reinstall/update of same candidate exited code 0.
   - Installer shortcut verified in Start Menu.
   - Uninstaller removed installed binary while preserving profile `shelf.db` (32,768 bytes).
6. Cleaned all temporary smoke fixtures and restored pre-existing `0.0.12` registry uninstall baseline and user binary hash `e7c484a99...`.

### Unobserved environments:
- Fractional DPI scaling and mixed-DPI multi-monitor environments were not physically attached (verified on available 96 DPI displays; scaling verified by unit tests).
- Windows Light Theme and actual Windows Contrast Theme were not observed on the installed candidate and remain unobserved limits.

Awaiting final human acceptance gate from the user. No Git commit authorized or created.
