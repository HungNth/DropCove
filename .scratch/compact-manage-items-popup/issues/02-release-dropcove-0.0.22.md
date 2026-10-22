# 02: Release DropCove 0.0.22

**What to build:** Produce and verify the exact DropCove 0.0.22 Windows x64 current-user NSIS release containing the approved compact Manage Items popup. Synchronize release metadata only after the implementation slice is accepted, exercise the installed candidate end to end, record exact evidence and limits in the roadmap, and present the candidate for human acceptance.

**Parent specification:** [DropCove 0.0.22 — Compact Manage Items Popup](../spec.md)

**Blocked by:** 01 — Compact Manage Items presentation.

**Status:** resolved — user tested and accepted candidate 0.0.22 on 2026-10-22; commit explicitly authorized.

## Acceptance criteria

- [x] Ticket 01 is accepted on the actual Windows Resident/WebView2 surface before release metadata changes begin.
- [x] Standards and specification review find no actionable issue, or findings are resolved and affected verification is rerun.
- [x] Canonical version, package, crate, lockfile, Tauri, executable, installer, and uninstall metadata are synchronized exactly to `0.0.22` with no stale `0.0.21` runtime or packaging identity.
- [x] Frontend production build and existing frontend regression suite pass.
- [x] Cargo-xwin Windows release/test compilation passes and the freshly produced Windows native regression executable passes when run directly.
- [x] Actual Windows inspection covers Manage Items from both the Drop Shelf and Edge Rail with the approved `240`-pixel width, name-plus-availability row layout, absent visible path and `Pinned ·` prefix, retained full-path tooltip/accessibility text, keyboard/focus, direct drag-out, scrolling, and virtualization.
- [x] Dark and Light presentation are inspected; actual Windows Contrast Theme and reduced-motion behavior are inspected where available, with unavailable configurations recorded as unobserved rather than inferred.
- [x] One exact current-user Windows x64 NSIS candidate is produced; installer and installed-executable byte sizes and SHA-256 hashes are recorded and provisional artifacts are explicitly superseded.
- [x] The exact installed candidate loads embedded frontend assets with the development server unavailable and reports version `0.0.22`.
- [x] Installed smoke covers short and virtualized Manage Items lists from both origins plus intake, duplicate handling, Pin/Unpin, Bulk Pinning, Remove Item/Batch, Clear Temporary Items, item/batch Copy drag-out, cancellation, Drop Shelf dismissal, Edge Rail transitions, Settings/About access, and source-file safety.
- [x] Secondary activation, resident startup/access, Cooperative Shutdown, same-candidate reinstall/update, and restart complete without duplicate state owners or orphaned surfaces.
- [x] Uninstall removes installed application bytes and registration while preserving complete owned Profile Data; owned profiles, fixtures, WebView data, processes, installer registrations, shortcuts, and temporary harnesses are cleaned or restored.
- [x] `docs/ROADMAP.md` records observed behavior, exact artifact evidence, restoration, human-reported checks, and unobserved limits without rewriting historical release evidence.
- [x] Final human acceptance is obtained for the exact candidate before the roadmap marks 0.0.22 Completed.
- [x] No Git commit is created without separate explicit user authorization after acceptance.

## Testing seam

The highest seam is the exact installed Windows 11 x64 candidate and its real WebView2/native windows. Existing frontend and native regressions support that evidence but do not replace visual, interaction, installer, lifecycle, or cleanup verification.

## Demo path

Build and install the exact 0.0.22 candidate into an owned profile. Open one mixed batch from the Drop Shelf and Edge Rail, show the 240-pixel layout, availability-only second line and retained full path, exercise pointer/keyboard/drag/scroll behavior, then demonstrate resident access, restart, reinstall, Cooperative Shutdown, uninstall, and Profile Data preservation. Present the retained installer and evidence for user acceptance.

## Non-goals

New product behavior beyond the approved compact Manage Items change, unrelated refactors, schema migration, new dependencies, signing, automatic updates, non-Windows platforms, rewriting historical roadmap evidence, or committing without explicit authorization.

## Evidence and remaining gate — 2026-10-22

- Retained candidate: `build/DropCove-Setup-0.0.22.exe`. Exact final artifact sizes/hashes, superseded provisional bytes, smoke corrections, and cleanup evidence are recorded once in [ROADMAP 0.0.22](../../../docs/ROADMAP.md#0022--compact-manage-items-popup).
- Final frontend typecheck/production build and cargo-xwin NSIS build passed; frontend tests **13/13**, native Windows tests **124/124**, placement checks **3/3**. Direct Standards and Spec reviews found no remaining actionable implementation finding; independent parallel review was unavailable.
- The final candidate fixed ordinary-list Flexbox shrink: 35 ordinary rows measured 54 pixels from both origins; 70-item virtualization remained bounded and keyboard traversal reached row 69 before wrapping. Three-row presentation was `240 × 292`. Exact path accessibility and Available/Missing/Unavailable content were observed; Dark/Light were inspected. Native item and batch Copy/cancellation, Temporary consumption, Pinned retention, destination bytes, and source safety were demonstrated with owned native targets. Supplemental exact-final-candidate smoke closed Bulk Pinning, item Pin/Unpin, and Remove Batch coverage.
- Earlier provisional-candidate installed smoke also exercised native intake, duplicate no-op, Clear cancellation/confirmation, and Settings/About. These are recorded as supporting preservation evidence, not misrepresented as a second full final-candidate interaction run.
- Final exact-candidate reinstall and secondary activation returned 0. Restart restored durable identities, order, paths, and Pin states; availability correctly refreshed a restored Missing fixture to Available. Standard uninstall returned 0, removed installed bytes and preserved exact owned Profile Data hashes. All owned smoke resources were removed, captured registrations/shortcuts restored, and the originally running ambient Resident restarted with unchanged executable/settings bytes.
- Startup Run registration was reconciled to the ambient installation's actual settings. Its original value was not captured before initial default-settings smoke startup; exact original Run-byte restoration is not claimed.
- Not observed: actual Windows Contrast Theme, mixed/fractional DPI, Narrator speech, physical Windows reduced-motion configuration, and the exhaustive physical interaction matrix. These remain human acceptance checks, not inferred passes.
- User action completed: the user verified the candidate, reported acceptance on 2026-10-22, and explicitly authorized the release commit.

### User acceptance — 2026-10-22

- User confirmed: `tất cả đã đạt, hãy commit các thay đổi`, closing human acceptance and authorizing the release commit.