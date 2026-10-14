# 03: Release DropCove 0.0.9

**What to build:** After the modernized build graph and Windows runtime are proven, synchronize DropCove to version 0.0.9, produce one exact Windows 11 x64 installer, verify its installed application and preserved workflows, record artifact evidence, and submit those exact bytes for final Dark, Light, Windows Contrast, and native drag-and-drop acceptance.

**Blocked by:** 02 — Windows Runtime Compatibility.

**Status:** resolved

**Human gate:** User confirmed all outstanding exact-candidate acceptance tests passed on 2026-10-14. Git commit authorization remains separate.

**Parent specification:** [DropCove 0.0.9 — Frontend Toolchain Modernization](../spec.md)

## Acceptance criteria

- [x] Version metadata remains `0.0.8` until tickets 01 and 02 are technically verified under the approved human-gate clarification, then package, crate, Tauri, PE, installer registration, and output filename synchronize to `0.0.9`.
- [x] The release dependency install resolves the approved exact lockfile graph cleanly; Pinia, Vue Router, direct PostCSS, Autoprefixer, and TypeScript 7 remain absent.
- [x] Full Vue single-file-component typechecking and the Vite/Tailwind production build pass with TypeScript `6.0.2`, vue-tsc `3.3.13`, Vite `8.3.4`, and Tailwind CSS `4.3.3`.
- [x] The complete produced Windows native test executable passes, and the cargo-xwin release/Tauri/NSIS packaging flow completes for `x86_64-pc-windows-msvc`.
- [x] The installed application loads embedded production assets from `tauri.localhost`, not the Vite development URL; standalone surface mounting and native IPC are proven. The standard default custom-protocol feature correction was approved on 2026-10-14.
- [x] One exact `DropCove-Setup-0.0.9.exe` is copied to the release build location; its byte size and SHA-256 hash are recorded before installed-artifact verification.
- [x] The exact installer upgrades or installs successfully on Windows 11 x64, registers version 0.0.9, launches the exact installed executable, and preserves the existing current-user installer, Cooperative Shutdown, Profile Data, shortcut, startup, and uninstall contracts.
- [x] The installed executable byte size and SHA-256 hash are recorded, and all release claims identify the exact setup and installed executable bytes exercised.
- [x] Installed-artifact smoke covers Drop Shelf, Edge Rail, Manage Items, Settings, dialogs, Shell Visuals, virtualization, intake, drag-out, hotkey, startup, Shake-to-open, update/repair, and uninstall/profile preservation without frontend modernization regression.
- [x] The user confirms Dark, Light, Windows Contrast, focus/reduced-motion presentation, and native drag-and-drop against the exact candidate before the release ticket is resolved.
- [x] The roadmap, specification, and all three ticket files record only exercised evidence, the TypeScript 7 upstream deferral, final hashes, and release status; version 0.0.10 remains unstarted.
- [x] Release verification restores ambient profiles, processes, shortcuts, registry values, theme/Contrast settings, and test fixtures changed during smoke, with the user-approved startup reconciliation explicitly documented instead of claiming unknown original bytes.
- [x] No Git commit is created without a separate explicit user instruction after acceptance.

## Testing seam

Use the approved exact-artifact seam. Rebuild from the locked graph, pin setup and installed executable bytes, then install and exercise those same bytes on Windows 11 x64. Technical verification belongs to the agent; final visual-theme and native drag-and-drop acceptance belongs to the user.

## Demo path

Run the exact 0.0.9 installer, launch the installed application, and complete the existing end-to-end Shelf workflow across every Tauri surface. Show resolved dependency versions, passing typecheck/build/native tests, installer and executable hashes, observable Dark/Light/Contrast equivalence, and successful native intake/drag-out. Uninstall and prove Profile Data remains preserved.

## Non-goals

TypeScript 7 before upstream vue-tsc/Volar compatibility, Drop Shelf or Edge Rail refinement assigned to 0.0.10, new product behavior, new routing/store architecture, package-manager migration, support outside Windows 11 x64, advancing beyond 0.0.9, or creating a Git commit without explicit authorization.

## Answer

### Exact candidate

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `build/DropCove-Setup-0.0.9.exe` | 3,228,174 | `6a340c51059c06060e0c81a3464c9d9e4a50528cfe5eeb6095d59cec5cf97fca` |
| Installed `dropcove.exe` | 13,383,168 | `f8a9d9ad75637d935159c6d1a8aa55f1f43e07b207c4b514a1f913d2f1aca082` |

The earlier dev-context candidate is invalidated. Only the setup hash above and its installed executable are acceptance candidates. The release-directory executable has a different hash; it is not substituted for the installed bytes.

### Exercised verification

- Frozen strict-peer pnpm install and full vue-tsc/Vite/Tailwind build passed on Node 24.18.0 and pnpm 12.11.2.
- Final production-feature native test executable `dropcove_lib-1ce9a8357c789290.exe` ran 101 tests: 101 passed, 0 failed. Cargo-xwin release/Tauri NSIS build passed; external SDK missing-PDB LNK4099 warnings remain.
- Exact setup installed silently into an owned temporary directory with `/S /NS /D=...`; registered DisplayVersion, FileVersion, and ProductVersion all reported 0.0.9.
- Exact installed application loaded `http://tauri.localhost/` and bundled `index-BnRKQAGt.js`, mounted native Shelf, and invoked native Shelf IPC. No Vite client or development origin was used.
- Installed Shelf accepted 71 batches including a 70-item batch. Manage Items rendered 8 rows with `Batch items (70)` semantics, real Shell file icons, and Escape teardown. Batch preparation/cancel retained all 71 batches. No browser errors were observed.
- Same-version repair exited 0, shut down the owned default-profile resident cooperatively, retained the pinned installed executable hash, and restored one persisted batch on relaunch. An initial isolated repair attempt correctly aborted with exit 2 because `/NS` had preserved the original installation's shortcut while the temporary registration no longer proved its ownership; restoring the original ownership registration for that test allowed repair. No shortcut was deleted or overwritten.
- Exact uninstaller exited 0, removed program files and uninstall registration, and preserved the owned Profile Data byte-for-byte. Original installation registration, shortcut bytes, and default Profile Data bytes were restored/preserved; original resident PID 18340 remained running. Owned test profiles, WebView data folders, processes, and fixtures were removed; no Windows theme setting was changed.

### Startup reconciliation exception

Temporary resident profiles inherited `StartWithWindows: true` and wrote the shared HKCU Run value during smoke. The snapshot taken immediately before installer testing already contained a dangling dev-profile startup command, so it was not a valid pre-session baseline. Exact restoration of the original value is not claimed.

User approved reconciling startup to the verified original installation and real profile, whose durable settings contain `StartWithWindows: true`. HKCU Run `DropCove` was set and read back exactly as `"C:\DropCove_007_FinalTest\dropcove.exe" --autostart --profile "\\?\C:\Users\Hung\AppData\Local\DropCove"`. The existing application was not closed or replaced. Future isolated smoke must capture shared integration values before the first launch and explicitly disable startup in temporary profile settings.

### Human acceptance completed

User confirmed all outstanding acceptance tests passed on 2026-10-14 against the exact candidate recorded above:
1. Interactive installation, current-user wizard, shortcut choices, and Finish-page launch.
2. Settings and collapsed/expanded Edge Rail; Dark, Light, Windows Contrast, keyboard focus, controls, and reduced-motion behavior.
3. Explorer drag intake and drag-out Copy/Move/cancel, hotkey, Start with Windows, and Shake-to-open workflows.

This is user-reported acceptance, not additional agent-run automation. Git commit authorization remains separate.

### Code review

- Standards axis: no documented-rule breaches or baseline smell findings in the frontend migration and approved Cargo production-feature/version delta.
- Spec axis: no code-level missing/incorrect requirements or scope-creep findings in those deltas. Final human acceptance is recorded separately above; review alone did not establish it.