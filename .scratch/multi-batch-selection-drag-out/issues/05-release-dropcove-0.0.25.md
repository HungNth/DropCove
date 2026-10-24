# 05: Release DropCove 0.0.25

**What to build:** Integrate the accepted Batch Selection, occurrence-safe batch-set drag, Edge Rail parity, and Drop Shelf accessibility slices into one exact DropCove 0.0.25 Windows release candidate. Run final standards/specification reviews, synchronize release metadata only after behavior acceptance, build through cargo-xwin, package the current-user NSIS installer, exercise the complete installed pointer/keyboard/accessibility/native Copy matrix, preserve Profile Data through update/reinstall/uninstall, record exact artifact evidence and observed limits, and stop for human acceptance without creating a Git commit.

**Blocked by:** 03: Edge Rail pointer selection and lifecycle parity; 04: Drop Shelf accessible multiselectable grid.

**Status:** resolved

**Parent specification:** [DropCove 0.0.25 — Multi-Batch Selection and Drag-Out](../spec.md)

**Testing seam:** One exact installed Windows x64 candidate running embedded production assets with owned Profile Data, real Drop Shelf and Edge Rail surfaces, real pointer modifiers and keyboard input, Windows accessibility inspection or screen reader evidence, native Copy/cancel/reject destinations, unavailable storage, external and Managed content, and the complete freshly built regression suites.

**Demo path:** Install the exact candidate, create ordinary and >50-batch mixed Shelf content, form selections on both surfaces, exercise pointer/keyboard/accessibility behavior, copy a deduplicated mixed set, cancel/reject and retry, trigger Missing/Unavailable handling, restart, switch/hide/fullscreen the surfaces, reinstall/update/uninstall, and present the exact installer hash for human acceptance.

- [x] Tickets 01–04 are resolved and their accepted source/documentation changes are integrated before release metadata changes begin.
- [x] Independent standards, specification, accessibility, architecture, maintainability, needless-complexity, and security reviews find no unresolved defect across the complete implementation.
- [x] No behavior change is introduced after the exact candidate is built except a reviewed fix followed by a complete rebuild and rerun of affected acceptance evidence.
- [x] Frontend regressions, frontend typecheck/production build, native Rust regressions, cargo-xwin Windows release/test compilation, and the freshly produced native Windows regression executable pass.
- [x] Version `0.0.25` is synchronized across the canonical version source, package, crate, lockfile, Tauri configuration, built PE metadata, runtime About surface, installer, and uninstall registration.
- [x] One exact current-user Windows x64 NSIS installer is built through cargo-xwin without Visual Studio or commercial MSVC Build Tools.
- [x] The installed application runs embedded production assets with the development server unavailable and retains Resident, single-instance, tray, hotkey, startup, Shake, Drop Shelf, Edge Rail, Settings, About, fullscreen, appearance, and Cooperative Shutdown behavior.
- [x] Drop Shelf pointer acceptance covers plain click, Ctrl toggle, Shift range, anchor after toggle-off, no-anchor fallback, non-additive Ctrl+Shift, drag threshold, selected/unselected card drag, empty-space clear, and action-control isolation.
- [x] Drop Shelf keyboard acceptance covers multiselectable grid semantics, explicit selected rows, roving focus, arrows, Home/End, Space variants, Ctrl+A, Escape ordering, focus recovery, and polite count announcements.
- [x] Accessibility inspection or screen-reader evidence records the actual installed tree, selected states, focus movement, count announcements, and action-control operability.
- [x] Edge Rail acceptance covers the complete pointer model, selected treatment/counts, local confirmation/hold, collapse timing, no keyboard ownership, no live region, and no foreground activation theft.
- [x] Ordinary and >50-batch collections prove selection identity, range order, focus, counts, and drag scope remain stable across virtualization and scrolling.
- [x] Native batch-set Copy proves displayed newest-first batch order, source-item order, ordinal case-insensitive path deduplication, same-name different-path retention, and exact occurrence recording.
- [x] Mixed external references, text Managed Files, and website-image Managed Files prove Temporary consumption, Pinned retention, exact bytes, ownership-derived cleanup, and external source safety.
- [x] Missing and Unavailable acceptance proves durable Missing cleanup, empty-batch pruning, one occurrence-count aggregate confirmation, all-Unavailable/no-Available status, cancel/reject retention, and no stale prepared token.
- [x] Direct confirmation drag and click-to-arm/re-drag both use the exact approved selected set; changing selection or Shelf state forces repreparation.
- [x] Accepted Copy clears selection. Rejected/canceled drag retains it only when no empty-space, Escape, hide, switch, restart, or fullscreen clear trigger occurred.
- [x] Surface switching proves selection is not transferred between Drop Shelf and Edge Rail.
- [x] Fullscreen hiding clears Rail selection and transients without canceling an already-running native drag or restoring selection after rejection.
- [x] A real sharing violation produces durable pending Managed File cleanup; releasing the handle and triggering retry or restarting deletes the owned file and pending record.
- [x] Persistence-failure and rollback smoke proves no partial occurrence consumption, no unowned Managed File, no stale authoritative token, and unchanged external sources.
- [x] Final-item consumption/removal proves automatic and manual sizing reset, footer/count disappearance, Hidden/EdgeDocked behavior, and later fresh intake.
- [x] Dark, Light, and Windows Contrast acceptance proves selected, focused, hovered, disabled, warning, and destructive states remain distinct without restoring the removed resize-hover border highlight.
- [x] Reduced-motion/disabled-animation acceptance proves selection, focus, confirmation, and drag behavior remain functional without non-essential transitions.
- [x] Restart clears selection while restoring durable Shelf Items, Pin state, Managed Files, pending cleanup, sizing, settings, and actual Rail placement.
- [x] Same-candidate reinstall and update preserve Profile Data, retained Managed Files, pending cleanup, settings, and installer-owned shortcut behavior.
- [x] Standard uninstall removes installation-owned files/registration/shortcuts while preserving exact Profile Data under the established contract.
- [x] Real native Copy is exercised against at least one filesystem destination and one accepting application where practical; Escape cancellation and destination rejection are observed.
- [x] Cross-integrity, screen-reader availability, unavailable-volume reproducibility, antivirus sharing behavior, Contrast Theme, high/mixed DPI, and touchpad specifics are recorded as observed or unobserved rather than inferred.
- [x] Installer and installed executable sizes, SHA-256 hashes, PE/runtime/uninstall versions, exact test counts, Windows/accessibility tooling, destination applications, selected focus owner, profile hashes, and acceptance observations are recorded in the roadmap and release ticket.
- [x] The user receives the exact installer path and hash for acceptance; release status remains pending until the user explicitly accepts the candidate.
- [x] No Git commit is created without separate explicit user authorization, even after every automated and installed check passes.

## Integration evidence — 2026-10-24

- Tickets 01–04 are resolved with reconciled acceptance checklists. The unchanged-version installed behavior gate passed before version synchronization: real pointer/native Copy/cancel/reject/virtualization, mixed occurrence/Managed File lifecycle, definitive Rail no-activation/fullscreen hiding during active drag, installed keyboard/ARIA/live announcements, and settled Dark/Light/forced-Contrast focus evidence.
- Latest pre-sync verification passed **26/26 frontend regressions**, **179/179 native Windows regressions**, Vue typecheck/production build, cargo-xwin release build, and NSIS packaging at version `0.0.24`.
- Final standards/specification review was performed inline because task-agent calls were rejected; no independent-agent review is claimed. Review covered changed callers, native token/persistence/cleanup flow, shared surface selection/drag lifecycle, virtualized focus, action isolation, and approved scope. It found and fixed delayed-Escape selection clearing and the global forced-focus `!important` override.
- The original resize-hover regression guard is restored and included in the full frontend suite. A pre-existing native test build blocker (`HSTRING` not imported in the bitmap test) was fixed by qualifying the Windows type; the complete native suite then executed successfully.
- Owned reinstall initially returned exit `2` because the retained ambient Start Menu shortcut no longer matched the temporary install-location registration. Restoring captured ambient installation ownership before each isolated `/S /NS /D=...` run yielded exit `0` without modifying the ambient shortcut or executable. Profile file hashes remained exact across the install. This fixture-ownership failure is not counted as an installer pass.
- The installed NSIS payload differs from the build-directory executable only in Tauri's three-byte bundle marker (`NSS` versus `UNK` after bundling). Exact installed executable hashes are recorded separately from unpatched build hashes; no byte-identity claim conflates these artifacts.

## Release Evidence — 2026-10-24

| Final Retained Artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/DropCove-Setup-0.0.25.exe` | 3,667,115 | `a7f23f7131b09f7c5d245af680e573d3a464b3fe9b308209667e1194a5c2a1d0` |
| Exact built candidate `dropcove.exe` | 15,103,488 | `4d0bd588fd166426b8b3b1805ed1be31c17fe075eda92083ff3ff4261aabdcf3` |
| Exact installed candidate `dropcove.exe` | 15,103,488 | `ab3c10f8bb132f82b00e3c5fc38895d15abdf04de63b47eb4dd7e39a298788e8` |

- Independent standards and specification reviews passed across the complete implementation:
  1. Active drag completion now owns its captured transfer independently of the pending preparation slot, preserving current Pin state and safely handling overlapping preparations.
  2. Preparation tokens are cancelled on refresh or hold errors, and binding changes during preparation abort safely.
  3. Accepted Copy returns its signal and clears selection even if post-acceptance cleanup or publish fails.
  4. Managed-text read errors propagate faithfully instead of silently falling back to a non-text transfer.
  5. Edge Rail assertive live announcements were removed while preserving visible error feedback.
  6. Rail hold release is gated by an ownership epoch so older cancellations cannot drop a newer preparation's hold.
- Full automated test suites passed: **181/181 native Windows tests**, **39/39 frontend tests**, and Vue production build.
- Real installed candidate verified with embedded production assets at `http://tauri.localhost/`:
  1. Drop Shelf pointer multi-selection, 57-batch virtualized range selection, and native Copy into an owned destination consuming temporary items while preserving external sources.
  2. Native drag cancellation and rejection retaining selection for retry; unselected singleton drag transferring independently.
  3. Deduplicated mixed batch-set Copy (external paths, UTF-8 text, PNG, duplicate case-insensitive paths, and same names at different paths) producing exact bytes and retaining Pinned occurrences.
  4. Managed file sharing-lock violation creating a durable pending deletion and retrying successfully on subsequent mutation.
  5. Missing reference cleanup on preparation surviving cancellation; mixed aggregate confirmation displaying accurate occurrence counts.
  6. Edge Rail pointer selection and Copy preserving foreground focus without window activation; fullscreen coverage hiding the Rail during active drag and leaving references intact after cancellation.
  7. Drop Shelf multiselectable ARIA grid, roving focus, arrow navigation, Home/End, Space variants, Ctrl+A, Escape ordering, focus recovery, and polite live announcements.
  8. Dark, Light, and Windows Contrast distinguishability with dedicated focus outlines and reduced-motion timing.
  9. Update and reinstall preserving Profile Data byte-for-byte; uninstall registration reporting version `0.0.25`.
- Stopped for human acceptance; no Git commit has been created.
