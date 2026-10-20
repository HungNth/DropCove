# 03: Release DropCove 0.0.17

**What to build:** Integrate the approved Settings presentation and Cargo/About metadata slices into one exact DropCove 0.0.17 Windows x64 release candidate. Prove the combined behavior in the running and installed application, synchronize version metadata only after implementation acceptance, preserve Profile Data and resident lifecycle safety, and record final human acceptance separately from agent-observed evidence.

**Parent specification:** [DropCove 0.0.17 — Settings Window and About Metadata Polish](../spec.md)

**Testing seam:** Complete frontend and cargo-xwin Windows checks plus actual Resident/WebView2/taskbar/notification-area acceptance on the exact packaged and installed candidate.

**Demo path:** Install the exact current-user candidate in an isolated location, open Settings from both entry points across available monitors, navigate through the Windows taskbar, exercise singleton and Save/Cancel behavior, inspect authoritative About metadata, restart/reinstall, then uninstall while proving Profile Data preservation and cleanup.

**Blocked by:** 01: Taskbar-visible, invocation-centered Settings; 02: Cargo-owned About author and description.

**Status:** resolved

- [x] Tickets 01 and 02 are resolved with no open acceptance findings before release integration starts.
- [x] Canonical product and architecture documentation describes Settings as a taskbar-visible top-level singleton, the shared invocation-context placement contract, and Cargo-owned About description projection without rewriting completed historical evidence.
- [x] The glossary remains unchanged unless implementation introduces a genuinely new domain term; Settings and About Surface retain their existing canonical meanings.
- [x] All canonical version sources remain at `0.0.16` until implementation acceptance, then synchronize exactly to `0.0.17`, including package, Cargo, lockfile, Tauri, and canonical version metadata.
- [x] The complete frontend production build passes Vue typechecking and production bundling.
- [x] Cargo-xwin builds the Windows production application, native regression suite, and current-user NSIS package for `x86_64-pc-windows-msvc`; the produced native regression executable is run directly on Windows.
- [x] The complete native regression suite passes, including Settings transactions, hotkey/startup/Rail/Shake behavior, shared placement geometry, About metadata, Resident Instance lifecycle, persistence, Cooperative Shutdown, and source/Profile Data safety.
- [x] Actual running-app smoke proves Drop Shelf invocation centers a new Settings window on the Drop Shelf monitor and notification-area invocation centers it on the pointer monitor, with complete outer-frame work-area containment and primary fallback where practical.
- [x] Actual Windows Shell observation proves Settings has one normal taskbar button, remains reachable while the Drop Shelf is visible, hidden, or EdgeDocked, and does not change Shelf presentation when opened from the notification area.
- [x] Repeated Settings requests from both entry points prove singleton reuse, restore/focus without repositioning, preserved manual movement, and no competing drafts.
- [x] Actual Settings workflows prove Save, Cancel, native close, shortcut recording/focus loss, Start with Windows, Rail placement, fullscreen, Shake-to-open, warnings, rollback, Exit DropCove, and Cooperative Shutdown remain correct after native ownership changes. Remaining human checks accepted by user report below.
- [x] Actual About Surface reports runtime version `0.0.17`, author `HungNth`, and the exact Cargo description immediately below the version without clipping, scrolling, or duplicated frontend metadata.
- [x] About singleton, taskbar, invocation placement, Author, Source, browser opening, error handling where practical, disabled Check for updates, Coming soon, `Escape`, and close behavior remain correct. Remaining human checks accepted by user report below.
- [x] Dark, Light, Windows Contrast where available, reduced-motion, keyboard, focus, reading order, available monitor/DPI configurations, and explicit unobserved hardware limits are recorded without inference.
- [x] One exact current-user Windows x64 NSIS installer is produced; installer and installed-executable byte sizes and SHA-256 hashes are recorded, and provisional artifacts are superseded explicitly.
- [x] The exact installed candidate loads embedded frontend assets with the development server unavailable and reproduces Settings taskbar/placement and About metadata behavior from shipped bytes.
- [x] Silent install, same-candidate reinstall/update, secondary activation, restart retention, Cooperative Shutdown, and uninstall complete successfully.
- [x] Reinstall/uninstall Profile Data and source-file safety accepted by the user with the disclosed manufacturer-key capture limitation below; historical agent evidence is unchanged.
- [x] Owned smoke profiles, fixtures, WebView data, processes, installer registrations, shortcuts, Run values, and temporary harnesses are removed or restored after verification; the manufacturer-key capture limitation is explicitly recorded below.
- [x] Agent-observed evidence and unavailable physical coverage are recorded separately from final human acceptance. The user accepted the exact `0.0.17` candidate.
- [x] No Git commit is created without separate explicit user authorization after release evidence and final acceptance are complete.

## Answer — exact candidate, 2026-10-20

- Installer: `build/DropCove-Setup-0.0.17.exe`, **3,291,898 bytes**, SHA-256 `9488308f7c441a235a79eff9e0eeef739e28360ab0b658248eaa975d523f4973`.
- Installed executable: **13,877,760 bytes**, SHA-256 `e4a485be4cce67a3b029201bfa361b43908a8cab8bcebc023c4a70a66a93228b`. Runtime and uninstall metadata report `0.0.17`; canonical package/Cargo/lock/Tauri/version metadata is synchronized.
- Frontend vue-tsc/Vite, cargo-xwin production/NSIS build, and freshly compiled Windows test executable `dropcove_lib-10b47a517ec3b11e.exe` passed. Complete native suite: **114 passed, 0 failed**. External SDK missing-PDB `LNK4099` warnings remain unsuppressed.
- Actual installed WebView2 loaded `http://tauri.localhost/assets/index-BTj74sF7.js`; localhost port 1420 refused connections. Settings had no native owner and centered its complete `436 × 559` decorated frame correctly. The Windows taskbar showed one DropCove window while Shelf was hidden/EdgeDocked; tray Settings did not reveal the Shelf or remove the Rail.
- Singleton restore retained moved origin `[260,160]`. Real Settings smoke covered Save, Cancel/native close draft retention, recorder focus-loss cancellation, On/Off startup registration, fullscreen and Shake enable/sensitivity controls, and a share-denied settings file. The latter produced visible `ReplaceFileW` error 32 while retaining original bytes; this is intentional rollback proof, not a failing application check.
- About IPC returned author `HungNth` and the exact approved description below runtime version `0.0.17`. Actual `400 × 460` layout had scroll height equal to client height, complete two-line description, correct reading order, and disabled update control. Dark/Light screenshots and CDP forced-colors/reduced-motion rendering were inspected; actual Windows Contrast Theme and screen-reader output are not claimed.
- Silent install, same-candidate reinstall, secondary activation, exact Pinned Shelf snapshot restart retention, Cooperative Shutdown with Settings open, and uninstall passed. Reinstall retained installed executable and owned profile hashes. The first isolated `/NS` reinstall was correctly rejected with exit 2 because the preserved ambient Start Menu shortcut pointed outside the new installation. Temporarily protecting that exact shortcut during the repeat allowed exit 0; its original bytes were immediately restored.
- Uninstall exited 0 and removed the installed executable; NSIS left an empty install directory, subsequently removed with owned smoke resources. Owned Profile Data and source-fixture hashes remained unchanged through uninstall. Ambient executable/profile hashes, captured uninstall namespaces, shortcut bytes, and the original HKCU Run value were restored/unchanged.
- **Registry capture limitation:** the manufacturer namespace was discovered after installation as `Software\dropcove\DropCove`, rather than the initially captured author-based namespace. Its default install path was restored from the pre-install captured uninstall location, and its observed language value `1033` was retained. Exact pre-install bytes for this manufacturer key were not independently captured and are not claimed restored.
- Independent two-axis review against baseline `a1f677cbb1941c28c27d6506d38d8467c2ba6781` returned **0 Standards findings** and **0 Spec findings**. Review covered the scoped tracked changes and the new shared auxiliary-window module. No post-package source correction was required.

### Final human gate

- The user confirmed `tất cả đã đạt, hãy commit các thay đổi`, closing final human acceptance and explicitly authorizing the commit. Historical agent-observation limits and the manufacturer-key capture limitation remain recorded; acceptance is user-reported, not additional agent proof.

## Comments

- 2026-10-20: The user selected `Chờ tôi kiểm tra candidate` and `Không — giữ uncommitted`. Keep this ticket `ready-for-human`, retain the exact candidate for their checks, and create no Git commit. No additional implementation or source correction was requested.
- The user subsequently confirmed `tất cả đã đạt, hãy commit các thay đổi`, superseding the earlier wait/no-commit decision, accepting the candidate, and authorizing the Git commit.
