# 06: Release DropCove 0.0.26

**What to build:** Integrate the accepted Item Selection, occurrence-safe item-set transfer, mixed availability, accessibility, focus, and cross-origin lifecycle slices into one exact DropCove 0.0.26 Windows release candidate. Review the complete change, synchronize version metadata only after behavior acceptance, package through cargo-xwin, and prove the installed production bytes through the approved pointer, keyboard, accessibility, native Copy, lifecycle, and installer seams.

**Blocked by:** 05: Cross-origin Item Selection lifecycle integration.

**Status:** resolved — candidate accepted by the user; commit authorized

**Parent specification:** [DropCove 0.0.26 — Manage Items Item Selection and Drag-Out](../spec.md)

**Dependencies:** Tickets 01–05, existing Windows x64 cargo-xwin build and NSIS packaging, owned Profile Data smoke setup, real Copy destinations, and Windows accessibility inspection or a screen reader.

**Testing seam:** One exact installed Windows x64 candidate running embedded production assets with owned Profile Data, real Drop Shelf and Edge Rail origins, ordinary and virtualized Manage Items collections, real pointer modifiers and keyboard input, accessibility inspection, native Copy/cancel/reject destinations, unavailable storage, and mixed external/Managed content.

**Demo path:** Install the exact candidate, execute the complete parent-spec Demo Path from both origins, restart and repeat retained-state checks, perform same-candidate reinstall/update, uninstall while preserving Profile Data, and present exact hashes and observed limits for human acceptance.

- [x] Tickets 01–05 are resolved and their accepted behavior and documentation changes are integrated before release metadata changes begin.
- [x] Independent standards, specification, accessibility, architecture, maintainability, needless-complexity, and security reviews leave no unresolved actionable defect.
- [x] Package, crate, lockfile, Tauri, PE, About, installer, and uninstall metadata are synchronized to `0.0.26` only in this release ticket.
- [x] Frontend interaction tests, frontend typecheck/production build, native Rust regressions, and the freshly cargo-xwin-built Windows regression executable pass.
- [x] The installed candidate runs embedded production assets with the development server unavailable.
- [x] Installed pointer acceptance covers plain/Ctrl/Shift/Ctrl+Shift, anchor edge cases, drag threshold, selected/unselected row drag, action isolation, empty-space clear, ordinary collections, and collections above 50 items.
- [x] Installed keyboard/accessibility acceptance covers the multiselectable grid, selected rows, roving focus, Up/Down/Home/End, Space variants, Ctrl+A, cyclic Tab order, announcement text, and focus recovery.
- [x] Edge Rail acceptance proves pointer selection, confirmation, Copy, pending close, and popup dismissal never steal foreground focus.
- [x] Native transfer acceptance proves source order, case-insensitive path deduplication, repeated occurrence recording, same-name different-path retention, singleton Unicode text, and multi-item storage-only payloads.
- [x] Mixed availability acceptance proves durable Missing cleanup, retained Unavailable occurrences, aggregate occurrence counts, no-Available status, direct confirmation drag, armed retry, stale-token cancellation, and Escape ordering.
- [x] Ownership acceptance proves Temporary consumption, current-Pinned retention, external-source safety, Managed File permanent deletion, real sharing-failure pending cleanup, and retry.
- [x] Persistence-failure and cleanup-failure smoke proves no partial consumption, stale authoritative token, unowned Managed File, or lost cleanup responsibility.
- [x] One-item survival, final-batch disappearance, popup replacement, owner hide, resize, active-drag delayed close, and restart follow the approved lifecycle.
- [x] Dark, Light, Windows Contrast, and reduced-motion inspection keeps selected, focused, hovered, disabled, warning, and destructive states distinct.
- [x] Exact-candidate restart, same-version reinstall/update, standard uninstall, shortcut/registration behavior, and Profile Data preservation pass.
- [x] Exact installer and executable hashes, test counts, Windows/accessibility tooling, destination applications, focus-owner observations, and unobserved physical limitations are recorded.
- [x] Human candidate acceptance, Git commit authorization, and deployment remain separate explicit decisions.

## Release evidence — 2026-10-26

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `build/DropCove-Setup-0.0.26.exe` | 3,688,557 | `f02bd5ddbf3343a711ebe7aeb7ee3fb44b5848d387f952f923ce30704f2da78b` |
| Built Windows x64 executable | 15,243,264 | `2673c22164a55f3dd2d9ad23ad3e551f0867283f5cc74aa9a9d0c2d84de49ae7` |
| Exact installed executable | 15,243,264 | `ebb5aa4143a1a6bb901d699d63f4ba97cfff3fe0b8e67802e1ef2ab9eb3cc66d` |

### Automated and review evidence

- `pnpm test`: **43 passed, 0 failed**. Production build included `vue-tsc` and Vite; both passed.
- `cargo xwin test --target x86_64-pc-windows-msvc --lib --no-run`, followed by the freshly compiled Windows regression executable: **185 passed, 0 failed** at version `0.0.26`.
- `pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc` and installer-copy step passed. No Visual Studio or commercial build-tool installation was used. External SDK missing-PDB `LNK4099` linker warnings remain non-fatal.
- Independent Standards and Spec reviews found no remaining actionable findings after shared prepared-drag extraction and cancellation fixes. The alleged native setup-failure token leak was rejected with reviewer confirmation: beginning a transfer takes the token out of the pending slot, and both Arc owners drop on failure without consumption.
- Rollback, current-Pin, deduplication, occurrence completion, pending deletion, ownership, and persistence failures were exercised through the approved real SQLite/filesystem regression seam. Installed sharing-error behavior was separately exercised with a real Windows delete-denying file handle; installed persistence failure was not injected with a debugger.

### Exact installed Windows evidence

- Embedded `http://tauri.localhost/` assets ran while TCP port `1420` had no development server. PE file version, actual About surface, and uninstall registration reported `0.0.26`.
- Ordinary and virtualized Manage Items surfaces ran from Drop Shelf and Edge Rail. The 65-item installed grid retained **240 × 480** geometry, **54-pixel** rows, and **10** mounted rows at the end. Installed Shift-click across the viewport selected **56** items; Ctrl+Shift replaced selection with **3**; toggle-off-anchor Shift selected **5**; empty space cleared selection; no-anchor Shift selected **1**.
- Actual keyboard opening focused the first item drag region. Tab followed Bulk Pinning, Close, grid stop, and source-order Pin/Remove. The Chromium accessibility tree exposed selected rows and independent gridcells/buttons; the live channel reported `65 items selected`. Ordinary same-index and virtualized previous-item focus recovery, one-item-card explicit-close fallback, and disappearing-batch parent fallback were observed.
- Real native mouse gestures transferred selected sets and unselected singletons into an owned Windows Forms OLE destination. Source order, exact copied bytes, Temporary consumption, Pinned retention, cancel/reject retention, selected-clear after Copy, direct confirmation drag, armed retry, and stale-selection cancellation were observed.
- Singleton text delivered exactly `Xin chào DropCove\r\n第二行 🚀` through Unicode text-only formats with no FileDrop. A text Managed File plus an external occurrence delivered two FileDrop paths, no UnicodeText format, and byte-exact destination files. The user explicitly reconfirmed preservation of 0.0.25 text-only singleton behavior; the earlier dual-format specification sentence was corrected.
- Missing cleanup, Available/Unavailable confirmation, Unavailable-only status, one-item popup survival, and item-occurrence isolation from an unselected batch sharing the same path were observed. A real sharing violation committed Temporary Managed File consumption, retained one pending deletion, and displayed the existing Shelf warning; releasing the handle and removing another reference completed permanent deletion and cleared the pending row.
- Edge Rail kept the same external foreground HWND through native pointer opening, selection, mixed confirmation, direct confirmation Copy, cancellation, and close. Rail popup origin was `edge-rail`, keyboard activation `false`, with zero selection live regions. Active-drag close stayed pending until native cancellation completed, then closed the popup without restoring selection or stealing focus. Released management holds allowed the 280-pixel expanded Rail to collapse to its 16-pixel handle.
- Restart restored **65** exact item identities, paths, and Pin states, with zero selected rows on reopening. Silent current-user install, same-candidate reinstall/update, and uninstall each returned **0**. Reinstall and uninstall preserved every owned Profile Data file byte-for-byte, including existing SQLite sidecars.
- The original HKCU installation/uninstall records, Run value, and Start Menu shortcut were restored or remained byte-exact. The original installed executable hash remained unchanged. All **75** owned external source hashes remained unchanged. Owned smoke processes, test profiles, destination files, scripts, and temporary WebView data were removed.

### Observation limits and acceptance gate

- Native input was automated with real Windows mouse/keyboard events at scale 1; physical high/mixed-DPI hardware, touchpad input, and spoken screen-reader output were not exercised. Accessibility-tree inspection was used instead of a screen reader.
- Dark and Light resource projection were visually inspected. Windows Contrast and reduced-motion were inspected through active Chromium media emulation; the user's Windows Contrast Theme and animation settings were not changed.
- A filesystem-only destination correctly rejected singleton text-only before the owned destination was configured to accept Unicode text. An earlier destination point overlapped the always-on-top Shelf; corrected native geometry produced the text-only acceptance recorded above. These fixture failures are not counted as passes.
- Candidate human acceptance and any commit/deployment authorization remain explicit user decisions. No human acceptance is inferred from automated or installed proof.

## Human handoff — 2026-10-26

The user initially selected “Giữ uncommitted để tôi test”. After testing, the user confirmed “tôi đã test và tất cả đã đạt, hãy commit các thay đổi”, accepting the 0.0.26 candidate and explicitly authorizing the commit. This supersedes the earlier deferral. No publish or deployment was performed.
