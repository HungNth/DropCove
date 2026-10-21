# 01: Follow pointer on Dismiss Drop Shelf

**What to build:** Make every fresh non-empty Dismiss Drop Shelf action place the Edge Rail on the user's current interaction display. A connected explicit Target monitor remains authoritative; otherwise the monitor containing the pointer wins, with deterministic fallbacks. Rename the automatic Settings option to `Follow pointer`, persist the actual monitor selected by a successful dismissal, and keep every dismissal entry point behaviorally identical.

**Blocked by:** None (can start immediately).

Status: resolved

- [x] The Settings Target monitor option for an empty `RailMonitorId` is named exactly `Follow pointer`; existing Settings files require no migration and retain every other preference.
- [x] A fresh non-empty Dismiss Drop Shelf operation selects a connected explicit Target monitor before considering the pointer.
- [x] With no connected explicit Target monitor, a fresh dismissal selects the monitor containing the pointer at the start of the operation.
- [x] A configured but disconnected explicit Target monitor remains unchanged in Settings while dismissal temporarily continues through the automatic fallback policy.
- [x] If pointer-monitor resolution fails, selection falls back in order through the Drop Shelf monitor, latest persisted dismissal monitor, then Windows primary display.
- [x] `X`, `Escape`, global hotkey, and native Drop Shelf close requests converge on the same Resident-owned monitor-selection and dismissal transaction; no frontend caller reproduces placement policy.
- [x] A successful non-empty dismissal persists the actual selected monitor as the latest dismissal monitor and presents the Rail Handle centered on the configured edge of that monitor.
- [x] A later fresh dismissal on another pointer monitor replaces the previous persisted dismissal monitor instead of being overridden by it.
- [x] Dismissing an empty Drop Shelf still hides every Shelf surface and does not select, show, or persist an Edge Rail monitor.
- [x] All dismissal paths preserve every Shelf Item, Profile Data, source file, Resident lifecycle, Manage Items cleanup, and existing failure-safe behavior.
- [x] Deterministic regression coverage includes: persisted secondary plus pointer/Shelf on primary; connected explicit override; disconnected explicit fallback without Settings mutation; pointer lookup failure through all fallback levels; persistence of the actual selected monitor; and empty dismissal.
- [x] Actual Windows Resident/WebView2 smoke on two connected monitors proves `X`, `Escape`, hotkey, and native close place the Rail Handle on the expected physical monitor, with native HWND visibility/bounds and persisted monitor identity observed.
- [x] Canonical ADR and Product Specification language remains aligned with the fresh-dismissal policy; no schema, dependency, compatibility path, generalized window manager, or new domain term is introduced.
- [x] No Git commit is created without separate explicit user authorization.

## Comments

- 2026-10-21: Resolved. The final `0.0.18` NSIS payload was exercised with a non-empty isolated Shelf on both connected 96-DPI displays. Actual `X`, `Escape`, registered hotkey, and native `WM_CLOSE` produced a visible Rail Handle at primary bounds `(1904,376,1920,656)` or secondary bounds `(3824,376,3840,656)` according to the pointer, and SQLite recorded the matching monitor. Source and Profile Data fixture hashes survived lifecycle checks. Raw assertions: `build/smoke_final_candidate.log`. Full native suite: 118/118. Release acceptance and commit approval remain separate.
