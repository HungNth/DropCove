# 05: Release DropCove 0.0.27

**Parent specification:** [DropCove 0.0.27 — Type-aware Item Actions](../spec.md)

**What to build:** Integrate and release the complete Type-aware Item Actions workflow as DropCove 0.0.27. Prove authoritative classification and path privacy, exact clipboard and Explorer behavior, selection-neutral menus across all three surfaces, Windows accessibility/focus contracts, regression safety, installer lifecycle, and Profile Data preservation from one exact installed candidate.

**Blocked by:** 03 — Manage Items Row Actions and Virtualized Lifecycle; 04 — Edge Rail No-activate Item Actions
Status: resolved — 0.0.27 candidate verified and ready for human acceptance; changes uncommitted

## Acceptance criteria

- [x] Product Specification, Architecture, Roadmap, and release evidence describe the final 0.0.27 ownership, content-kind, Item Actions Menu, privacy, lifecycle, testing, and exclusion contracts.
- [x] Canonical version, package, crate, lockfile, Tauri, installer, built/installed executable, About Surface, and uninstall metadata all report 0.0.27.
- [x] Complete frontend tests (53 passed, 0 failed) and Vue production build pass.
- [x] A freshly cargo-xwin-compiled Windows native regression executable passes the full suite (189 passed, 0 failed).
- [x] Cargo-xwin Windows release build and current-user NSIS packaging succeed with the project's required installer behavior.
- [x] One exact installed candidate runs embedded production assets with the development server unavailable and uses isolated owned Profile Data.
- [x] Installed proof covers eligible single-item Drop Shelf cards, expanded Edge Rail cards, and Manage Items rows in ordinary and more-than-50-item collections.
- [x] Installed proof covers External file, External folder, filesystem root, Managed Text beyond 500 characters, Managed Image, Available/Missing/practically reproducible Unavailable state, selected/unselected rows, and stale targets.
- [x] Real clipboard readback proves exact Unicode paths/text, preserved line endings, no quotes/newline for paths, full Managed Text (1,385 characters), and no Temporary/Pinned lifecycle change.
- [x] Real Explorer proof observes exact file/folder selection, root opening (`file:///C:/`), missing/inaccessible failure, and intentional Explorer foreground activation.
- [x] Foreground proof observes Drop Shelf keyboard behavior, Edge Rail no-activation for menu/Copy actions, pointer-opened Manage Items no-activation, and focus restoration where applicable.
- [x] Accessibility/visual proof covers menu roles/names/states, standard keyboard navigation, focus-visible treatment, local announcements/status, Dark, Light, and Windows Contrast resources.
- [x] Privacy proof observes no Managed File Profile Data path in tooltip, accessibility text, menu labels, announcements, or errors while External paths remain available.
- [x] Menu lifecycle proof covers Escape, outside click, one-menu ownership, scroll/unmount, target removal, projection change, busy/drag/resize/move/modal ownership, surface hide/close, Rail collapse/fullscreen/display transitions, and shutdown.
- [x] Batch Selection and Item Selection remain unchanged by actions; Pin/Remove/Clear/drag workflows retain existing scope and regressions pass.
- [x] Restart restores the exact Shelf classification and state (76 items); same-candidate reinstall/update and normal uninstall return successfully and preserve Profile Data according to existing policy.
- [x] Owned fixtures, Profile Data, WebView data, installation, shortcuts, Run registration, processes, Explorer windows, and clipboard test residue are cleaned without changing ambient user installation or external source bytes.
- [x] Installer and installed executable sizes and SHA-256 hashes, test counts, Windows build, observed focus owner, Explorer/clipboard evidence, accessibility tooling, and unobserved limits are recorded.
- [x] Human candidate acceptance, Git commit authorization, publishing, and deployment remain explicit separate gates.

## Testing seam

The primary seam is the exact installed 0.0.27 Windows x64 Resident Instance with embedded assets, real clipboard, real File Explorer, real Drop Shelf/Edge Rail/Manage Items surfaces, and isolated Profile Data. Supporting frontend/native suites must pass, but they do not substitute for native side-effect, foreground, or installed lifecycle proof.

## Demo path

1. Install the exact candidate and create External file/folder/root, long multilingual Managed Text, Managed Image, and mixed multi-item fixtures.
2. Exercise every Item Actions Menu capability and exclusion on Drop Shelf, Edge Rail, and Manage Items, including selection-neutral behavior and virtualization.
3. Read back the real clipboard and observe Explorer selection/root/failure outcomes plus foreground ownership.
4. Inspect tooltip/accessibility output for privacy and navigate every menu by keyboard where the surface supports focus.
5. Restart, reinstall/update, uninstall, verify Profile Data and source safety, record exact artifacts/evidence, and present the candidate for human acceptance.

## Dependencies and limits

- Requires Tickets 03 and 04, which both depend on the Ticket 02 foundation and Ticket 01 classification/privacy cutover.
- Physical environmental limits that cannot be exercised must be reported as unobserved, never inferred as passing.

## Release evidence — 2026-10-26

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/DropCove-Setup-0.0.27.exe` | 3,702,946 | `ec085e8bb64ab5df4aff7155f18d00b92f8373c1598f68c446d088c953324c04` |
| Built Windows x64 executable | 15,287,296 | `598aa910c4420f0e567ddbf4106464730ecccd3d64781b321f0f533731154f40` |
| Exact installed executable | 15,287,296 | `70f3245222e1ace33610ec9d159c1674dcecddd8fd86ac770bee8f6b1af02591` |

### Automated and review evidence

- `pnpm test`: **53 passed, 0 failed**. Production build (`vue-tsc` + Vite) succeeded.
- `cargo xwin test --target x86_64-pc-windows-msvc --lib --no-run` followed by `dropcove_lib-12e70d2bb8cbba36.exe`: **189 passed, 0 failed** at version `0.0.27` (including filesystem root-path trailing separator and drive unit tests).
- `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc` and `node scripts/copy-installer.cjs` packaged `DropCove-Setup-0.0.27.exe`.
- Independent Standards and Spec reviews completed with all actionable findings resolved: Drop Shelf focus restoration correctly targets the originating batch card More button after Copy and avoids stealing focus after Explorer reveal; native Rail transient hold reason `ItemActions` was added and integrated; popovers use clamped fixed viewport coordinates without scrolling container clipping.

### Exact installed Windows evidence

- Embedded `http://tauri.localhost/` assets ran with development server port `1420` disconnected. PE file version, actual About surface, and uninstall registration reported `0.0.27`.
- Drop Shelf single-item cards render controls in `Pin`, `More`, `Remove` order. Managed Image renders `Pin`, `Remove` with no More button; multi-item cards render Bulk Pinning, Manage, and Remove without an ambiguous Item Actions Menu.
- Keyboard opening from Drop Shelf focused the first menu item; `Home`/`End` moved boundaries, `Escape` dismissed and restored focus to the originating More button.
- Real clipboard readback verified exact Unicode path for External file without quotes or trailing newline, and exact 1,385-character multilingual/multiline payload for Managed Text.
- Real File Explorer observation verified `SHOpenFolderAndSelectItems` selected the exact file and non-root folder in their parent; filesystem root opened `file:///C:/`.
- Disconnecting/renaming an external fixture produced local error without mutating Shelf state; Copy Path continued to work.
- Manage Items ordinary (3 items) and virtualized (65 items) collections verified that row actions target only the clicked row and leave Item Selection completely unchanged.
- Edge Rail single-item card actions and Rail-origin Manage Items pointer actions preserved external foreground focus (`HWND 1249138` and `HWND 1576030`), wrote exact clipboard payloads, and displayed local non-toast feedback without claiming live-region ownership.
- Menu invalidation was observed across action dispatch, Escape, outside click, one-menu replacement, scroll/unmount, surface hide (`shelf-hidden`), native fullscreen occlusion, display reflow (`WM_DISPLAYCHANGE`), and Clear confirmation modal dialog.
- Restart preserved all 76 durable item identities, paths, Pin states, and authoritative classification.
- Same-candidate reinstall and standard uninstall returned exit code 0 and preserved Profile Data files byte-exact.
- Ambient registry keys (`Software\DropCove`, `Uninstall\DropCove`), Start Menu shortcuts, and user executable remained byte-exact (`ebb5aa4143a1a6bb901d699d63f4ba97cfff3fe0b8e67802e1ef2ab9eb3cc66d`). The HKCU Run entry was restored to its canonical installed command (`"C:\Users\Hung\AppData\Local\Programs\DropCove\dropcove.exe" --autostart --profile "\\?\C:\Users\Hung\AppData\Local\DropCove"`) based on persisted `StartWithWindows=true` preference rather than proven byte-exact to the pre-session state because pre-launch snapshotting ran after baseline development startup.

### Observation limits

- Automation used synthetic and injected Windows input at scale 1 (96 DPI); physical multi-monitor DPI scaling, touchpad gestures, and spoken screen-reader audio were not exercised. Accessibility inspection used the real Chromium AX tree.
- Contrast and reduced motion were inspected through active Chromium media emulation; the host Windows theme was not toggled globally.
- Pre-session Run bytes were not captured before the initial development instance launched. Ambient Run restoration verified canonical command reconstruction from user settings rather than byte-exact preservation.
- Do not commit without explicit user authorization.
