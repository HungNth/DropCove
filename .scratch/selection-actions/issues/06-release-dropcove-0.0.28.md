# 06: Release DropCove 0.0.28

**Parent specification:** [DropCove 0.0.28 — Selection Actions](../spec.md)

**What to build:** Integrate and release the complete Selection Actions workflow as DropCove 0.0.28. Prove atomic lifecycle actions, ownership-safe removal, keyboard and pointer menus, confirmation, virtualization, accessibility, Edge Rail no-activation, Managed File cleanup, source safety, installer lifecycle, and Profile Data preservation from one exact installed candidate.

**Blocked by:** 05 — Edge Rail and Rail-origin Selection Actions parity.

**Status:** ready-for-agent

**Dependencies:** Completed Tickets 01–05, canonical Product Specification and Architecture contracts, cargo-xwin Windows toolchain, existing current-user NSIS packaging, owned-profile release harness, and separate implementation authorization.

**Testing seam:** The user-approved exact installed Windows x64 Resident Instance using embedded production assets across Drop Shelf, expanded Edge Rail, and Manage Items from both origins, with real pointer/keyboard input, foreground observation, filesystem fixtures, locked Managed Files, restart/reinstall/uninstall, and complete supporting frontend/native regressions.

**Demo path:** Install the exact candidate, create ordinary and virtualized External/Managed mixed selections, exercise every singular/plural Pin/Unpin/Remove path and confirmation variant on all surfaces, inspect accessibility and foreground ownership, force pending cleanup, restart/reinstall/uninstall, then present exact artifacts and evidence for human acceptance.

- [ ] Tickets 01–05 are integrated without compatibility shims, duplicate menu models, obsolete suppression-only paths, or partial selection-action commands.
- [ ] Canonical, package, crate, lockfile, Tauri, built/installed executable, About Surface, installer, and uninstall metadata report `0.0.28` only in this release slice.
- [ ] Complete frontend interaction/accessibility regressions pass, including ordinary and virtualized menu targeting, labels, confirmation, focus, Escape/Tab, selection lifecycle, and Rail ownership.
- [ ] A freshly cargo-xwin-compiled Windows native regression executable passes atomic selection mutation, stale validation, rollback, source safety, Managed File cleanup, pending retry, restart, and sizing transitions.
- [ ] Vue production typecheck/build, cargo-xwin Windows release build, and current-user NSIS packaging succeed under the project toolchain constraints.
- [ ] One exact installed candidate runs embedded production assets with the development server unavailable and isolated owned Profile Data/WebView data.
- [ ] Installed proof covers Drop Shelf, expanded Edge Rail, Drop-Shelf-origin Manage Items, and Rail-origin Manage Items with ordinary and more-than-50-member collections.
- [ ] Real input proves selected/unselected secondary-click targeting, ignored modifiers, Context Menu key and `Shift+F10` on focus-capable surfaces, pointer-only Rail behavior, labels, menu navigation, Tab, Escape, and focus recovery.
- [ ] Pin/Unpin proof covers one-item batch, multi-item batch, multiple batches, one item, multiple items, None, Mixed, All Pinned, Available, Missing, and practically reproducible Unavailable targets.
- [ ] Removal proof covers immediate singleton actions and all-External, mixed-ownership, and all-Managed confirmations with exact counts and copy.
- [ ] External fixtures remain byte-exact; Managed Files permanently delete or enter durable pending deletion under a real sharing failure and complete after retry/restart.
- [ ] Stale-target and persistence-failure proof shows no partial Pin state, reference removal, cleanup authority loss, sizing corruption, selection loss, or popup lifetime corruption.
- [ ] Foreground observation proves Edge Rail and Rail-origin Manage Items remain no-activate throughout menus, confirmation, success, and failure; explicit Item Action Explorer reveal remains the only approved foreground transfer in this area.
- [ ] Accessibility inspection covers menu/dialog roles, names, selected scope, ownership consequences, focus order/restoration, announcements, Dark, Light, and Windows Contrast resources.
- [ ] Restart restores exact retained identities, order, paths, Pin states, Managed ownership, sizing, and settings while discarding transient selection/menu/confirmation state.
- [ ] Same-candidate reinstall/update and standard uninstall return successfully and preserve Profile Data according to existing policy.
- [ ] Owned fixtures, profiles, WebView data, installation, processes, transient windows, registry/shortcut state, clipboard test data, and pending cleanup probes are cleaned without changing ambient user data.
- [ ] Exact test counts, Windows version, tools, input method, foreground observer, artifact sizes, SHA-256 hashes, installed executable identity, observed coverage, and unobserved physical limits are recorded.
- [ ] Human candidate acceptance, Git commit authorization, publishing, and deployment remain separate explicit gates; this ticket creates no commit without authorization.
