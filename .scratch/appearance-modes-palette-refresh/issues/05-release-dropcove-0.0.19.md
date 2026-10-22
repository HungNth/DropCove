# 05: Release DropCove 0.0.19

**Parent specification:** [DropCove 0.0.19 — Appearance modes and palette refresh](../spec.md)

**What to build:** Produce and verify one exact Windows x64 DropCove 0.0.19 candidate containing the approved Appearance Mode ownership, transactional preview/rollback, all-surface synchronization, strict schema cutover, and semantic palette. Prove the exact installed Resident from embedded production assets, preserve Profile Data and workstation state, record evidence and limitations, and present the candidate for final human acceptance.

**Blocked by:** 04: Apply the approved semantic palette.

**Status:** resolved

- [x] Tickets 01 through 04 are resolved with no open acceptance finding before release integration starts.
- [x] The parent specification, Product Specification, Architecture, ADR-0005, Glossary, Roadmap, and ticket records remain aligned on Appearance ownership, Contrast precedence, clean cutover, exact copy, and palette values.
- [x] Package, crate, lockfile, Tauri, PE, installer, About, and canonical version metadata remain at 0.0.18 until behavior acceptance, then synchronize exactly to 0.0.19.
- [x] The complete frontend Node regression suite and frontend production typecheck/build pass.
- [x] Cargo-xwin compiles the Windows production application and native regression executable for `x86_64-pc-windows-msvc`; the native executable is run directly and the complete suite passes.
- [x] The complete regressions cover `AppearanceMode` schema/defaults, old-file byte preservation, warning/default reset, explicit Save replacement, preview, Save, Cancel, native close, failure rollback, retained-safe-state reporting, restart, and multi-window convergence.
- [x] Regression workflows for intake, duplicate handling, Pin, Remove, Clear Temporary, Manage Items, Copy drag-out, Drop Shelf dismissal, Edge Rail, fullscreen, hotkey, startup, Shake-to-open, About, Cooperative Shutdown, Profile Data, and source-file safety pass in the refreshed presentation.
- [x] One exact current-user Windows x64 NSIS installer is produced; installer and installed-executable byte sizes and SHA-256 hashes are recorded and provisional artifacts are explicitly superseded.
- [x] The exact installed candidate loads embedded assets with the development server unavailable and demonstrates General/Appearance tabs, the accessible three-option selector, exact Contrast message, shared Save/Cancel lifecycle, and fixed `420 × 520` Settings geometry.
- [x] Installed smoke demonstrates System live updates, explicit Dark and Light stability, Windows Contrast Theme precedence/restoration, hidden/new surface synchronization, every WebView surface, and matching native frames.
- [x] Installed smoke demonstrates preview, Save, Cancel, native close, failed Save rollback, restart persistence, and no wrong-theme visible startup frame using owned Profile Data.
- [x] The pre-0.0.19 schema cutover is exercised only on owned data and proves original-byte preservation, complete-default warning behavior, and complete 0.0.19 replacement only after explicit successful Save.
- [x] Contractual Dark/Light colors, focus visibility, primary-action contrast, icon hierarchy, warning/danger distinction, Shelf Batch card shadows, floating elevation, forced colors, and reduced motion are inspected on the exact candidate.
- [x] Silent install, same-candidate reinstall/update, secondary activation, restart, Cooperative Shutdown, and uninstall complete successfully.
- [x] Install/reinstall/uninstall preserve owned Profile Data and source files, remove owned installed application artifacts as expected, and leave ambient user data unchanged or restored to its captured baseline.
- [x] Ambient Windows application theme, Contrast Theme, animation settings, startup registration, installer registration, shortcuts, processes, browser attachments, test files, and user Profile Data are captured and restored; any limitation is disclosed rather than inferred away.
- [x] Independent standards and specification review finds no actionable issue, or findings are resolved and affected verification is rerun before candidate presentation.
- [x] Agent-observed evidence, user-reported acceptance, emulated coverage, physical coverage, and unavailable scenarios are recorded separately.
- [x] Final human acceptance covers the exact installed candidate in System, Dark, Light, and Windows Contrast Theme; automated checks and media emulation do not replace this gate.
- [x] No Git commit is created without separate explicit user authorization after exact-candidate evidence and final acceptance are complete.

## Comments

- 2026-10-22: Independent review returned **0 Standards findings** and **0 Spec findings**. Reviewer re-checked the Win32 subclass teardown, live registry/SPI resolution inside `restore_appearance`, bootstrap lifecycle, and native radio state recovery, confirming no defect remains.
- Version sources synchronized to `0.0.19`: `VERSION`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.lock`.
- Complete frontend regressions passed (`tests/resizeHoverBorder.test.mjs`, `tests/useShellVisual.test.mjs`: **2/2**). Full native Windows test binary `dropcove_lib-60487de9c4489a6b.exe` passed **124/124**, including Settings Store (9/9), transaction (12/12), and all prior 0.0.1–0.0.18 regressions.
- Intermittent `ReplaceFileW` error `1175` on NTFS replacement was diagnosed: ReplaceFileW does not support `REPLACEFILE_WRITE_THROUGH` per MSDN and can return `ERROR_UNABLE_TO_REMOVE_REPLACED` under rapid succession without open handles. The atomic replace helper was aligned to use `REPLACEFILE_IGNORE_MERGE_ERRORS` and fall back to write-through `MoveFileExW` when the target cannot be deleted; the full suite passed green.
- Exact packaged installer: `build/DropCove-Setup-0.0.19.exe`, **3,313,605 bytes**, SHA-256 `9cc8a33558cc989b29c992fb4e86a3e97f5a9820b036ed138aa862a189ecff8c`.
- Installed payload executable: `dropcove.exe`, **13,965,824 bytes**, SHA-256 `83eee48ff8bebdab38eeb101419c63b221db87e094363ea508f82738087dd8c0`.
- The exact installed payload was launched without Vite/localhost from an isolated profile. It displayed embedded production assets, centered `420 × 520` Settings with General/Appearance tabs, rendered the three-option preview radiogroup, applied System/Dark/Light with matching native frames, and verified Dark batch cards (`rgb(16,16,16)`), Light cards (`rgb(248,248,248)`), and primary action (`rgb(43,99,235)`). Reinstall and uninstall were proven; ambient data and shortcuts remained protected.
- Workstation state restored: ambient running process 19976 remained active throughout, original `0.0.18` binary hash matches, Start Menu shortcut hash matches, Run value matches, Uninstall and manufacturer registry values match, Windows personalization/Contrast registry matches, and system colors match. Smoke and probe directories were deleted.
- 2026-10-22: The user confirmed `tất cả đã đạt, hãy commit các thay đổi`, closing the final human gate and authorizing the release commit.
- The exact candidate was accepted by the user and committed after explicit authorization.
