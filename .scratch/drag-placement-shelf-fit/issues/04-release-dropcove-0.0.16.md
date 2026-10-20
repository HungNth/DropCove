# 04: Release DropCove 0.0.16

**What to build:** Integrate the approved Automatic Shelf Sizing and Drop Placeholder slices into one exact DropCove 0.0.16 Windows x64 release candidate, prove the combined behavior in the running and installed application, synchronize version metadata only after implementation acceptance, and preserve resident, installer, and Profile Data safety.

**Parent specification:** [DropCove 0.0.16 — Drag Placement Feedback and Drop Shelf Fit](../spec.md)

**Testing seam:** Complete frontend and cargo-xwin Windows checks plus actual Resident/WebView2/OLE acceptance on the exact packaged and installed candidate.

**Demo path:** Install the exact candidate into an isolated current-user location, exercise Drop Shelf and Edge Rail placeholder workflows plus automatic/manual sizing and resident lifecycle, reinstall the same candidate, then uninstall while proving Profile Data preservation and cleanup.

**Blocked by:** 01: Exact Automatic Shelf Sizing; 03: Edge Rail Drop Placeholder feedback.

**Status:** resolved

- [x] Tickets 01 and 03 are complete with no unresolved acceptance findings before release integration begins.
- [x] Canonical version sources remain at `0.0.15` until implementation acceptance, then synchronize exactly to `0.0.16` without hard-coded frontend milestone text.
- [x] The complete frontend production build passes Vue single-file-component typechecking and production bundling.
- [x] Cargo-xwin compiles the Windows production application, native regression suite, and NSIS package for `x86_64-pc-windows-msvc`; the produced native test executable is run directly on Windows.
- [x] The complete native regression suite passes, including exact Automatic Shelf Sizing, Manual Height Override, final-empty reset, storage eligibility, Edge Rail timing, resident lifecycle, persistence rollback, and source-file safety.
- [x] Complete physical acceptance of the independent resize-hover outline and drag-time window-movement guard. Actual OLE/WebView2 already proved placeholder feedback, no surface glow, stable hover geometry, ordinary/virtualized collections, and accepted/duplicate/unsupported input; unchanged guards were reviewed but not all native gestures were re-exercised.
- [x] Actual running-app smoke proves Edge Rail delayed expansion, one placeholder after expansion, no focus theft, no preview resize, both docking edges, existing collapse/transient behavior, and unchanged retained-batch height formula.
- [x] Complete actual-surface acceptance of every row-reducing UI workflow. Native Shelf State regressions already cover all mutation paths and manual precedence; WebView2/installed smoke proved every automatic tier, width-only grow/shrink, explicit batch removal, top-left anchoring, manual-height preservation, restart, and final reset.
- [x] Dark, Light, Windows Contrast where available, reduced-motion, available DPI/monitor combinations, and unobserved hardware limitations are recorded without inference.
- [x] Produce one exact current-user Windows x64 NSIS installer and record installer and installed-executable byte sizes and SHA-256 hashes; supersede provisional artifacts explicitly.
- [x] Silent installation registers version `0.0.16`, installed production assets load with the development server unavailable, and the exact installed bytes exercise the approved placeholder and sizing workflows.
- [x] Complete physical notification-area/hotkey/Shake acceptance. Secondary activation, Open Shelf, Cooperative Shutdown, same-candidate reinstall/update, and restart were observed; resident/Shake native regressions passed.
- [x] Uninstall removes installed application bytes and registration while preserving complete owned Profile Data; source fixtures and ambient user data remain unchanged or are restored to the captured baseline.
- [x] Owned smoke profiles, fixtures, WebView data, processes, installer registrations, shortcuts, and temporary harnesses are removed or restored after verification.
- [x] Product specification, architecture, glossary, and roadmap remain aligned with the delivered behavior; release evidence and explicit observation limits are recorded in the roadmap without rewriting completed historical ticket evidence.
- [x] Final human acceptance gates are reported separately from agent-observed evidence.
- [x] No Git commit is created without separate explicit user authorization after the release candidate and acceptance evidence are complete.

## Answer — exact candidate, 2026-10-20

- Candidate: build/DropCove-Setup-0.0.16.exe, 3,287,614 bytes, SHA-256 5002330c008cdde3978857355e9f86f331b968a614d87442303747d4c3605a29.
- Exact installed executable: 13,869,568 bytes, SHA-256 eeccf4037e0c547aac3cd3181096703ec8315a6b93741e7bea1e6fc3479f77a3. The owned installation was removed after verification; the retained installer is the acceptance candidate.
- Complete frontend production build and cargo-xwin Windows release/NSIS packaging passed. The final 0.0.16 native test executable dropcove_lib-1ae260f7822f63f3.exe passed 113/113 directly on Windows. An earlier run of the previous test artifact is not the final-version proof. External SDK LNK4099 missing-PDB warnings remain nonfatal and unsuppressed.
- PE FileVersion was 0.0.16.0; runtime app version and uninstall DisplayVersion were 0.0.16. Installed assets loaded from http://tauri.localhost/assets/index-oUXleKGu.js with the owned Vite service stopped and port 1420 refusing connections.
- Exact installed bytes passed real OLE Shelf/Rail feedback, no-glow and no-preview-resize checks, Left/Right Rail focus preservation, accepted intake, immediate Shelf shrink, restart retention, secondary activation, and Cooperative Shutdown. Dark/Light development visuals and installed forced-colors/reduced-motion emulation were inspected.
- Silent install and same-candidate reinstall exited 0; installed executable hash and owned Profile Data hashes remained unchanged across reinstall. Restart restored three batches including the Pinned file-75.txt reference.
- Uninstall exited 0, removed the installed binary and directory, and preserved every owned Profile Data file byte-for-byte. Source fixtures and ambient profile hashes stayed unchanged. Installer registration, shortcut bytes, and the original captured HKCU Run value were restored exactly. The owned Resident, OLE source, Vite service, WebView attachments, fixtures, profiles, copies, and temporary harness directory were removed.
- Independent Standards and Spec reviewers both returned **no actionable findings** against HEAD `df81b67cfc9864ef4d17cc420bbeceb3ad340db3`, the scoped uncommitted implementation, and the new DropPlaceholder module. Standards checked sizing/manual intent, leading-slot virtualization, lifecycle cleanup, accessibility, and absence of speculative abstractions. Spec checked both surfaces, native timing/geometry preservation, exact bidirectional tiers, manual precedence/final reset, and ordinary/virtualized collection behavior. Physical hardware gates remain unverified rather than inferred from source review.

### Final human gate

The user reported on 2026-10-20 that Ticket 04 passed and the accepted `0.0.16` work was committed, closing the final human gate. Historical agent-observation limits above remain recorded rather than being reclassified as agent-verified coverage.
