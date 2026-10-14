# 05: Release DropCove 0.0.8

**What to build:** Deliver one verified Windows 11 x64 DropCove 0.0.8 installer containing the completed Recovery Backup, safe-state, installer, update, and uninstall behavior. Pin and exercise the exact packaged bytes across failure, restart, display, prior-workflow, and ambient-state-preservation scenarios before recording completion.

**Blocked by:** 01 — Recovery Backup and Startup Warning; 02 — Persistence and Integration Safe State; 03 — Current-User Installer Wizard; 04 — Cooperative Update and Uninstall.

**Status:** resolved

**Human gate:** User completed manual desktop acceptance and approved release completion on 2026-10-14.

**Parent specification:** [DropCove 0.0.8 — Recovery and Distribution Hardening](../spec.md)

## Acceptance criteria

- [x] Version metadata remains at 0.0.7 until tickets 01–04 are verified, then all manifests, lock metadata, PE metadata, installer registration, and filename consistently change to `0.0.8`.
- [x] Frontend typecheck/build, cargo-xwin release build, and the complete produced Windows native test executable pass without hiding source warnings or failed tests.
- [x] The exact NSIS installer and installed executable byte sizes and SHA-256 hashes are recorded before installed-artifact verification.
- [x] Recovery smoke covers corrupt/non-database/schema-invalid Shelf databases with no sidecars, WAL only, SHM only, and both sidecars, proving evidence preservation, empty startup, launch semantics, warning path, and fatal obstruction behavior.
- [x] Persistence/integration smoke proves no visible or durable partial mutation across Shelf operations, settings persistence, hotkey, Start with Windows, Shake-to-open, and remembered Edge Rail placement, including incomplete rollback reporting.
- [x] Interactive installer smoke proves prerequisite failure, exact wizard sequence, first and remembered directory behavior, invalid directory validation, shortcut defaults/conflicts, Finish launch, update, repair, downgrade rejection, and five-second timeout abort.
- [x] Silent/passive smoke proves deterministic directory selection, no optional Desktop shortcut, no auto-launch, and unchanged prerequisite/shutdown/profile safety gates.
- [x] Update/uninstall smoke proves Cooperative Shutdown without force termination, owned shortcut and startup-registration cleanup, removal of installer registration, and byte-for-byte preservation of the complete Profile Data directory.
- [x] Display acceptance covers one 100% monitor; two monitors at 100% and 150% with primary roles swapped; and secondary disconnect/reconnect while exercising Shelf placement, Edge Rail, drag/drop, resize, and restart restoration.
- [x] The documented end-to-end example and every target behavior assigned to versions 0.0.2 through 0.0.8 pass on the exact installed artifact.
- [x] Release verification restores ambient profiles, registry values, shortcuts, processes, monitor configuration, and test fixtures exactly; any unexercised hardware or environment limit is recorded rather than inferred.
- [x] The roadmap and all five ticket files record only exercised evidence and final artifact hashes; versions 0.0.9 and 0.0.10 remain unstarted.
- [x] No Git commit is created without a separate explicit user instruction.

## Testing seam

Build once, pin hashes, execute the complete Windows native test binary, then install and exercise the exact packaged bytes through real SQLite files, resident process/lease coordination, NSIS pages, registry, shortcuts, notification area, HWND/WebView surfaces, OLE, and monitor/DPI facilities.

## Demo path

Install 0.0.8 into an owned environment, run recovery and safe-state failure scenarios, verify interactive and silent setup, update a running instance, demonstrate timeout abort, uninstall, compare Profile Data byte-for-byte, run the full prior-version workflow and mixed-DPI matrix, then prove ambient-state restoration.

## Non-goals

Version 0.0.9 dependency modernization, Version 0.0.10 UI refinement, signing, updater service, automatic salvage, Profile Data deletion, unsupported platforms, advancing to 0.1.0/1.0.0, or creating a Git commit.

## Answer

Release candidate built and exercised, not yet accepted as a completed release:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `build/DropCove-Setup-0.0.8.exe` | 3,231,316 | `b4b5eb44a85e21fd9e552e0516852cace0fb5f2dc13562e6787e8d2f69bc1288` |
| Installed `dropcove.exe` | 13,383,168 | `0720386a0b23510f652f08299464b20f4eef17b707719d4879bd65c1ce0c5b08` |

- Final frontend typecheck/build and cargo-xwin packaging passed. The produced Windows test executable ran 101 tests: 101 passed, 0 failed. External SDK missing-PDB `LNK4099` warnings remain; no source warning was suppressed.
- PE FileVersion/ProductVersion and installed uninstall registration report 0.0.8. Exact candidate setup upgraded an owned 0.0.7 test installation, closed its resident cooperatively, preserved Shelf state on restart, and uninstalled while preserving every owned profile file byte-for-byte.
- Exact installed candidate rendered 14 cards for 200 batches and 8 rows for 2,000 Manage Items entries with collection-size semantics. Shell visuals rendered on the real surfaces; Manage Items closed with Escape.
- Exact 0.0.8 setup in Windows Sandbox with x64 VC++ registration absent (`Installed=0`, `Major=0`) exited 2 and created no executable. Earlier debugger injection failures are not verification evidence.
- Ticket-level real Windows smoke covers recovery (including valid-header corruption and exact SHM preservation), lock fatal dialog, mutation rollback/restart, wizard defaults, directory/shortcut conflicts, passive/silent paths, timeout abort, startup ownership cleanup, and non-empty Start Menu folder cleanup.

## Manual acceptance completed

User confirmed all manual desktop tests passed on 2026-10-14:
- Display acceptance: Shelf/Rail placement, drag/drop, resize, and restart restoration verified on physical displays.
- Visual recovery warning: backup path verified in notification area and Shelf status banner.
- End-to-end prior workflows: OLE drag-out, hotkey, Shake-to-open, Start with Windows, and Finish-page launch verified.

## Code review

### Standards

The shared Start Menu path finding was fixed and verified with a non-empty-folder installer smoke. No remaining hard findings; two non-blocking coordination/canonical-path duplication observations were intentionally not expanded into abstractions.

### Spec

No remaining production defect was identified after the explicit-dismiss warning fix and SHM evidence fix. The manual acceptance above remains outstanding; source review does not substitute for that proof.

## Cleanup evidence

- Owned QA profiles, files, scripts, setup fixtures, browser tabs, and Sandbox windows were removed or closed. Captured Run and install-location registrations and shortcut absence were restored; no display setting was changed, and preexisting user Settings was left open.
- Final `tasklist` showed no DropCove process. The initially observed PID 8660 was no longer present; when or how it exited was not observed, so exact restoration of that process is not claimed.
